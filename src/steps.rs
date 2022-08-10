
use crate::error::{Result, Error};
use std::path::{PathBuf};
use std::fs::OpenOptions;
use serde::{Serialize, Deserialize};
use crate::oak::OakRead;
use crate::oak::OakWrite;
use tempfile::TempDir;
use registry::Security;

#[derive(Debug, Serialize, Deserialize)]
pub enum FileType {
    File,
    Folder,
    Shortcut(PathBuf),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Data {
    None,
    String(String),
    ExpandString(String),
    Binary(Vec<u8>),
    U32(u32),
    U32BE(u32),
    Link,
    MultiString(Vec<String>),
    ResourceList,
    FullResourceDescriptor,
    ResourceRequirementsList,
    U64(u64),
}

impl From<&str> for Data {
    fn from(s: &str) -> Self {
        match s.parse::<u32>() {
            Ok(v) => {Data::U32(v)}
            Err(_) => {Data::String(String::from(s))}
        }
    }
}

impl From<registry::Data> for Data {
    fn from(d: registry::Data) -> Self {
        match d {
            registry::Data::None => {Data::None}
            registry::Data::String(z) => {Data::String(z.to_string_lossy())}
            registry::Data::ExpandString(z) => {Data::ExpandString(z.to_string_lossy())}
            registry::Data::Binary(z) => {Data::Binary(z)}
            registry::Data::U32(z) => {Data::U32(z)}
            registry::Data::U32BE(z) => {Data::U32BE(z)}
            registry::Data::Link => {Data::Link}
            registry::Data::MultiString(z) => {Data::MultiString(z.iter().map(|x| x.to_string_lossy()).collect())}
            registry::Data::ResourceList => {Data::ResourceList}
            registry::Data::FullResourceDescriptor => {Data::FullResourceDescriptor}
            registry::Data::ResourceRequirementsList => {Data::ResourceRequirementsList}
            registry::Data::U64(z) => {Data::U64(z)}
        }
    }
}

impl From<&Data> for registry::Data {
    fn from(d: &Data) -> Self {
        match d {
            Data::None => {registry::Data::None}
            Data::String(z) => {registry::Data::String(utfx::U16CString::try_from(z).unwrap())}
            Data::ExpandString(z) => {registry::Data::ExpandString(utfx::U16CString::try_from(z).unwrap())}
            Data::Binary(z) => {registry::Data::Binary(z.clone())}
            Data::U32(z) => {registry::Data::U32(z.clone())}
            Data::U32BE(z) => {registry::Data::U32BE(z.clone())}
            Data::Link => {registry::Data::Link}
            Data::MultiString(z) => {registry::Data::MultiString(z.iter().map(|s| utfx::U16CString::try_from(s).unwrap()).collect())}
            Data::ResourceList => {registry::Data::ResourceList}
            Data::FullResourceDescriptor => {registry::Data::FullResourceDescriptor}
            Data::ResourceRequirementsList => {registry::Data::ResourceRequirementsList}
            Data::U64(z) => {registry::Data::U64(z.clone()) }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RootKey {
    HKLM,
    HKCC,
    HKCR,
    HKCU,
    HKU
}

impl From<&str> for RootKey {
    fn from(s: &str) -> Self {
        match s {
            "hklm" => RootKey::HKLM,
            "hkcc" => RootKey::HKCC,
            "hkcr" => RootKey::HKCR,
            "hkcu" => RootKey::HKCU,
            "hku" => RootKey::HKU,
            _ => panic!("Invalid registry root key")
        }
    }
}

impl From<&RootKey> for registry::Hive {
    fn from(rk: &RootKey) -> Self {
        match rk {
            RootKey::HKLM => {registry::Hive::LocalMachine}
            RootKey::HKCC => {registry::Hive::CurrentConfig}
            RootKey::HKCR => {registry::Hive::ClassesRoot}
            RootKey::HKCU => {registry::Hive::CurrentUser}
            RootKey::HKU => {registry::Hive::Users}
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpecialPath {
    Path(PathBuf),
    TemporaryFolder(PathBuf),
}

impl SpecialPath {
    pub fn path(&self, temp: Option<&TempDir>) -> PathBuf {
        match self {
            SpecialPath::Path(path) => {
                path.clone()
            }
            SpecialPath::TemporaryFolder(path) => {
                temp.unwrap().path().join(path)
            }
        }
    }

    pub fn is_temp(&self) -> bool {
        if let SpecialPath::Path(_) = self {
            false
        } else {
            true
        }
    }
}

impl From<&str>  for  SpecialPath {
    fn from(string: &str) -> Self {
        let key = "temp";

        if string.starts_with(key) {
            SpecialPath::TemporaryFolder(PathBuf::from(&string[key.len()..]).strip_prefix("\\").unwrap().to_path_buf())
        } else {
            SpecialPath::Path(PathBuf::from(string))
        }
    }
}

///A step is the smallest unit of an installation process
#[derive(Debug, Serialize, Deserialize)]
pub enum Step {
    Data{ name: String, destination: SpecialPath },
    Move{ source: SpecialPath, destination: PathBuf },
    Delete { path: PathBuf },
    Create { path: SpecialPath, f_type: FileType },
    Copy { source: SpecialPath, destination: SpecialPath },
    Rename { from: SpecialPath, to: SpecialPath },
    Zip { folder: SpecialPath, archive: SpecialPath },
    Unzip { folder: SpecialPath, archive: SpecialPath },
    //Regedit,
    DeleteRegistryEntry { root: RootKey, key: String, value: Option<String> },
    WriteRegistryValue { root: RootKey, key: String, value: String, data: Data },
    WriteRegistryKey { root: RootKey, key: String, new: String },
    Download { url: String, destination: SpecialPath },
    //Edit{ source: SpecialPath }, //- Edit a text file using something like sed or awk. What should its inverse be? Editing the line out, or restoring the original file?
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
                        SpecialPath::Path(s) => {
                            Ok(Some(Step::Move { source: SpecialPath::Path(destination.clone()), destination: s.clone() }))
                        }
                        SpecialPath::TemporaryFolder(_) => {

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
                    Ok(Some(Step::Data { name: name.unwrap(), destination: SpecialPath::Path(path.clone()) }))
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
                    Ok(Some(Step::Delete { path: archive.path(temp) }))
                }

            }
            Step::DeleteRegistryEntry { root, key, value } => {

                let reg = registry::Hive::from(root).open(key, Security::AllAccess)?;


                match value {
                    None => {
                        reg.delete("", false)?;
                    }
                    Some(val) => {
                        reg.delete_value(val)?;
                    }
                }


                Ok(None)
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
            Step::WriteRegistryKey { root, key, new } => {

                let reg = registry::Hive::from(root).open(key, Security::AllAccess)?;

                reg.create(new, Security::AllAccess)?;

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

