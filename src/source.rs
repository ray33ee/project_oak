use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use crate::{hlc, Info};

///Struct containing all the necessary information to create an installer
#[derive(Serialize, Deserialize)]
pub struct Source {
    ///Lua code to add to the installer
    code: String,

    ///Extra data to include in the archive
    info: crate::oak::Info,
}

impl Source {

    ///Take a source struct and create an installer
    pub fn create_installer(&self,  path: &Path) {
        hlc::create_installer(self.code.as_str(), path, &self.info).unwrap()
    }

    pub fn load_from_path(path: &Path) -> Self {
        let file = OpenOptions::new().read(true).open(path).unwrap();

        serde_json::from_reader(file).unwrap()
    }

}

impl Default for Source {
    fn default() -> Self {
        Self {
            code: "".to_string(),
            info: Info::default()
        }
    }
}

