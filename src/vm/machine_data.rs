use std::path::Path;
use tempfile::TempDir;
use crate::{OakRead, OakWrite};
use crate::vm::operand::Operand;

pub struct Data {
    pub (crate) temp: Option<TempDir>,
    pub (crate) install_archive: OakRead,
    pub (crate) uninstall_archive: Option<OakWrite>,
    pub (crate) inverse: Option<Vec<(String, Vec<Operand>)>>,
    pub (crate) failed: bool,
}

impl Data {

    pub fn install<P: AsRef<Path>>(installer: P, uninstaller: P) -> Self {
        let install_archive = OakRead::new(installer.as_ref()).unwrap();

        let uninstall_archive = OakWrite::new(uninstaller.as_ref());

        let temp = tempfile::tempdir().unwrap();

        Self {
            temp: Some(temp),
            install_archive,
            uninstall_archive: Some(uninstall_archive),
            inverse: Some(vec![]),
            failed: false,
        }

    }

    pub fn uninstall<P: AsRef<Path>>(uninstaller: P) -> Self {
        let uninstall_archive =  OakRead::new(uninstaller).unwrap();

        Self {
            temp: None,
            install_archive: uninstall_archive,
            uninstall_archive: None,
            inverse: None,
            failed: false,
        }
    }

    //If a function fails in the machine, this function should be called to clean up, and to
    //Undo any installation steps taken if this is an installer
    /*pub fn reverse(&self) {

    }*/

    /*pub fn finish(&self) {

    }*/

}