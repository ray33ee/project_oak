extern crate lazy_static;

extern crate zip;
extern crate fs_extra;
extern crate serde;
extern crate serde_json;
extern crate zip_extensions;
extern crate clap;
extern crate tempfile;
extern crate registry;

use std::path::{Path};
use oak::{OakRead, OakWrite};
use error::Error;

mod error;
mod steps;
mod command;
mod vm;
mod oak;
mod registry_ex;

use crate::error::Result;
use std::fs::OpenOptions;
use clap::lazy_static::lazy_static;
use stack_vm::{Builder, Code, Machine, WriteManyTable};
use steps::{PathType};
use crate::vm::machine_data::Data;

/*

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

*/

// Really simple language for oak
fn create_installer<P: AsRef<Path>>(_source_file: P, _installer_path: P) -> Result<()> {

    lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new("([\\s]+)data[\\s]+p\"([^\"]+)\"[\\s]*").unwrap();
    }

    let original_source = std::fs::read_to_string(_source_file).unwrap();

    let mut oak_writer = OakWrite::new(_installer_path);

    //Loop over all data commands, add each file (named as an argument to the data command)
    //replace with argument with the name (returned when a file is added to the archive)
    let cow = RE.replace_all(original_source.as_str(), |caps: &regex::Captures| {
        let name = oak_writer.archive(&caps[2]);
        format!("{}data \"{}\"", &caps[1], name) //We use &cap[1] to preserve the original whitespace (including an end of line line break and maintain readability
    });

    oak_writer.commands(cow.as_ref());

    Ok(())

}


fn install<P: AsRef<Path>>(installer: P, uninstaller: P) {

    let table = vm::instructions::get_instruction_table();

    //Run the installation step
    let mut rel = install_step(Data::install(installer, uninstaller));

    let mut inverse_builder  = Builder::new(&table);

    for (s, v) in rel.inverse.as_ref().unwrap() {
        inverse_builder.push(&s, v.clone());
    }


    let inverse_source = format!("{:?}", Code::from(inverse_builder));

    //Save uninstaller code
    let archive = rel.uninstall_archive.as_mut().unwrap();

    archive.commands(&inverse_source);

}

fn uninstall<P: AsRef<Path>>(uninstaller: P) {
    install_step(Data::uninstall( uninstaller));
}

fn install_step(mut data: Data) -> Data {

    let table = vm::instructions::get_instruction_table();

    // Get installer source
    let install_source = &data.install_archive.commands().unwrap();

    let code = Code::parse(&install_source, &table);

    //Setup and run VM
    let constants: WriteManyTable<vm::operand::Operand> = WriteManyTable::new();
    let mut machine = Machine::new(code, &constants, &table, data);
    machine.run();

    machine.release()
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

    println!("{}", read.commands().unwrap());


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

                    install(i, u);
                }
                ("uninstall", matches) => {
                    let u = matches.value_of("uninstaller path").unwrap();

                    uninstall(u);

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
                install("installer", "uninstaller");
            } else if Path::new("uninstaller").exists() {
                uninstall("uninstaller")
            } else {
                panic!("No installer or uninstaller found")
            }

        }
    }


}
