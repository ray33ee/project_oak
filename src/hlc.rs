use crate::error::Result;
use std::fs::OpenOptions;
use clap::lazy_static::lazy_static;
use stack_vm::{Builder, Code, Machine, WriteManyTable};
use crate::vm::machine_data::Data;
use std::path::{Path};
use crate::oak::{OakRead, OakWrite};


// Really simple language for oak
pub fn create_installer<P: AsRef<Path>>(_source_file: P, _installer_path: P) -> Result<()> {

    lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new("([\\s]+)data[\\s]+p\"([^\"]+)\"[\\s]*").unwrap();
    }

    let original_source = std::fs::read_to_string(_source_file).unwrap();

    let mut oak_writer = OakWrite::new(_installer_path.as_ref());

    //Loop over all data commands, add each file (named as an argument to the data command)
    //replace with argument with the name (returned when a file is added to the archive)
    let cow = RE.replace_all(original_source.as_str(), |caps: &regex::Captures| {
        let name = oak_writer.archive(&caps[2]);
        format!("{}data \"{}\"", &caps[1], name) //We use &cap[1] to preserve the original whitespace (including an end of line line break and maintain readability
    });

    let extra = "
    jmp \"_end\"

._error:
    unwind
    panic
._end:
    ";

    oak_writer.commands(format!("{}{}", cow.as_ref(), extra).as_str());



    Ok(())

}


pub fn install<P: AsRef<Path>>(installer: P, uninstaller: P) {
    let failed = {
        let table = crate::vm::instructions::get_instruction_table();

        //Run the installation step
        let mut rel = install_step(Data::install(installer.as_ref(), uninstaller.as_ref()));

        let mut inverse_builder = Builder::new(&table);

        for (s, v) in rel.inverse.as_ref().unwrap() {
            inverse_builder.push(&s, v.clone());
        }


        let inverse_source = format!("{:?}", Code::from(inverse_builder));

        //Save uninstaller code
        let archive = rel.uninstall_archive.as_mut().unwrap();

        archive.commands(&inverse_source);

        rel.failed
    };


    //If failed, then call uninstall
    if failed {
        uninstall(uninstaller);
    }

}

pub fn uninstall<P: AsRef<Path>>(uninstaller: P) {
    install_step(Data::uninstall( uninstaller));
}

fn install_step(mut data: Data) -> Data {

    let table = crate::vm::instructions::get_instruction_table();

    // Get installer source
    let install_source = &data.install_archive.commands().unwrap();

    let code = Code::parse(&install_source, &table);

    //Setup and run VM
    let constants: WriteManyTable<crate::vm::operand::Operand> = WriteManyTable::new();
    let mut machine = Machine::new(code, &constants, &table, data);
    machine.run();

    machine.release()
}

///List all the files, folders and commands in an oak repo
pub fn list<P: AsRef<Path>>(repo: P) -> Result<()> {

    {
        let archive = zip::ZipArchive::new(OpenOptions::new().read(true).open(repo.as_ref())?)?;

        println!("Stored files:");

        for name in archive.file_names() {
            if name != "_commands" {
                println!("    {}", name);
            }
        }
    }

    let mut read = OakRead::new(repo.as_ref()).unwrap();

    println!("Commands:");

    println!("{}", read.commands().unwrap());


    Ok(())
}
