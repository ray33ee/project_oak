extern crate zip;
extern crate fs_extra;
extern crate serde;
extern crate serde_json;
extern crate zip_extensions;
extern crate clap;
extern crate tempfile;

use std::path::{Path};
use oak::{OakRead, OakWrite};
use error::Error;


mod error;
mod steps;
mod command;
mod vm;
mod oak;

use crate::error::Result;
use std::fs::OpenOptions;

///Install from the `installer` file, and write the uninstaller to `uninstaller`
fn install<P: AsRef<Path>>(installer: P, uninstaller: P) -> Result<()> {
    let mut oak_read = OakRead::new(installer.as_ref()).unwrap();

    let commands = oak_read.commands().unwrap();

    let mut oak_write = OakWrite::new(uninstaller.as_ref());

    let mut inverses = Vec::with_capacity(commands.len());

    let mut temp = tempfile::tempdir().unwrap();

    for command in commands {
        match command.action(&temp, &mut oak_read, &mut oak_write) {
            Ok(inv) => {
                if !inv.is_empty() {
                    inverses.push(inv);
                }
            }
            Err(e) => {
                println!("Error: {:?}", &e);

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

    match m.subcommand().unwrap() {
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
        _ => {}
    }

}
