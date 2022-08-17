
use stack_vm::{Instruction, InstructionTable, Machine};
use crate::vm::operand::Operand;
use crate::PathType;
use crate::vm::machine_data::Data;

/* Stack operations */

//Push a literal onto the stack
fn push(machine: & mut Machine<Operand, Data>, args: &[usize]) {
    let arg = machine.get_data(args[0]).clone();
    machine.operand_push(arg);
}

//Clone the top of the stack
fn clone(machine: & mut Machine<Operand, Data>, _args: &[usize]) {
    let val = machine.operand_pop();
    machine.operand_push(val.clone());
    machine.operand_push(val);
}

/* Conversion operations */

//Convert item to bool
fn bool(machine: & mut Machine<Operand, Data>, _args: &[usize]) {
    let val = machine.operand_pop();
    machine.operand_push(Operand::Bool(val.bool()))
}

//Convert item to string
/*fn str(machine: & mut Machine<Operand, Data>, _args: &[usize]) {
    let val = String::from(machine.operand_pop());
    machine.operand_push(Operand::from(val))
}*/

//Convert item to integer
/*fn int(machine: & mut Machine<Operand, Data>, _args: &[usize]) {
    let val = i64::from(machine.operand_pop());
    machine.operand_push(Operand::from(val))
}*/

//Convert item to path
/*fn path(machine: & mut Machine<Operand, Data>, _args: &[usize]) {
    let val = PathBuf::from(machine.operand_pop());
    machine.operand_push(Operand::from(val))
}*/

/* Arithmetic operations */

//Add two operands
fn add(machine: & mut Machine<Operand, Data>, _args: &[usize]) {
    let a = machine.operand_pop();
    let b = machine.operand_pop();

    machine.operand_push(a.try_add(&b).unwrap_or_else(|_| panic!("Oak Script Error: Cannot add specified operands {:?} and {:?}", a, b)))
}

/* Control flow operations */

//Unconditional jump
fn jmp(machine: & mut Machine<Operand, Data>, args: &[usize]) {
    let arg = machine.get_data(args[0]).clone();
    if let Operand::String(str) = arg {
        machine.jump(&str);
    } else {
        panic!("Oak Script Error: jmp operand must be a literal string label ({:?})", arg)
    }
}

//Jump if the top of the stack is zero
fn jz(machine: & mut Machine<Operand, Data>, args: &[usize]) {
    let condition = machine.operand_pop();
    if condition.bool() {
        jmp(machine, args);
    }
}

fn call(machine: & mut Machine<Operand, Data>, args: &[usize]) {
    let arg = machine.get_data(args[0]).clone();
    if let Operand::String(str) = arg {
        machine.call(&str);
    } else {
        panic!("Oak Script Error: call operand must be a literal string label ({:?})", arg)
    }
}

fn ret(machine: & mut Machine<Operand, Data>, _args: &[usize]) {
    machine.ret();
}

/* Install/uninstall steps */

fn data(machine: & mut Machine<Operand, Data>, args: &[usize]) {
    let name = machine.get_data(args[0]).clone();
    let destination = PathType::try_from(machine.operand_pop()).unwrap();

    let destination_path = destination.path(machine.data().temp.as_ref());

    machine.data().install_archive.extract(String::try_from(name).unwrap().as_str(), &destination_path).unwrap();

    if !destination.is_temp() {
        if let Some(list) = & mut machine.data().inverse {
            list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(destination_path.as_path().to_path_buf()))]));
            list.insert(1, (String::from("delete"), vec![]));
        }
    }
}

fn _move(machine: & mut Machine<Operand, Data>, _args: &[usize]) {

    let source = PathType::try_from(machine.operand_pop()).unwrap();
    let destination = machine.operand_pop().absolute_path(machine.data().temp.as_ref());

    let source_path = source.path(machine.data().temp.as_ref());

    if destination.exists() {
        panic!("Already exists") //, crate::Error::AlreadyExists)
    } else {
        if source_path.is_dir() {
            let mut options = fs_extra::dir::CopyOptions::default();
            options.content_only = true;

            fs_extra::dir::move_dir(&source_path, &destination, &options).unwrap();

        } else if source_path.is_file() {
            let options = fs_extra::file::CopyOptions::default();

            fs_extra::file::move_file(&source_path, &destination, &options).unwrap();

        } else {
            panic!("File is not a path or file");
        }


        if let Some(list) = & mut machine.data().inverse {
            match &source {
                PathType::Absolute(s) => {


                    list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(s.clone()))]));
                    list.insert(1, (String::from("push"), vec![Operand::Path(PathType::Absolute(destination))]));
                    list.insert(2, (String::from("move"), vec![]));

                    //Ok(Some(Step::Move { source: SpecialPath::Path(destination.clone()), destination: s.clone() }))
                }
                PathType::Temporary(_) => {


                    list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(destination))]));
                    list.insert(1, (String::from("delete"), vec![]));

                    //Ok(Some(Step::Delete { path: destination.clone() }))
                }
            }

        }
    }
}

fn delete(machine: & mut Machine<Operand, Data>, _args: &[usize]) {

    let path = machine.operand_pop().absolute_path(machine.data().temp.as_ref());

    let name = match machine.data().uninstall_archive.as_mut() {
        None => { None }
        Some(archive) => {Some(archive.archive(&path))}
    };

    if path.is_dir() {
        std::fs::remove_dir_all(&path).unwrap();
    } else if path.is_file() {
        std::fs::remove_file(&path).unwrap();
    } else {
        panic!("File is not a path or file");
    };

    if let Some(_) = machine.data().uninstall_archive {

        if let Some(list) = & mut machine.data().inverse {

            list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(path.clone()))]));
            list.insert(1, (String::from("push"), vec![Operand::String(name.unwrap())]));
            list.insert(2, (String::from("data"), vec![]));

        }

    }
}

fn copy(machine: & mut Machine<Operand, Data>, _args: &[usize]) {
    let source = PathType::try_from(machine.operand_pop()).unwrap();
    let destination = PathType::try_from(machine.operand_pop()).unwrap();

    let source_path = source.path(machine.data().temp.as_ref());
    let destination_path = destination.path(machine.data().temp.as_ref());

    if destination_path.exists() {
        panic!("{:?}", crate::error::Error::AlreadyExists)
    } else {
        if source_path.is_file() {

            std::fs::copy(&source_path, &destination_path).unwrap();
        } else if source_path.is_dir() {
            let mut options = fs_extra::dir::CopyOptions::default();
            options.content_only = true;

            fs_extra::dir::copy(&source_path, &destination_path, &options).unwrap();
        } else {
            panic!("Source is not a file or directory");
        }

        if !destination.is_temp() {

            if let Some(list) = & mut machine.data().inverse {


                list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(destination_path.as_path().to_path_buf()))]));
                list.insert(1, (String::from("delete"), vec![]));


            }


        }
    }


}


// Not actually an instruction, it is instead used by the other create methods
/*fn create(machine: & mut Machine<Operand, Data>, _args: &[usize]) {

}*/

fn rename(machine: & mut Machine<Operand, Data>, _args: &[usize]) {


    let from = PathType::try_from(machine.operand_pop()).unwrap();
    let to = PathType::try_from(machine.operand_pop()).unwrap();

    let temp = machine.data().temp.as_ref();

    let from_path = from.path(temp);
    let to_path = to.path(temp);

    if from.is_temp() != to.is_temp() {
        panic!("Arguments for rename must either be both temporary, or both permanent locations")
    }

    std::fs::rename(from_path.as_path(), to_path.as_path()).unwrap();

    if !from.is_temp() {


        if let Some(list) = & mut machine.data().inverse {
            list.insert(0, (String::from("push"), vec![Operand::Path(from.clone())]));
            list.insert(1, (String::from("push"), vec![Operand::Path(to.clone())]));
            list.insert(2, (String::from("rename"), vec![]));

        }
        //Ok(Some(Step::Rename {from: to.clone(), to: from.clone()}))
    }

}

fn zip(machine: & mut Machine<Operand, Data>, _args: &[usize]) {

    let archive = PathType::try_from(machine.operand_pop()).unwrap();
    let folder = PathType::try_from(machine.operand_pop()).unwrap();

    let temp = machine.data().temp.as_ref();

    zip_extensions::write::zip_create_from_directory(&archive.path(temp), &folder.path(temp)).unwrap();

    if !archive.is_temp() {

        if let Some(list) = & mut machine.data().inverse {

            list.insert(0, (String::from("push"), vec![Operand::Path(archive)]));
            list.insert(1, (String::from("delete"), vec![]));

        }

        //Ok(Some(Step::Delete { path: archive.path(temp) }))
    }
}

fn unzip(machine: & mut Machine<Operand, Data>, _args: &[usize]) {

    let archive = PathType::try_from(machine.operand_pop()).unwrap();
    let folder = PathType::try_from(machine.operand_pop()).unwrap();

    let temp = machine.data().temp.as_ref();


    std::fs::create_dir(&folder.path(temp)).unwrap();

    zip_extensions::read::zip_extract(&archive.path(temp), &folder.path(temp)).unwrap();

    if !archive.is_temp() {
        //Ok(Some(Step::Delete { path: folder.path(temp) }))

        if let Some(list) = & mut machine.data().inverse {

            list.insert(0, (String::from("push"), vec![Operand::Path(folder)]));
            list.insert(1, (String::from("delete"), vec![]));

        }


    }
}



pub fn get_instruction_table() -> InstructionTable<Operand, Data> {

    let mut table = InstructionTable::new();

    table.insert(Instruction::new(0, "push", 1, push));
    table.insert(Instruction::new(1, "clone", 0, clone));

    table.insert(Instruction::new(100, "bool", 0, bool));

    table.insert(Instruction::new(200, "jmp", 1, jmp));
    table.insert(Instruction::new(201, "jz", 1, jz));
    table.insert(Instruction::new(202, "call", 1, call));
    table.insert(Instruction::new(203, "ret", 0, ret));

    table.insert(Instruction::new(300, "add", 0, add));

    table.insert(Instruction::new(400, "data", 1, data));
    table.insert(Instruction::new(401, "copy", 0, copy));
    table.insert(Instruction::new(402, "delete", 0, delete));
    table.insert(Instruction::new(403, "move", 0, _move));
    table.insert(Instruction::new(404, "rename", 0, rename));
    table.insert(Instruction::new(405, "zip", 0, zip));
    table.insert(Instruction::new(406, "unzip", 0, unzip));

    table

}