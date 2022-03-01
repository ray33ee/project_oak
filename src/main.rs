extern crate zip;
extern crate fs_extra;
extern crate threadpool;
extern crate serde;
extern crate serde_json;

use command::Command;
use steps::{Step, FileType};
use std::path::PathBuf;
use std::thread::ThreadId;
use threadpool::ThreadPool;
use oak::{OakRead, OakWrite};

mod error;
mod steps;
mod command;
mod vm;
mod oak;

fn main() {

    let command1 = Command(
        vec![
             Step::Copy { source: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\file"), destination: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\install") },
             Step::Rename { from: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\install\\xparc"), to: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\install\\shanrkbark") }]);

    let command2 = Command(vec![Step::Print { message: format!("Printed from a command thread!") }]);

    let command3 = Command(vec![Step::File { name: "_0".to_string(), destination: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\SkypeIcon.idb") }]);

    let command4 = Command(vec![Step::Delete { path: PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\p.txt") }]);
    
    let commands = vec![command1, command2, command3, command4];

    let inverses = {
        let mut oak_read = OakRead::new("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\a.zip");

        let mut oak_write = OakWrite::new("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\b.zip");

        //Join all threads and collect the calculated inverses
        let inverses = commands.iter().map(|x| {
            let inverse = x.action(&mut oak_read, &mut oak_write);

            inverse
        }).collect();

        oak_write.commands(&inverses);

        inverses
    };

    let mut oak_read = OakRead::new("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\b.zip");

    for inv in inverses.iter() {
        inv.inverse(& mut oak_read);
    }

}
