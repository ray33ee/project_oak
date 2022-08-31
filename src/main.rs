extern crate lazy_static;

extern crate zip;
extern crate fs_extra;
extern crate zip_extensions;
extern crate clap;
extern crate tempfile;
extern crate registry;
extern crate core;

use std::path::{Path};
use oak::{OakRead, OakWrite};

mod error;
mod oak;
mod registry_ex;
mod hlc;
mod tests;
mod mlc;
mod functions;
mod path_type;
mod extra_functions;

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

                    crate::hlc::install(i, u);

                }
                ("uninstall", matches) => {
                    let u = matches.value_of("uninstaller path").unwrap();

                    hlc::uninstall(u);

                }
                ("list", matches) => {
                    let p = matches.value_of("path").unwrap();

                    hlc::list(p).unwrap();

                }
                ("create", matches) => {
                    let s = matches.value_of("script path").unwrap();
                    let i = matches.value_of("installer path").unwrap();

                    hlc::create_installer(s, i).unwrap();
                    hlc::list(i).unwrap();

                }
                _ => {}
            }
        }
        None => {
            //If no arguments are supplied, look to ".\installer" and ".\uninstaller" and run them

            if Path::new("installer").exists() {
                hlc::install("installer", "uninstaller");
            } else if Path::new("uninstaller").exists() {
                hlc::uninstall("uninstaller");
            } else {
                panic!("No installer or uninstaller found")
            }

        }
    }


}
