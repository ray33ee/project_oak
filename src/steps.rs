
use crate::error::{Result, Error};
use std::path::{PathBuf};
use std::fs::OpenOptions;
use serde::{Serialize, Deserialize};
use crate::oak::OakRead;
use crate::oak::OakWrite;
use tempfile::TempDir;

#[derive(Debug, Serialize, Deserialize)]
pub enum FileType {
    File,
    Folder,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SpecialPath {
    Path(PathBuf),
    TemporaryFolder,
}

///A step is the smallest unit of an installation process
#[derive(Debug, Serialize, Deserialize)]
pub enum Step {
    Data{ name: String, destination: PathBuf },
    Move{ source: SpecialPath, destination: PathBuf },
    Delete { path: PathBuf },
    Create { path: PathBuf, f_type: FileType },
    Copy { source: SpecialPath, destination: PathBuf },
    Rename { from: PathBuf, to: PathBuf },
    //Zip { source: SpecialPath, destination: SpecialPath },
    //Environment,
    //Regedit,
    //Download { url: String, destination: SpecialPath },
    //Install - Installer path, plus a reg entry to check it installed correctly, locate the uninstaller,
    //Uninstall - Uninstaller path, plus a reg entry to check it installed correctly,
    //Edit,
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
                install_archive.extract(&name, &destination)?;

                Ok(Some(Step::Delete { path: destination.clone() }))
            }
            Step::Move { source, destination } => {

                let source_path = match source {
                    SpecialPath::Path(s) => {s.as_path()}
                    SpecialPath::TemporaryFolder => {temp.unwrap().path()}
                };

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
                        SpecialPath::TemporaryFolder => {

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

                if let Some(archive) = uninstall_archive {
                    Ok(Some(Step::Data { name: name.unwrap(), destination: path.clone() }))
                } else {
                    Ok(None)
                }

            }
            Step::Create { path, f_type } => {
                match f_type {
                    FileType::File => {
                        OpenOptions::new().create_new(true).write(true).open(&path)?;

                        Ok(Some(Step::Delete { path: path.clone() }))
                    }
                    FileType::Folder => {
                        std::fs::create_dir(path)?;

                        Ok(Some(Step::Delete { path: path.clone() }))
                    }
                }
            }
            Step::Copy { source, destination } => {

                let source_path = match source {
                    SpecialPath::Path(s) => {s.as_path()}
                    SpecialPath::TemporaryFolder => {temp.unwrap().path()}
                };

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
                std::fs::rename(&from, &to)?;

                Ok(Some(Step::Rename {from: to.clone(), to: from.clone()}))
            }
            Step::Print { message } => {
                println!("{}", message);

                Ok(None)
            }
        }


    }

}

