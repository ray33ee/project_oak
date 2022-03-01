use crate::steps::Step;
use serde::{Serialize, Deserialize};
use crate::oak::OakRead;
use crate::oak::OakWrite;

#[derive(Debug, Serialize, Deserialize)]
pub struct Command(pub Vec<Step>);

impl Command {
    pub fn action(&self, install_archive: & mut OakRead, uninstall_archive: & mut OakWrite) -> Self {

        let mut inverse = Vec::with_capacity(self.0.len());

        for step in self.0.iter() {
            let inv = step.action(install_archive, Some(uninstall_archive)).unwrap();

            if let Some(s) = inv {
                inverse.push(s);
            }
        }

        inverse.reverse();

        Self(inverse)
    }

    pub fn inverse(&self, uninstall_archive: & mut OakRead) {
        for step in self.0.iter() {
            step.action(uninstall_archive, None).unwrap();
        }
    }
}
