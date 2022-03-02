use crate::steps::Step;
use serde::{Serialize, Deserialize};
use crate::oak::OakRead;
use crate::oak::OakWrite;
use crate::error::{Error, Result};

///A command is a series of steps, executed in order. It also has an inverse, defined as the inverse of each step in reverse order
#[derive(Debug, Serialize, Deserialize)]
pub struct Command(pub Vec<Step>);

impl Command {
    ///Iterate over each step, execute the action, then return the inverse action
    pub fn action(&self, install_archive: & mut OakRead, uninstall_archive: & mut OakWrite) -> Result<Self> {

        let mut inverses = Vec::with_capacity(self.0.len());

        for step in self.0.iter() {
            match step.action(install_archive, Some(uninstall_archive)) {
                Ok(inv) => {
                    if let Some(s) = inv {
                        inverses.push(s);
                    }
                }
                Err(e) => {
                    //If a step fails, we must iterate in reverse order over the steps executed so far and execute their inverses

                    for inverse in inverses {
                        inverse.action(install_archive, Some(uninstall_archive));
                    }

                    return Err(e);
                }
            }

        }

        inverses.reverse();

        Ok(Self(inverses))
    }

    ///Iterate over each step, treating it as an inverse
    pub fn inverse(&self, uninstall_archive: & mut OakRead) {
        for step in self.0.iter() {
            step.action(uninstall_archive, None).unwrap();
        }
    }

    ///Returns true iif the underlying Vec<Step> is empty
    pub fn is_empty(& self) -> bool {
        self.0.is_empty()
    }
}
