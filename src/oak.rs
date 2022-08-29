
use zip::{ZipArchive, ZipWriter};
use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use zip::write::FileOptions;
use crate::error::{Result};
use zip_extensions::{ZipWriterExtensions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::{DerefMut};
use std::sync::Mutex;

///A struct used to read an oak archive
pub struct OakRead {
    archive: Mutex<ZipArchive<std::fs::File>>,
}

impl OakRead {

    ///Create a new reader from an existing oak archive
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            archive: Mutex::new(ZipArchive::new(OpenOptions::new().read(true).open(path).unwrap())?),
        })
    }

    ///Get the list of commands stored in the archive
    pub fn commands(& self) -> Result<String> {
        //bincode::deserialize_from(self.archive.by_name("_command").unwrap()).unwrap()
        let mut guard = self.archive.lock().unwrap();
        let mut res = String::new();
        guard.by_name("_commands")?.read_to_string(& mut res)?;
        Ok(res)
    }

    ///Extract the specified file `name` to `destination`
    pub fn extract<P: AsRef<Path>>(& self, name: &str, destination: P) -> Result<()> {

        let mut guard = self.archive.lock().unwrap();

        let mut afile = guard.by_name(name)?;

        if name.as_bytes()[1] == 'd' as u8 {

            let mut temp = tempfile::tempfile().unwrap();

            std::io::copy(& mut afile, & mut temp)?;

            temp.seek(SeekFrom::Start(0)).unwrap();

            let mut archive = zip::ZipArchive::new(temp).unwrap();

            std::fs::create_dir(destination.as_ref())?;

            archive.extract(destination)?;

            Ok(())
        } else {
            //Create constraint has been removed for the inverse of the Edit step, which restores a file from an oak archive
            //and replace the original file
            let mut dfile = OpenOptions::new().write(true).create(true).open(destination)?;
            std::io::copy(& mut afile, & mut dfile)?;

            Ok(())
        }


    }
}

///A struct used to write to an oak archive
pub struct OakWrite {
    data: Mutex<(ZipWriter<std::fs::File>, u32)>
}

impl OakWrite {
    ///Create a new oak archive and return an `OakWrite` object
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            data: Mutex::new((ZipWriter::new(OpenOptions::new().create_new(true).write(true).open(path.as_ref()).unwrap()), 0)),
        }
    }

    ///Archive a file or folder into the archive
    pub fn archive<P: AsRef<Path>>(& self, path: P) -> String {

        let mut guard = self.data.lock().unwrap();

        let (archive, count) = guard.deref_mut();

        if path.as_ref().is_dir() {
            //self.archive.add_directory(path.as_ref()., FileOptions::default());

            let mut temp = tempfile::tempfile().unwrap();

            //let mut temp = std::fs::OpenOptions::new().read(true).write(true).create(true).open("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\create.zip").unwrap();

            {
                let mut writer = ZipWriter::new(&temp);

                writer.create_from_directory(&PathBuf::from(path.as_ref())).unwrap();

            }

            let identifier = format!("_d_{}", count);
            archive.start_file(identifier.clone(), FileOptions::default()).unwrap();

            temp.seek(SeekFrom::Start(0)).unwrap();

            std::io::copy(& mut temp, archive).unwrap();
            *count = *count + 1;


            identifier
        } else if path.as_ref().is_file() {
            let identifier =format!("_{}", count);
            archive.start_file(identifier.clone(), FileOptions::default()).unwrap();
            let mut file  = OpenOptions::new().read(true).open(path.as_ref()).unwrap();
            std::io::copy(& mut file, archive).unwrap();
            *count = *count + 1;
            identifier
        } else {
            panic!("{:?} is not a file or folder", path.as_ref());
        }



    }

    ///Write the commands list to the archive
    pub fn commands(& self, commands: & str) {

        let mut guard = self.data.lock().unwrap();

        let (archive, _) = guard.deref_mut();

        archive.start_file(format!("_commands"), FileOptions::default()).unwrap();
        archive.write_all(commands.as_bytes()).unwrap();
    }

    /*
    ///Finish the archive. This is called on `drop`
    pub fn finish(& mut self) -> Result<()> {
        self.archive.finish()?;
        Ok(())
    }
    */

}
