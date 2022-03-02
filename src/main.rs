extern crate zip;
extern crate fs_extra;
extern crate threadpool;
extern crate serde;
extern crate serde_json;

use command::Command;
use steps::{Step, FileType};
use std::path::{PathBuf, Path};
use std::thread::ThreadId;
use threadpool::ThreadPool;
use oak::{OakRead, OakWrite};
use error::Error;

mod error;
mod steps;
mod command;
mod vm;
mod oak;

use crate::error::Result;

///Install from the `installer` file, and write the uninstaller to `uninstaller`
fn install<P: AsRef<Path>>(installer: P, uninstaller: P) -> Result<()> {
    let mut oak_read = OakRead::new(installer.as_ref()).unwrap();

    let commands = oak_read.commands().unwrap();

    let mut oak_write = OakWrite::new(uninstaller.as_ref());

    let mut inverses = Vec::with_capacity(commands.len());

    for command in commands {
        match command.action(&mut oak_read, &mut oak_write) {
            Ok(inv) => {
                if !inv.is_empty() {
                    inverses.push(inv);
                }
            }
            Err(e) => {
                println!("Error: {:?}", &e);

                //A command has failed. The failed command has itself been reversed (handled in Command::action)
                //But the other commands that succeeded needs reversing too:
                for inv in inverses.iter() {
                    inv.inverse(& mut oak_read);
                }

                //Delete the uninstaller we created
                std::fs::remove_file(uninstaller.as_ref()).unwrap();

                return Err(e);
            }
        }
    }

    oak_write.commands(&inverses);

    Ok(())
}

///Run the `uninstaller` file as an uninstaller
fn uninstall<P: AsRef<Path>>(uninstaller: P) -> Result<()> {

    let mut oak_read = OakRead::new(uninstaller.as_ref()).unwrap();

    for inv in oak_read.commands().unwrap().iter() {
        inv.inverse(& mut oak_read);
    }

    std::fs::remove_file(uninstaller.as_ref()).unwrap();

    Ok(())
}

fn main() {

    let command1 = Command(
        vec![
            Step::Copy { source: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\file"), destination: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\install") },
            Step::Rename { from: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\install\\xparc"), to: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\install\\shanrkbark") }]);

    let command2 = Command(vec![Step::Print { message: format!("Printed from a command thread!") }]);

    let command3 = Command(vec![Step::File { name: "_0".to_string(), destination: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\SkypeIcon.idb") }]);

    let command4 = Command(vec![Step::Delete { path: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\p.txt") }]);

    //let command5 = Command(vec![Step::Rename { from: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\stuff.txt"), to: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\renamed.txt") },
    //                            Step::Panic]);

    let commands = vec![command1, command2, command3, command4];

    println!("{}", serde_json::to_string_pretty(&commands).unwrap());


    //install("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\a.zip", "E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\b.zip").unwrap();

    uninstall("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\b.zip").unwrap();



}
