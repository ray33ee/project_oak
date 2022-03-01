
use crate::error::{Result, Error};
use std::path::{PathBuf};
use crate::vm::VM;
use zip::write::ZipWriter;
use zip::read::ZipArchive;
use std::fs::OpenOptions;
use serde::{Serialize, Deserialize};
use crate::oak::OakRead;
use crate::oak::OakWrite;

#[derive(Debug, Serialize, Deserialize)]
pub enum FileType {
    File,
    Folder,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Step {
    File{ name: String, destination: PathBuf },
    Move{ source: PathBuf, destination: PathBuf },
    Delete { path: PathBuf },
    Create { path: PathBuf, f_type: FileType },
    Copy { source: PathBuf, destination: PathBuf },
    Rename { from: PathBuf, to: PathBuf },
    //Environment,
    //Regedit,
    //Download { url: String, destination: PathBuf },
    Execute { command: String, args: Vec<String> },
    //Edit,
    //If,
    Print { message: String },
    //Input,
    //String,
}

impl Step {

    ///Execute the action, and return the default inverse action if required
    pub fn action(
        & self,
        //vm: & mut VM,
        install_archive: & mut OakRead,
        mut uninstall_archive: Option<& mut OakWrite>,
        //default: bool
    ) -> Result<Option<Step>> {

        match &self {
            Step::File { name, destination } => {
                install_archive.extract(&name, &destination);

                Ok(Some(Step::Delete { path: destination.clone() }))
            }
            Step::Move { source, destination } => {


                if source.is_dir() {
                    let mut options = fs_extra::dir::CopyOptions::default();
                    options.content_only = true;

                    fs_extra::dir::move_dir(&source, &destination, &options).unwrap();

                } else if source.is_file() {
                    let options = fs_extra::file::CopyOptions::default();

                    fs_extra::file::move_file(&source, &destination, &options).unwrap();

                } else {
                    panic!("File is not a path or file");
                }

                Ok(Some(Step::Move { source: destination.clone(), destination: source.clone() }))
            }
            Step::Delete { path } => {

                if let Some( archive) = uninstall_archive.as_mut() {
                    archive.archive(&path);
                }

                if path.is_dir() {
                    std::fs::remove_dir_all(&path).unwrap();
                } else if path.is_file() {
                    std::fs::remove_file(&path).unwrap();
                } else {
                    panic!("File is not a path or file");
                }

                if let Some(archive) = uninstall_archive {
                    Ok(Some(Step::File { name: format!("_{}", archive.count()-1), destination: path.clone() }))
                } else {
                    Ok(None)
                }

            }
            Step::Create { path, f_type } => {
                match f_type {
                    FileType::File => {
                        OpenOptions::new().create_new(true).write(true).open(&path).unwrap();

                        Ok(Some(Step::Delete { path: path.clone() }))
                    }
                    FileType::Folder => {
                        std::fs::create_dir(path).unwrap();

                        Ok(Some(Step::Delete { path: path.clone() }))
                    }
                }
            }
            Step::Copy { source, destination } => {

                if source.is_file() {

                    std::fs::copy(&source, &destination).unwrap();
                } else if source.is_dir() {
                    let mut options = fs_extra::dir::CopyOptions::default();
                    options.content_only = true;

                    fs_extra::dir::copy(&source, &destination, &options).unwrap();
                } else {
                    panic!("Source is not a file or directory");
                }

                Ok(Some(Step::Delete { path: destination.clone() }))
            }
            Step::Rename { from, to } => {
                std::fs::rename(&from, &to).unwrap();

                Ok(Some(Step::Rename {from: to.clone(), to: from.clone()}))
            }
            Step::Execute { command, args } => {
                std::process::Command::new(command)
                    .args(args)
                    .output()
                    .unwrap();

                Ok(None)
            }
            Step::Print { message } => {
                println!("{}", message);

                Ok(None)
            }
        }


    }

}

