use crate::error::Result;
use std::fs::OpenOptions;
use clap::lazy_static::lazy_static;
use std::path::{Path};
use crate::oak::{OakRead, OakWrite};
use crate::path_type::Inverse;


// Really simple language for oak
pub fn create_installer<P: AsRef<Path>>(_source_file: P, _installer_path: P) -> Result<()> {

    lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new("([\\s]+)data[\\s]+p\"([^\"]+)\"[\\s]*").unwrap();
    }

    let original_source = std::fs::read_to_string(_source_file).unwrap();

    let mut oak_writer = OakWrite::new(_installer_path.as_ref());

    //Loop over all data commands, add each file (named as an argument to the data command)
    //replace with argument with the name (returned when a file is added to the archive)
    /*let cow = RE.replace_all(original_source.as_str(), |caps: &regex::Captures| {
        let name = oak_writer.archive(&caps[2]);
        format!("{}data \"{}\"", &caps[1], name) //We use &cap[1] to preserve the original whitespace (including an end of line line break and maintain readability
    });*/


    oak_writer.commands(original_source.as_str());



    Ok(())

}

fn _install<P: AsRef<Path>>(installer: P, uninstaller: Option<P>) {

    let failed = {
        //Open installer
        let mut read = OakRead::new(installer).unwrap();

        //Get code
        let code = read.commands().unwrap();

        //Open uninstaller
        let mut write = uninstaller.as_ref().map(|u| OakWrite::new(u));

        let temp = tempfile::TempDir::new().unwrap();

        let mut inverses = match &uninstaller {
            None => {None}
            Some(_) => {Some(Inverse::new())}
        };

        let res = crate::mlc::run(code.as_str(), & mut read,  write.as_ref(),  inverses.as_ref(), &temp);

        match res {
            Ok(_) => {

                if let Some(writer) = & mut write {
                    let st = inverses.unwrap().combine();

                    println!("st: {}", st);

                    writer.commands(st.as_str())
                }

                false
            }
            Err(_) => {
                true
            }
        }
    };

    if failed {

        if let Some(u) = uninstaller {
            _install(u, None);
        }

    }


}

pub fn install<P: AsRef<Path>>(installer: P, uninstaller: P) {
    _install(installer, Some(uninstaller));
}

pub fn uninstall<P: AsRef<Path>>(uninstaller: P) {

    _install(uninstaller, None);
}

///List all the files, folders and commands in an oak repo
pub fn list<P: AsRef<Path>>(repo: P) -> Result<()> {

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
