extern crate zip;
extern crate fs_extra;
extern crate serde;
extern crate serde_json;
extern crate zip_extensions;
extern crate clap;
extern crate tempfile;
extern crate registry;

use std::path::{Path, PathBuf};
use oak::{OakRead, OakWrite};
use error::Error;

mod error;
mod steps;
mod command;
mod vm;
mod oak;

use crate::error::Result;
use std::fs::OpenOptions;
use std::io::Read;
use steps::{Step, SpecialPath, FileType};
use command::Command;
use crate::steps::{Data, RootKey};

///Install from the `installer` file, and write the uninstaller to `uninstaller`
fn install<P: AsRef<Path>>(installer: P, uninstaller: P) -> Result<()> {
    let mut oak_read = OakRead::new(installer.as_ref()).unwrap();

    let commands = oak_read.commands().unwrap();

    let mut oak_write = OakWrite::new(uninstaller.as_ref());

    let mut inverses = Vec::with_capacity(commands.len());

    let temp = tempfile::tempdir().unwrap();

    for command in commands {
        match command.action(&temp, &mut oak_read, &mut oak_write) {
            Ok(inv) => {
                if !inv.is_empty() {
                    inverses.push(inv);
                }
            }
            Err(e) => {
                println!("Error: {:?}", &e);
                println!("Command: {:?}", &command);

                //A command has failed. The failed command has itself been reversed (handled in Command::action)
                //But the other commands that succeeded needs reversing too:
                for inv in inverses.iter() {
                    inv.inverse(& mut oak_read);
                }

                //Delete the uninstaller we created
                std::fs::remove_file(uninstaller.as_ref()).unwrap();

                return Err(e);
            }
        }
    }

    oak_write.commands(&inverses);

    Ok(())
}

///Run the `uninstaller` file as an uninstaller
fn uninstall<P: AsRef<Path>>(uninstaller: P) -> Result<()> {

    let mut oak_read = OakRead::new(uninstaller.as_ref()).unwrap();

    let commands = oak_read.commands().unwrap();

    for inv in commands.iter() {
        inv.inverse(& mut oak_read);
    }

    std::fs::remove_file(uninstaller.as_ref()).unwrap();

    Ok(())
}

///List all the files, folders and commands in an oak repo
fn list<P: AsRef<Path>>(repo: P) -> Result<()> {

    {
        let archive = zip::ZipArchive::new(OpenOptions::new().read(true).open(repo.as_ref())?)?;

        println!("Stored files:");

        for name in archive.file_names() {
            if name != "_commands" {
                println!("    {}", name);
            }
        }
    }

    let mut read = OakRead::new(repo.as_ref()).unwrap();

    println!("Commands:");

    println!("{}", serde_json::to_string_pretty(&read.commands().unwrap()).unwrap());


    Ok(())
}

// Really simple language for oak
fn create_installer<P: AsRef<Path>>(source_file: P, installer_path: P) -> Result<()> {

    let contents = {
        let mut file = OpenOptions::new().read(true).open(source_file.as_ref())?;

        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        contents
    };

    let mut writer = OakWrite::new(installer_path);

    let mut tokens = contents.split(',');

    let mut commands = Vec::new();

    let mut command = Vec::new();

    let mut token = tokens.next();


    while token.is_some() {
        match token.unwrap() {
            "data" | "file" => {
                let name = PathBuf::from(tokens.next().unwrap());
                let name = writer.archive(name);
                println!("name: {}", name);
                let destination = SpecialPath::from(tokens.next().unwrap());
                command.push(Step::Data { name, destination });
            }
            "move" => {
                let source = SpecialPath::from(tokens.next().unwrap());

                let destination = PathBuf::from(tokens.next().unwrap());
                command.push(Step::Move { source, destination });
            }
            "delete" => {
                let path = PathBuf::from(tokens.next().unwrap());
                command.push(Step::Delete { path })
            }
            "create" => {
                let f_type = match tokens.next().unwrap() {
                    "file" => {FileType::File}
                    "folder" | "dir" | "directory" => {FileType::Folder}
                    _ => {panic!("Invalid file type (must be 'file' or 'dir'")}
                };
                let path = SpecialPath::from(tokens.next().unwrap());
                command.push(Step::Create { path, f_type });
            }
            "copy" => {
                let source = SpecialPath::from(tokens.next().unwrap());
                let destination = SpecialPath::from(tokens.next().unwrap());
                command.push(Step::Copy { source, destination });
            }
            "download" => {
                let url = String::from(tokens.next().unwrap());
                let destination = SpecialPath::from(tokens.next().unwrap());
                command.push(Step::Download { url, destination })
            }
            "rename" => {
                let from = SpecialPath::from(tokens.next().unwrap());
                let to = SpecialPath::from(tokens.next().unwrap());
                command.push(Step::Rename { from, to });
            }
            "zip" => {
                let folder = SpecialPath::from(tokens.next().unwrap());
                let archive = SpecialPath::from(tokens.next().unwrap());
                command.push(Step::Zip { folder, archive });
            }
            "unzip" => {
                let folder = SpecialPath::from(tokens.next().unwrap());
                let archive = SpecialPath::from(tokens.next().unwrap());
                command.push(Step::Unzip { folder, archive });
            }
            "reg_write_value" => {

                let root = RootKey::from(tokens.next().unwrap());
                let key = String::from(tokens.next().unwrap());
                let value = String::from(tokens.next().unwrap());
                let data = Data::from(tokens.next().unwrap());

                command.push(Step::WriteRegistryValue { root, key, value, data })
            }
            "reg_write_key" => {

                let root = RootKey::from(tokens.next().unwrap());
                let key = String::from(tokens.next().unwrap());
                let new = String::from(tokens.next().unwrap());

                command.push(Step::WriteRegistryKey { root, key, new })
            }
            "end_command" => {
                commands.push(Command(command));
                command = Vec::new();
            }
            _ => {

            }
        }

        token = tokens.next();
    }

    writer.commands(&commands);

    Ok(())

}

fn main() {

    let m = clap::Command::new(clap::crate_name!())
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .subcommands(vec![
            clap::Command::new("install")
                .about("Install and create uninstaller")
                .arg(
                    clap::Arg::new("installer path")
                        .short('i')
                        .long("installer_path")
                        .takes_value(true)
                        .value_name("Installer path")
                        //.validator(|x| {Ok(())})
                        .required(true))
                .arg(
                    clap::Arg::new("uninstaller path")
                        .short('u')
                        .long("uninstaller_path")
                        .takes_value(true)
                        .value_name("Uninstaller path")
                        //.validator(|x| {Ok(())})
                        .required(true)),


            clap::Command::new("create")
                .about("Create an installer from a script")
                .arg(
                    clap::Arg::new("script path")
                        .short('s')
                        .long("script_path")
                        .takes_value(true)
                        .value_name("Script path")
                        //.validator(|x| {Ok(())})
                        .required(true))
                .arg(
                    clap::Arg::new("installer path")
                        .short('i')
                        .long("installer_path")
                        .takes_value(true)
                        .value_name("Installer path")
                        //.validator(|x| {Ok(())})
                        .required(true)),

            clap::Command::new("uninstall")
                .about("Install and create uninstaller")
                .arg(
                    clap::Arg::new("uninstaller path")
                        .short('u')
                        .long("uninstaller_path")
                        .takes_value(true)
                        .value_name("Uninstaller path")
                        //.validator(|x| {Ok(())})
                        .required(true)),
            clap::Command::new("list")
                .about("List the files, folders and commands associated with an oak archive")
                .arg(
                    clap::Arg::new("path")
                        .short('p')
                        .long("path")
                        .takes_value(true)
                        .value_name("Path")
                        //.validator(|x| {Ok(())})
                        .required(true))
        ]).get_matches();

    match m.subcommand() {
        Some(command) => {
            match command {
                ("install", matches) => {
                    let i = matches.value_of("installer path").unwrap();
                    let u = matches.value_of("uninstaller path").unwrap();

                    install(i, u).unwrap();
                }
                ("uninstall", matches) => {
                    let u = matches.value_of("uninstaller path").unwrap();

                    uninstall(u).unwrap();

                }
                ("list", matches) => {
                    let p = matches.value_of("path").unwrap();

                    list(p).unwrap();

                }
                ("create", matches) => {
                    let s = matches.value_of("script path").unwrap();
                    let i = matches.value_of("installer path").unwrap();

                    create_installer(s, i).unwrap();
                    list(i).unwrap();

                }
                _ => {}
            }
        }
        None => {
            //If no arguments are supplied, look to ".\installer" and ".\uninstaller" and run them

            if Path::new("installer").exists() {
                install("installer", "uninstaller").unwrap();
            } else if Path::new("uninstaller").exists() {
                uninstall("uninstaller").unwrap()
            } else {
                panic!("No installer or uninstaller found")
            }

        }
    }


}
