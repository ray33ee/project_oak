extern crate lazy_static;

extern crate zip;
extern crate fs_extra;
extern crate zip_extensions;
extern crate clap;
extern crate tempfile;
extern crate registry;
extern crate core;

use std::path::{PathBuf};
use clap::Arg;
use tempfile::TempDir;
use oak::{OakRead, OakWrite};
use crate::oak::{Info};
use crate::source::Source;

mod error;
mod oak;
//mod registry_ex;
mod hlc;
mod tests;
mod mlc;
mod path_type;
mod source;
mod exe_extender;


fn main() {

    let m = clap::Command::new(clap::crate_name!())
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .arg(Arg::new("source file")
            .short('s')
            .long("source")
            .value_name("Source path")
        ).get_matches();

        let (offset, length) = exe_extender::get_meta();

        if offset == 0 {
            //If the offset is zero, the exe contains no archive. This means it can only be used in 'create_installer' mode

            let source =  m.value_of("source file").unwrap();

            let tmp = TempDir::new().unwrap();

            let tmp_file = tmp.path().join("install");

            let complete = Source::load_from_path(PathBuf::from(source).as_path());

            complete.create_installer(tmp_file.as_path());

            exe_extender::extend_exe(tmp_file.as_path(), PathBuf::from(".\\install.exe").as_path(), length);



        } else {
            //If the offset is non-zero, we have an archive appended. We use this offset to obtain the archive and run it

            let tmp = TempDir::new().unwrap();

            let tmp_file = tmp.path().join("archive");

            exe_extender::get_archive(tmp_file.as_path(), length, offset);

            hlc::execute(tmp_file.as_path());

        }





}
