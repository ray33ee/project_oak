use std::fmt::Formatter;
use crate::error::{Result, Error};
use crate::oak::OakRead;
use crate::oak::OakWrite;
use crate::registry_ex;

use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use std::io::Write;
use serde::{Serialize, Deserialize};
use tempfile::TempDir;
use registry::Security;
use registry_ex::RootKey;
use registry_ex::Data;
use crate::registry_ex::Tree;



#[derive(Debug, Serialize, Deserialize)]
pub enum FileType {
    File,
    Folder,
    Shortcut(PathBuf),
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum PathType {
    Absolute(PathBuf),
    Temporary(PathBuf),
}

impl PathType {
    pub fn path(&self, temp: Option<&TempDir>) -> PathBuf {
        match self {
            PathType::Absolute(path) => {
                path.clone()
            }
            PathType::Temporary(path) => {
                temp.unwrap().path().join(path)
            }
        }


    }

    pub fn is_temp(&self) -> bool {
        if let PathType::Absolute(_) = self {
            false
        } else {
            true
        }
    }
}

impl std::fmt::Debug for PathType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PathType::Absolute(p) => {write!(f, "p\"{}\"", p.to_str().unwrap())}
            PathType::Temporary(p) => {write!(f, "t\"{}\"", p.to_str().unwrap())}
        }
    }
}

impl From<&str>  for PathType {
    fn from(string: &str) -> Self {
        let key = "temp";

        if string.starts_with(key) {
            PathType::Temporary(PathBuf::from(&string[key.len()..]).strip_prefix("\\").unwrap().to_path_buf())
        } else {
            PathType::Absolute(PathBuf::from(string))
        }
    }
}

///A step is the smallest unit of an installation process
#[derive(Debug, Serialize, Deserialize)]
pub enum Step {
    Data{ name: String, destination: PathType },
    Move{ source: PathType, destination: PathBuf },
    Delete { path: PathBuf },
    Create { path: PathType, f_type: FileType },
    Copy { source: PathType, destination: PathType },
    Rename { from: PathType, to: PathType },
    Zip { folder: PathType, archive: PathType },
    Unzip { folder: PathType, archive: PathType },
    DeleteRegistryEntry { root: RootKey, key: String, value: Option<String> },
    WriteRegistryValue { root: RootKey, key: String, value: String, data: Data },
    WriteRegistryKey { root: RootKey, key: String },
    RestoreRegistryKey { root: RootKey, key: String, tree: Tree }, //Used only as the inverse of a DeleteRegistry key
    Download { url: String, destination: PathType },
    Edit{ source: PathType, command: String },
    RestoreEdit { name: String, destination: PathType }, //Used only as inverse of edit
    Print { message: String },
    Panic,
}

impl Step {

    ///Execute the action, and return the default inverse action if required
    pub fn action(
        & self,
        //vm: & mut VM,
        temp: Option<& TempDir>,
        install_archive: & mut OakRead,
        mut uninstall_archive: Option<& mut OakWrite>,
        //default: bool
    ) -> Result<Option<Step>> {

        match &self {
            Step::Panic => {
                Err(Error::Custom)
            }
            Step::Data { name, destination } => {
                let destination_path = destination.path(temp);

                install_archive.extract(&name, &destination_path)?;

                if destination.is_temp() {
                    Ok(None)
                } else {
                    Ok(Some(Step::Delete { path: destination_path }))
                }
            }
            Step::Move { source, destination } => {

                let source_path = source.path(temp);

                if destination.exists() {
                    Err(crate::Error::AlreadyExists)
                } else {
                    if source_path.is_dir() {
                        let mut options = fs_extra::dir::CopyOptions::default();
                        options.content_only = true;

                        fs_extra::dir::move_dir(&source_path, &destination, &options)?;

                    } else if source_path.is_file() {
                        let options = fs_extra::file::CopyOptions::default();

                        fs_extra::file::move_file(&source_path, &destination, &options)?;

                    } else {
                        panic!("File is not a path or file");
                    }

                    match source {
                        PathType::Absolute(s) => {
                            Ok(Some(Step::Move { source: PathType::Absolute(destination.clone()), destination: s.clone() }))
                        }
                        PathType::Temporary(_) => {

                            Ok(Some(Step::Delete { path: destination.clone() }))
                        }
                    }


                }

            }
            Step::Delete { path } => {

                let name = match uninstall_archive.as_mut() {
                    None => { None }
                    Some(archive) => {Some(archive.archive(&path))}
                };

                if path.is_dir() {
                    std::fs::remove_dir_all(&path)?;
                } else if path.is_file() {
                    std::fs::remove_file(&path)?;
                } else {
                    panic!("File is not a path or file");
                };

                if let Some(_) = uninstall_archive {
                    Ok(Some(Step::Data { name: name.unwrap(), destination: PathType::Absolute(path.clone()) }))
                } else {
                    Ok(None)
                }

            }
            Step::Create { path, f_type } => {


                let _path = path.path(temp);

                match f_type {
                    FileType::File => {
                        OpenOptions::new().create_new(true).write(true).open(&_path)?;
                    }
                    FileType::Folder => {
                        std::fs::create_dir(_path.as_path())?;
                    }
                    FileType::Shortcut(_) => {
                        todo!()
                    }
                }

                if path.is_temp() {
                    Ok(None)
                } else {
                    Ok(Some(Step::Delete { path: _path.clone() }))
                }
            }
            Step::Copy { source, destination } => {

                let source_path = source.path(temp);
                let destination_path = destination.path(temp);

                if destination_path.exists() {
                    Err(crate::error::Error::AlreadyExists)
                } else {
                    if source_path.is_file() {

                        std::fs::copy(&source_path, &destination_path)?;
                    } else if source_path.is_dir() {
                        let mut options = fs_extra::dir::CopyOptions::default();
                        options.content_only = true;

                        fs_extra::dir::copy(&source_path, &destination_path, &options)?;
                    } else {
                        panic!("Source is not a file or directory");
                    }

                    if destination.is_temp() {
                        Ok(None)
                    } else {
                        Ok(Some(Step::Delete { path: destination_path.clone() }))
                    }
                }
            }
            Step::Rename { from, to } => {

                let from_path = from.path(temp);
                let to_path = to.path(temp);

                if from.is_temp() != to.is_temp() {
                    panic!("Arguments for rename must either be both temporary, or both permanent locations")
                }

                std::fs::rename(from_path.as_path(), to_path.as_path())?;

                if from.is_temp() {
                    Ok(None)
                } else {
                    Ok(Some(Step::Rename {from: to.clone(), to: from.clone()}))
                }


            }
            Step::Zip { folder, archive } => {
                zip_extensions::write::zip_create_from_directory(&archive.path(temp), &folder.path(temp))?;

                if archive.is_temp() {
                    Ok(None)
                } else {
                    Ok(Some(Step::Delete { path: archive.path(temp) }))
                }

            }
            Step::Unzip { folder, archive } => {
                std::fs::create_dir(&folder.path(temp))?;

                zip_extensions::read::zip_extract(&archive.path(temp), &folder.path(temp))?;

                if archive.is_temp() {
                    Ok(None)
                } else {
                    Ok(Some(Step::Delete { path: folder.path(temp) }))
                }

            }
            Step::DeleteRegistryEntry { root, key, value } => {

                let reg = registry::Hive::from(root).open(key, Security::AllAccess)?;

                match value {
                    None => {


                        let tree = Tree::from(&reg);



                        reg.delete("", true)?; //Delete the contents of the key
                        reg.delete_self(false)?; //Delete the key itself




                        Ok(Some(Step::RestoreRegistryKey {
                            root: root.clone(),
                            key: key.clone(),
                            tree,
                        }))
                    }
                    Some(val) => {
                        let old_value = reg.value(val)?;

                        reg.delete_value(val)?;

                        Ok(Some(Step::WriteRegistryValue {
                            root: root.clone(),
                            key: key.clone(),
                            value: val.clone(),
                            data: Data::from(old_value)
                        }))
                    }
                }



            }
            Step::WriteRegistryValue { root, key, value, data } => {

                let reg = registry::Hive::from(root).open(key, Security::AllAccess)?;

                println!("reg: {}", reg);

                //For inverses, there are two cases. If the value already exists (i.e. we are modifying it)
                //and if the value does not already exist (i.e. we are creating it). In the first case,
                //the inverse is to revert to the previous value. In the second case, the inverse is
                //to delete the value.
                let inverse = if let Err(registry::value::Error::NotFound(_,_)) = reg.value(value) {
                    Step::DeleteRegistryEntry {
                        root: root.clone(),
                        key: key.clone(),
                        value: Some(value.clone()),
                    }
                } else {
                    //Save the old value
                    let old_value = reg.value(value)?;

                    Step::WriteRegistryValue {
                        root: root.clone(),
                        key: key.clone(),
                        value: value.clone(),
                        data: Data::from(old_value)
                    }
                };

                reg.set_value(value, &registry::Data::from(data))?;

                Ok(Some(inverse))
            }
            Step::WriteRegistryKey { root, key } => {

                let reg = registry::Hive::from(root); //.open(key, Security::AllAccess)?;



                //Look for the oldest ancestor that was newly created as part of this call.
                //For example, if a registry key looks like 'example\path\to\' before, and
                //'example\path\to\demonstrate\inverse' after, then you want to delete 'example\path\to\demonstrate'
                let common = {
                    let ancestors = Path::new(key).ancestors().collect::<Vec<_>>();

                    for ancestor in ancestors.iter() {
                        let o = reg.open(ancestor.to_str().unwrap(), Security::Read);
                        println!("an: {:?} {:?}", ancestor, o);
                    }

                    ancestors
                        .iter()
                        .rev()
                        .map(|x| {

                            let o= reg.open(x.to_str().unwrap(), Security::Read);

                            (x.clone(), o)
                        })
                        .find(|(_, x)| x.is_err())
                        .map(|(path, _)| path)
                };

                let inverse = common.map(|x| {
                    Step::DeleteRegistryEntry {
                        root: root.clone(),
                        key: String::from(x.to_str().unwrap()),
                        value: None
                    }
                });

                reg.create(key, Security::AllAccess)?;

                Ok(inverse)
            }
            Step::RestoreRegistryKey { root, key, tree } => {

                let path = Path::new(key);

                let mut anc = path.ancestors();

                anc.next();

                let path = anc.next().unwrap();



                tree.restore(root, path.to_str().unwrap()).unwrap();

                Ok(None)
            }
            Step::Edit { source, command } => {

                let name = match uninstall_archive.as_mut() {
                    None => { None }
                    Some(archive) => {Some(archive.archive(&source.path(temp)))}
                };


                //Load `source`
                let content = std::fs::read_to_string(source.path(temp))?;

                println!("before: {}", content);

                //Perform find and replace
                let res = sedregex::find_and_replace(content.as_str(), &[command]).unwrap();

                println!("after: {}", res.as_ref());

                //Save back to `source`
                let mut fh = OpenOptions::new().write(true).open(source.path(temp))?;

                fh.write_all(res.as_ref().as_bytes())?;

                if source.is_temp() {
                    Ok(None)
                } else {
                    Ok(Some(Step::RestoreEdit { name: name.unwrap(), destination: source.clone() }))
                }
            }
            Step::RestoreEdit { name, destination } => {

                let destination_path = destination.path(temp);

                std::fs::remove_file(&destination_path)?;

                install_archive.extract(&name, &destination_path)?;

                Ok(None)
            }
            Step::Download { url, destination } => {

                let response = reqwest::blocking::get(url)?;

                let file_name = if destination.path(temp).is_dir() {
                    let fname = response
                        .url()
                        .path_segments()
                        .and_then(|segments| segments.last())
                        .and_then(|name| if name.is_empty() {None} else {Some(name)})
                        .unwrap_or("tmp.bin");

                    destination.path(temp).join(fname)
                } else if destination.path(temp).is_file() {
                    destination.path(temp)
                } else {
                    panic!("Download destination must be a directory or file")
                };

                let mut dest = std::fs::File::create(file_name.clone())?;

                let content = response.text().unwrap();
                std::io::copy(&mut content.as_bytes(), &mut dest)?;

                if destination.is_temp() {
                    Ok(None)
                } else {
                    Ok(Some(Step::Delete { path: file_name }))
                }


            }
            Step::Print { message } => {
                println!("{}", message);

                Ok(None)
            }
        }


    }

}

