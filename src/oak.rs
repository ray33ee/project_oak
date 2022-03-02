
use zip::{ZipArchive, ZipWriter};
use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use crate::command::Command;
use zip::write::FileOptions;
use crate::error::{Result};
use zip_extensions::{ZipWriterExtensions};
use std::io::{Seek, SeekFrom};

///A struct used to read an oak archive
pub struct OakRead {
    archive: ZipArchive<std::fs::File>,
}

impl OakRead {

    ///Create a new reader from an existing oak archive
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            archive: ZipArchive::new(OpenOptions::new().read(true).open(path).unwrap())?,
        })
    }

    ///Get the list of commands stored in the archive
    pub fn commands(& mut self) -> Result<Vec<Command>> {
        //bincode::deserialize_from(self.archive.by_name("_command").unwrap()).unwrap()
        Ok(serde_json::from_reader(self.archive.by_name("_commands")?)?)
    }

    ///Extract the specified file `name` to `destination`
    pub fn extract<P: AsRef<Path>>(& mut self, name: &str, destination: P) -> Result<()> {


        let mut afile = self.archive.by_name(name)?;

        if name.as_bytes()[1] == 'd' as u8 {

            let mut temp = tempfile::tempfile().unwrap();

            std::io::copy(& mut afile, & mut temp)?;

            temp.seek(SeekFrom::Start(0)).unwrap();

            let mut archive = zip::ZipArchive::new(temp).unwrap();

            std::fs::create_dir(destination.as_ref())?;

            archive.extract(destination)?;

            Ok(())
        } else {
            let mut dfile = OpenOptions::new().write(true).create(true).open(destination)?;
            std::io::copy(& mut afile, & mut dfile)?;

            Ok(())
        }


    }
}

///A struct used to write to an oak archive
pub struct OakWrite {
    archive: ZipWriter<std::fs::File>,
    count: u32,
}

impl OakWrite {
    ///Create a new oak archive and return an `OakWrite` object
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            archive: ZipWriter::new(OpenOptions::new().create_new(true).write(true).open(path.as_ref()).unwrap()),
            count: 0,
        }
    }

    ///Returns the current count
    ///
    /// Each write to the archive, the count variable is incremented. Used to uniquely name files in an oak archive
    pub fn count(&self) -> u32 { self.count }

    ///Archive a file or folder into the archive
    pub fn archive<P: AsRef<Path>>(& mut self, path: P) {

        if path.as_ref().is_dir() {
            //self.archive.add_directory(path.as_ref()., FileOptions::default());

            let mut temp = tempfile::tempfile().unwrap();

            //let mut temp = std::fs::OpenOptions::new().read(true).write(true).create(true).open("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\create.zip").unwrap();

            {
                let mut writer = ZipWriter::new(&temp);

                writer.create_from_directory(&PathBuf::from(path.as_ref())).unwrap();

            }

            self.archive.start_file(format!("_d_{}", self.count), FileOptions::default()).unwrap();

            temp.seek(SeekFrom::Start(0)).unwrap();

            std::io::copy(& mut temp, & mut self.archive).unwrap();

        } else if path.as_ref().is_file() {
            self.archive.start_file(format!("_{}", self.count), FileOptions::default()).unwrap();
            let mut file  = OpenOptions::new().read(true).open(path.as_ref()).unwrap();
            std::io::copy(& mut file, & mut self.archive).unwrap();
        } else {
            panic!("{:?} is not a file or folder", path.as_ref());
        }

        self.count = self.count + 1;

    }

    ///Write the commands list to the archive
    pub fn commands(& mut self, commands: & Vec<Command>) {
        self.archive.start_file(format!("_commands"), FileOptions::default()).unwrap();
        //bincode::serialize_into(& mut self.archive, commands).unwrap();
        serde_json::to_writer(& mut self.archive, commands).unwrap()
    }

    /*
    ///Finish the archive. This is called on `drop`
    pub fn finish(& mut self) -> Result<()> {
        self.archive.finish()?;
        Ok(())
    }
    */

}
