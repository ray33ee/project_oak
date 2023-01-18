extern crate lazy_static;

extern crate zip;
extern crate fs_extra;
extern crate zip_extensions;
extern crate clap;
extern crate tempfile;
extern crate registry;
extern crate core;

use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use clap::Arg;
use tempfile::TempDir;
use oak::{OakRead, OakWrite};
use crate::oak::{Info, UninstallLocation};
use crate::source::Source;

mod error;
mod oak;
mod registry_ex;
mod hlc;
mod tests;
mod mlc;
mod functions;
mod path_type;
mod extra_functions;
mod higher_functions;
mod source;

pub fn append_archive_to_exe(archive_path: &Path, new_exe: &Path) {

    let exe_path = std::env::current_exe().unwrap();


    let length = OpenOptions::new().read(true).open(&exe_path).unwrap().metadata().unwrap().len();

    std::fs::copy(&exe_path, new_exe).unwrap();

    let mut exe = std::fs::OpenOptions::new().write(true).append(true).open(new_exe).unwrap();

    let mut archive = OpenOptions::new().read(true).open(archive_path).unwrap();

    std::io::copy(& mut archive, & mut exe).unwrap();

    exe.write_all(length.to_be_bytes().as_ref()).unwrap();
}

fn main() {


    let m = clap::Command::new(clap::crate_name!())
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .arg(Arg::new("source file")
            .short('s')
            .long("source")
            .value_name("Source path")
        )
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
                /*("create", matches) => {
                    let s = matches.value_of("script path").unwrap();
                    let i = matches.value_of("installer path").unwrap();

                    hlc::create_installer(Path::new(s), Path::new(i), UninstallLocation::CommandLine).unwrap();
                    hlc::list(i).unwrap();

                }*/
                _ => {}
            }
        }
        None => {
            //If no arguments are supplied, look to ".\installer" and ".\uninstaller" and run them

            /*if Path::new("installer").exists() {
                hlc::install("installer", "uninstaller");
            } else if Path::new("uninstaller").exists() {
                hlc::uninstall("uninstaller");
            } else {
                panic!("No installer or uninstaller found")
            }*/

            let exe_path = std::env::current_exe().unwrap();

            let (offset, length) = {
                let mut fh = OpenOptions::new().read(true).open(&exe_path).unwrap();

                let length = fh.metadata().unwrap().len();

                fh.seek(SeekFrom::End(-8)).unwrap();

                let mut v = [0u8; 8];

                fh.read_exact(& mut v).unwrap();

                (u64::from_be_bytes(v), length)
            };

            if offset == 0 {
                //If the offset is zero, the exe contains no archive. This means it can only be used in 'create_installer' mode

                let source =  m.value_of("source file").unwrap();

                let tmp = TempDir::new().unwrap();

                let tmp_file = tmp.path().join("install");

                let complete = Source::load_from_path(PathBuf::from(source).as_path());

                complete.create_installer(tmp_file.as_path());


                append_archive_to_exe(tmp_file.as_path(), PathBuf::from(".\\install.exe").as_path());



            } else {
                //If the offset is non-zero, we have an archive appended. We use this offset to obtain the archive and run it

                let tmp = TempDir::new().unwrap();

                let tmp_file = tmp.path().join("archive");

                let mut exe = std::fs::OpenOptions::new().read(true).open(exe_path.as_path()).unwrap();
                exe.seek(SeekFrom::Start(offset)).unwrap();
                let mut t = exe.take(length - offset - 8);

                {let mut archive = OpenOptions::new().create(true).write(true).open(tmp_file.as_path()).unwrap();

                std::io::copy(& mut t, & mut archive).unwrap();}

                hlc::execute(tmp_file.as_path());

            }

        }
    }


}
