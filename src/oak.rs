
use zip::{ZipArchive, ZipWriter};
use std::path::Path;
use std::iter::Zip;
use std::fs::OpenOptions;
use crate::command::Command;
use std::borrow::BorrowMut;
use zip::write::FileOptions;

pub struct OakRead {
    archive: ZipArchive<std::fs::File>,
}

impl OakRead {

    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            archive: ZipArchive::new(OpenOptions::new().read(true).open(path).unwrap()).unwrap(),
        }
    }

    pub fn commands(& mut self) -> Vec<Command> {
        //bincode::deserialize_from(self.archive.by_name("_command").unwrap()).unwrap()
        serde_json::from_reader(self.archive.by_name("_command").unwrap()).unwrap()
    }

    pub fn extract<P: AsRef<Path>>(& mut self, name: &str, destination: P) {
        let mut afile = self.archive.by_name(name).unwrap();
        let mut dfile = OpenOptions::new().write(true).create(true).open(destination).unwrap();
        std::io::copy(& mut afile, & mut dfile).unwrap();
    }
}

pub struct OakWrite {
    archive: ZipWriter<std::fs::File>,
    count: u32,
}

impl OakWrite {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            archive: ZipWriter::new(OpenOptions::new().create_new(true).write(true).open(path.as_ref()).unwrap()),
            count: 0,
        }
    }

    pub fn count(&self) -> u32 { self.count }

    pub fn archive<P: AsRef<Path>>(& mut self, path: P) {

        if path.as_ref().is_dir() {
            todo!()
            //self.archive.add_directory(path, FileOptions::default());


        } else if path.as_ref().is_file() {
            self.archive.start_file(format!("_{}", self.count), FileOptions::default());
            let mut file  = OpenOptions::new().read(true).open(path.as_ref()).unwrap();
            std::io::copy(& mut file, & mut self.archive).unwrap();
        } else {
            panic!("{:?} is not a file or folder", path.as_ref());
        }

        self.count = self.count + 1;

    }

    pub fn commands(& mut self, commands: & Vec<Command>) {
        self.archive.start_file(format!("_commands"), FileOptions::default()).unwrap();
        //bincode::serialize_into(& mut self.archive, commands).unwrap();
        serde_json::to_writer(& mut self.archive, commands).unwrap()
    }

    pub fn finish(& mut self) {
        self.archive.finish();
    }

}
