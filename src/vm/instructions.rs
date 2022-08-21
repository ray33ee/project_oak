use std::path::{Path};
use registry::Security;
use stack_vm::{Instruction, InstructionTable, Machine};
use crate::vm::operand::Operand;
use crate::PathType;
use crate::registry_ex::RootKey;
use crate::vm::machine_data::Data;

//This macro will unwrap Results, and handle them accordingly.
// It will push a message onto the stack (used by the panic instruction)
// and jump to the error section (at the end of the code).
//First argument is the object to unwrap, second is the machine and third
//is a closure that takes the error type, and returns an Operand that is
//used as an argument to the panic instruction. This closure can also be used to clean up
//if an instruction has partially been invoked
macro_rules! unwrap_or_return {
    ( $e:expr , $m:expr) => {
        match $e {
            Ok(x) => x,
            Err(_) => {
                $m.jump("_error");
                $m.operand_push(Operand::String("Project Oak Unwrap error".to_string()));
                return
            },
        }
    }
}

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

fn panic(machine: & mut Machine<Operand, Data>, _args: &[usize]) {

    let message = machine.operand_pop();

    println!("Oak script Panic instruction: {:?}", message);
}

fn unwind(machine: & mut Machine<Operand, Data>, _args: &[usize]) {
    machine.data().failed = true;
}

/* Install/uninstall steps */

fn data(machine: & mut Machine<Operand, Data>, args: &[usize]) {
    let name = machine.get_data(args[0]).clone();
    let destination = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));

    let destination_path = destination.path(machine.data().temp.as_ref());

    machine.data().install_archive.extract(String::try_from(name).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string")).as_str(), &destination_path).unwrap();

    if !destination.is_temp() {
        if let Some(list) = & mut machine.data().inverse {
            list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(destination_path.as_path().to_path_buf()))]));
            list.insert(1, (String::from("delete"), vec![]));
        }
    }
}

fn _move(machine: & mut Machine<Operand, Data>, _args: &[usize]) {

    let source = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));
    let destination = machine.operand_pop().absolute_path(machine.data().temp.as_ref());

    let source_path = source.path(machine.data().temp.as_ref());

    if destination.exists() {
        panic!("Already exists") //, crate::Error::AlreadyExists)
    } else {
        if source_path.is_dir() {
            let mut options = fs_extra::dir::CopyOptions::default();
            options.content_only = true;

            unwrap_or_return!(fs_extra::dir::move_dir(&source_path, &destination, &options), machine);

        } else if source_path.is_file() {
            let options = fs_extra::file::CopyOptions::default();

            unwrap_or_return!(fs_extra::file::move_file(&source_path, &destination, &options), machine);

        } else {
            machine.operand_push(Operand::I64(100));
            machine.jump("_error");
            return;
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
        unwrap_or_return!(std::fs::remove_dir_all(&path), machine);
    } else if path.is_file() {
        unwrap_or_return!(std::fs::remove_file(&path), machine);
    } else {
        panic!("File is not a path or file");
    };

    if let Some(_) = machine.data().uninstall_archive {

        if let Some(list) = & mut machine.data().inverse {

            list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(path.clone()))]));
            list.insert(1, (String::from("data"), vec![Operand::String(name.unwrap())]));

        }

    }
}

fn copy(machine: & mut Machine<Operand, Data>, _args: &[usize]) {
    let source = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));
    let destination = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));

    let source_path = source.path(machine.data().temp.as_ref());
    let destination_path = destination.path(machine.data().temp.as_ref());

    if destination_path.exists() {
        panic!("{:?}", crate::error::Error::AlreadyExists)
    } else {
        if source_path.is_file() {

            unwrap_or_return!(std::fs::copy(&source_path, &destination_path), machine);
        } else if source_path.is_dir() {
            let mut options = fs_extra::dir::CopyOptions::default();
            options.content_only = true;

            unwrap_or_return!(fs_extra::dir::copy(&source_path, &destination_path, &options), machine);
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
fn create(_machine: & mut Machine<Operand, Data>, _args: &[usize]) {

}


fn zip(machine: & mut Machine<Operand, Data>, _args: &[usize]) {

    let archive = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));
    let folder = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));

    let temp = machine.data().temp.as_ref();

    unwrap_or_return!(zip_extensions::write::zip_create_from_directory(&archive.path(temp), &folder.path(temp)), machine);

    if !archive.is_temp() {

        if let Some(list) = & mut machine.data().inverse {

            list.insert(0, (String::from("push"), vec![Operand::Path(archive)]));
            list.insert(1, (String::from("delete"), vec![]));

        }
    }
}

fn unzip(machine: & mut Machine<Operand, Data>, _args: &[usize]) {

    let archive = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));
    let folder = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));



    let temp = machine.data().temp.as_ref();


    //std::fs::create_dir(&folder.path(temp)).unwrap();

    unwrap_or_return!(zip_extensions::read::zip_extract(&archive.path(temp), &folder.path(temp)), machine);

    if !archive.is_temp() {
        //Ok(Some(Step::Delete { path: folder.path(temp) }))

        if let Some(list) = & mut machine.data().inverse {

            list.insert(0, (String::from("push"), vec![Operand::Path(folder.clone())]));
            list.insert(1, (String::from("delete"), vec![]));

        }


    }
}


fn download(machine: & mut Machine<Operand, Data>, _args: &[usize]) {


    let url = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    let destination = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));

    let response = unwrap_or_return!(reqwest::blocking::get(url), machine);

    let temp= machine.data().temp.as_ref();

    let file_name = if destination.path(temp).is_dir() {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() {None} else {Some(name)})
            .unwrap_or("tmp.bin");

        destination.path(temp).join(fname)
    } else if destination.path(temp).is_file() {
        destination.path(temp)
    } else {
        panic!("Download destination must be a directory or file")
    };




    let mut dest = unwrap_or_return!(std::fs::File::create(file_name.clone()), machine);

    if !destination.is_temp() {

        if let Some(list) = & mut machine.data().inverse {

            list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(file_name.clone()))]));
            list.insert(1, (String::from("delete"), vec![]));

        }
    }

    let content = unwrap_or_return!(response.text(), machine);
    unwrap_or_return!(std::io::copy(&mut content.as_bytes(), &mut dest), machine);

}

fn edit(machine: & mut Machine<Operand, Data>, _args: &[usize]) {

    use std::io::Write;

    let command = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));

    let s = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));

    let source = s.path(machine.data().temp.as_ref());


    //Load `source`
    let content = unwrap_or_return!(std::fs::read_to_string(source.as_path()), machine);


    //Perform find and replace
    let res = unwrap_or_return!(sedregex::find_and_replace(content.as_str(), &[command]), machine);


    match machine.data().uninstall_archive.as_mut() {
        None => {  }
        Some(archive) => {

            let name = archive.archive(&source);

            if !s.is_temp() {


                if let Some(list) = & mut machine.data().inverse {
                    list.insert(0, (String::from("push"), vec![Operand::Path(s.clone())]));
                    list.insert(1, (String::from("delete"), vec![]));


                    list.insert(2, (String::from("push"), vec![Operand::Path(s.clone())]));
                    list.insert(3, (String::from("data"), vec![Operand::String(name)]));
                }

            }


        }
    }

    //Save back to `source`
    let mut fh = unwrap_or_return!(std::fs::OpenOptions::new().write(true).open(source.as_path()), machine);

    unwrap_or_return!(fh.write_all(res.as_ref().as_bytes()), machine);


}


fn write_reg_key(machine: & mut Machine<Operand, Data>, _args: &[usize]) {


    let root = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    let key = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));

    let reg = registry::Hive::from(&RootKey::from(root.as_str())); //.open(key, Security::AllAccess)?;



    //Look for the oldest ancestor that was newly created as part of this call.
    //For example, if a registry key looks like 'example\path\to\' before, and
    //'example\path\to\demonstrate\inverse' after, then you want to delete 'example\path\to\demonstrate'
    let common = {
        let ancestors = Path::new(key.as_str()).ancestors().collect::<Vec<_>>();

        ancestors
            .iter()
            .rev()
            .map(|x| {

                let o= reg.open(x.to_str().unwrap(), Security::Read);

                (x.clone(), o)
            })
            .find(|(_, x)| x.is_err())
            .map(|(path, _)| path)
    };

    unwrap_or_return!(reg.create(key.as_str(), Security::AllAccess), machine);


    if let Some(p) = common {
        if let Some(list) = &mut machine.data().inverse {
            list.insert(0, (String::from("push"), vec![Operand::String(String::from(p.to_str().unwrap()))]));
            list.insert(1, (String::from("push"), vec![Operand::String(root.clone())]));
            list.insert(2, (String::from("reg_delete_key"), vec![]));
        }
    }
}


fn write_reg_value(machine: & mut Machine<Operand, Data>, _args: &[usize]) {


    let root = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    let key = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    let value = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    let data = registry::Data::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to registry data type"));

    let reg = unwrap_or_return!(registry::Hive::from(&RootKey::from(root.as_str())).open(key.as_str(), Security::AllAccess), machine);

    //For inverses, there are two cases. If the value already exists (i.e. we are modifying it)
    //and if the value does not already exist (i.e. we are creating it). In the first case,
    //the inverse is to revert to the previous value. In the second case, the inverse is
    //to delete the value.

    if let Some(list) = &mut machine.data().inverse {
        if let Err(registry::value::Error::NotFound(_,_)) = reg.value(value.as_str()) {

            list.insert(0, (String::from("push"), vec![Operand::String(value.clone())]));
            list.insert(1, (String::from("push"), vec![Operand::String(key.clone())]));
            list.insert(2, (String::from("push"), vec![Operand::String(root.clone())]));
            list.insert(3, (String::from("reg_delete_value"), vec![]));
        } else {

            let old_value = reg.value(value.as_str()).unwrap();

            list.insert(0, (String::from("push"), vec![Operand::try_from(old_value).unwrap()]));
            list.insert(1, (String::from("push"), vec![Operand::String(value.clone())]));
            list.insert(2, (String::from("push"), vec![Operand::String(key.clone())]));
            list.insert(3, (String::from("push"), vec![Operand::String(root.clone())]));
            list.insert(4, (String::from("reg_write_value"), vec![]));

        }
    }

    unwrap_or_return!(reg.set_value(value.as_str(), &data), machine);


    //Ok(Some(inverse))
}

fn delete_reg_value(machine: & mut Machine<Operand, Data>, _args: &[usize]) {

    let root = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    let key = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    let value = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));

    let reg = unwrap_or_return!(registry::Hive::from(&RootKey::from(root.as_str())).open(key.as_str(), Security::AllAccess), machine);

    let old_value = reg.value(value.as_str()).unwrap();

    unwrap_or_return!(reg.delete_value(value.as_str()), machine);

    if let Some(list) = &mut machine.data().inverse {

        list.insert(0, (String::from("push"), vec![Operand::try_from(old_value).unwrap()]));
        list.insert(1, (String::from("push"), vec![Operand::String(value.clone())]));
        list.insert(2, (String::from("push"), vec![Operand::String(key.clone())]));
        list.insert(3, (String::from("push"), vec![Operand::String(root.clone())]));
        list.insert(4, (String::from("reg_write_value"), vec![]));
    }


}

fn recursive_recover(
    regkey: &registry::RegKey,
    rootkey: & Operand,
    list: & mut Vec<(String, Vec<Operand>)>,
    index: & mut usize) {

    let name = regkey.to_string();
    let name = name.split_once("\\").unwrap().1;

    list.insert(*index, (String::from("push"), vec![Operand::String(name.to_string())]));
    list.insert(*index + 1, (String::from("push"), vec![rootkey.clone()]));
    list.insert(*index + 2, (String::from("reg_write_key"), vec![]));

    *index = *index + 3;

    for value in regkey.values().map(|x| x.unwrap()) {

        list.insert(*index, (String::from("push"), vec![Operand::try_from(value.data().clone()).unwrap()]));
        list.insert(*index+1, (String::from("push"), vec![Operand::String(value.name().to_string().unwrap())]));
        list.insert(*index+2, (String::from("push"), vec![Operand::String(name.to_string())]));
        list.insert(*index + 3, (String::from("push"), vec![rootkey.clone()]));
        list.insert(*index + 4, (String::from("reg_write_value"), vec![]));

        *index = *index + 5;

    }

    for key in regkey.keys().map(|x| x.unwrap()) {
        recursive_recover(&key.open(Security::Read).unwrap(), rootkey, list, index);
    }
}

fn delete_reg_key(machine: & mut Machine<Operand, Data>, _args: &[usize]) {


    let root = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    let key = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));

    let reg = unwrap_or_return!(registry::Hive::from(&RootKey::from(root.as_str())).open(key.as_str(), Security::AllAccess), machine);


    if let Some(list) = &mut machine.data().inverse {
        let mut index = 0;
        recursive_recover(&reg, &Operand::String(root.clone()), list, & mut index);
    }

    unwrap_or_return!(reg.delete("", true), machine); //Delete the contents of the key
    unwrap_or_return!(reg.delete_self(false), machine); //Delete the key itself





}

fn test(machine: & mut Machine<Operand, Data>, _args: &[usize]) {
    let thing: Option<i32> = None;


    unwrap_or_return!(thing.ok_or(()), machine);

    println!("Here");
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
    table.insert(Instruction::new(204, "panic", 0, panic));
    table.insert(Instruction::new(205, "unwind", 0, unwind));

    table.insert(Instruction::new(300, "add", 0, add));

    table.insert(Instruction::new(400, "data", 1, data));
    table.insert(Instruction::new(401, "copy", 0, copy));
    table.insert(Instruction::new(402, "delete", 0, delete));
    table.insert(Instruction::new(403, "move", 0, _move));
    //table.insert(Instruction::new(404, "rename", 0, rename));
    table.insert(Instruction::new(405, "zip", 0, zip));
    table.insert(Instruction::new(406, "unzip", 0, unzip));
    table.insert(Instruction::new(407, "download", 0, download));
    table.insert(Instruction::new(408, "edit", 0, edit));
    table.insert(Instruction::new(409, "reg_write_key", 0, write_reg_key));
    table.insert(Instruction::new(410, "reg_write_value", 0, write_reg_value));
    table.insert(Instruction::new(411, "reg_delete_key", 0, delete_reg_key));
    table.insert(Instruction::new(412, "reg_delete_value", 0, delete_reg_value));
    table.insert(Instruction::new(413, "create", 0, create));




    table.insert(Instruction::new(9999, "test", 0, test));

    table

}