
use crate::error::{Result, Error};
use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use serde::{Serialize, Deserialize};
use crate::oak::OakRead;
use crate::oak::OakWrite;
use tempfile::TempDir;

#[derive(Debug, Serialize, Deserialize)]
pub enum FileType {
    File,
    Folder,
    Shortcut(PathBuf),
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

    pub fn is_perm(&self) -> bool {
        !self.is_temp()
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
    Copy { source: SpecialPath, destination: PathBuf },
    Rename { from: SpecialPath, to: SpecialPath },
    //Zip { source: SpecialPath, destination: SpecialPath }, - Take a folder and zip its contents
    //Unzip { source: SpecialPath, destination: SpecialPath }, - Take a zip archive and extract it to a location
    //Environment,
    //Regedit,
    Download { url: String, destination: SpecialPath },
    //Install - Installer path, plus a reg entry to check it installed correctly, locate the uninstaller,
    //Uninstall - Uninstaller path, plus a reg entry to check it installed correctly,
    //Edit{ source: SpecialPath }, - Edit a text file, be sure to make this as comprehensive as possible
    //If,
    Print { message: String },
    //String,
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

                if destination.exists() {
                    Err(crate::error::Error::AlreadyExists)
                } else {
                    if source_path.is_file() {

                        std::fs::copy(&source_path, &destination)?;
                    } else if source_path.is_dir() {
                        let mut options = fs_extra::dir::CopyOptions::default();
                        options.content_only = true;

                        fs_extra::dir::copy(&source_path, &destination, &options)?;
                    } else {
                        panic!("Source is not a file or directory");
                    }

                    Ok(Some(Step::Delete { path: destination.clone() }))
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

