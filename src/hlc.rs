use crate::error::Result;
use std::fs::OpenOptions;
use clap::lazy_static::lazy_static;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use crate::exe_extender::{extend_exe, get_meta};
use crate::oak::{Info, OakRead, OakWrite, OakType, UninstallLocation};
use crate::path_type::Inverse;


pub fn execute<P: AsRef<Path>>(archive: P) -> bool {

    //Open the archive
    let info = {
        let a = OakRead::new(&archive).unwrap();

        a.info().unwrap()
    };

    let tmpdir = TempDir::new().unwrap();
    let tmp_un = tmpdir.path().join("uninstaller");

    let uninstaller = {
        match info.u_location {
            UninstallLocation::Path(path) => {
                Some(path)
            }
            UninstallLocation::InstallationDirectory => {
                todo!()
            }
            UninstallLocation::Null => {
                None
            }
        }
    };

    //Get the OakType field of the _info data
    match info.oak_type {
        OakType::Installer => {
            let result = _install(archive, Some(tmp_un.as_path()));

            if !result {
                let (_, length) = get_meta();

                extend_exe(tmp_un.as_path(), uninstaller.unwrap().as_path(), length);
            }

            result
        }
        OakType::Uninstaller => {
            _install::<P, PathBuf>(archive, None)
        }
    }



}

// Really simple language for oak
pub fn create_installer(_source: &str, _installer_path: &Path, info: &Info) -> Result<()> {

    lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new("([\\s]+)data[\\s]+p\"([^\"]+)\"[\\s]*").unwrap();
    }

    let original_source = _source;

    let oak_writer = OakWrite::new(_installer_path);

    oak_writer.info(info);

    //Loop over all data commands, add each file (named as an argument to the data command)
    //replace with argument with the name (returned when a file is added to the archive)
    /*let cow = RE.replace_all(original_source.as_str(), |caps: &regex::Captures| {
        let name = oak_writer.archive(&caps[2]);
        format!("{}data \"{}\"", &caps[1], name) //We use &cap[1] to preserve the original whitespace (including an end of line line break and maintain readability
    });*/

    let mut quotes = false;

    let mut source = String::new();

    let mut file = None;

    for (i, c) in original_source.as_bytes().iter().enumerate() {
        if *c == '\"' as u8 {
            quotes = !quotes;
        } else if *c == '$' as u8 && !quotes {

            if file.is_none() {
                file = Some(i+1);
            } else {
                let start = file.unwrap();
                let finish = i;


                let path = PathBuf::from(&original_source[start+1..finish-1]);

                let name = oak_writer.archive(path);

                source.push_str("\"");
                source.push_str(name.as_str());
                source.push_str("\"");

                file = None;
            }
            continue;
        }

        if file.is_none() {
            source.push(*c as char);
        }
    }

    oak_writer.commands(source.as_str());



    Ok(())

}

fn _install<P: AsRef<Path>, Q: AsRef<Path>>(installer: P, uninstaller: Option<Q>) -> bool {

    let failed = {
        //Open installer
        let mut read = OakRead::new(installer).unwrap();

        //Get code
        let code = read.commands().unwrap();

        //Open uninstaller
        let mut write = uninstaller.as_ref().map(|u| OakWrite::new(u));


        let temp = tempfile::TempDir::new().unwrap();

        let inverses = match &uninstaller {
            None => {None}
            Some(_) => {Some(Inverse::new())}
        };


        let res = crate::mlc::run(code.as_str(), & mut read, write.as_ref(), inverses.as_ref(), &temp);

        if let Some(writer) = & mut write {
            let st = inverses.unwrap().combine();

            writer.commands(st.as_str());

            writer.info( &Info::default().set_type(OakType::Uninstaller).set_uninstaller_location(UninstallLocation::Null) );
        }


        res.is_err()
    };

    if failed {

        if let Some(u) = uninstaller {
            _install(u.as_ref(), None::<PathBuf>);


            std::fs::remove_file(u).unwrap();
        }

    }

    failed
}

pub fn install<P: AsRef<Path>>(installer: P, uninstaller: P) -> bool {
    _install(installer, Some(uninstaller))
}

pub fn uninstall<P: AsRef<Path>>(uninstaller: P) -> bool {

    _install(uninstaller, None::<PathBuf>)
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

    let read = OakRead::new(repo.as_ref()).unwrap();

    println!("Commands:");

    println!("{}", read.commands().unwrap());


    Ok(())
}
