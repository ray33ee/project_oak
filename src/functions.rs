use std::path::{Path};
use registry::Security;
use tempfile::TempDir;
use crate::{OakRead, OakWrite};
use crate::registry_ex::RootKey;
use crate::path_type::{Inverse, PathType};


pub fn data(installer: & mut OakRead, inverses: Option<& mut Vec<String>>, name: & str, destination: PathType, temp: & TempDir) {
    //let destination = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));

    let destination_path = destination.to_absolute_path(temp);

    installer.extract(name, &destination_path).unwrap();

    if !destination.is_temp() {
        if let Some(list) = inverses {
            //list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(destination_path.as_path().to_path_buf()))]));
            //list.insert(1, (String::from("delete"), vec![]));
        }
    }
}

pub fn _move(inverses: Option<& mut Vec<String>>, source: PathType, destination: & PathType, temp: & TempDir) {

    //let source = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));
    let destination = destination.to_absolute_path(temp);

    let source_path = source.to_absolute_path(temp);

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
            panic!("Not a file or directory");
        }


        if let Some(list) = inverses {
            match &source {
                PathType::Absolute(s) => {


                    //list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(s.clone()))]));
                    //list.insert(1, (String::from("push"), vec![Operand::Path(PathType::Absolute(destination))]));
                    //list.insert(2, (String::from("move"), vec![]));

                    //Ok(Some(Step::Move { source: SpecialPath::Path(destination.clone()), destination: s.clone() }))
                }
                PathType::Temporary(_) => {


                    //list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(destination))]));
                    //list.insert(1, (String::from("delete"), vec![]));

                    //Ok(Some(Step::Delete { path: destination.clone() }))
                }
            }

        }
    }
}

pub fn delete(mut uninstaller: Option<& OakWrite>, inverses: Option<&Inverse>, path: & PathType, temp: & TempDir) {

    let path = path.to_absolute_path(temp);

    let name = match uninstaller.as_mut() {
        None => { None }
        Some(archive) => {Some(archive.archive(&path))}
    };

    println!("path: {}", path.to_str().unwrap());

    if path.is_dir() {
        std::fs::remove_dir_all(&path).unwrap();
    } else if path.is_file() {
        std::fs::remove_file(&path).unwrap();
    } else {
        panic!("File is not a path or file");
    };

    if let Some(_) = uninstaller {

        if let Some(list) = inverses {

            //list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(path.clone()))]));
            //list.insert(1, (String::from("data"), vec![Operand::String(name.unwrap())]));

        }

    }
}

pub fn copy(inverses: Option<& mut Vec<String>>, source: PathType, destination: PathType, temp: & TempDir) {
    //let source = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));
    //let destination = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));

    let source_path = source.to_absolute_path(temp);
    let destination_path = destination.to_absolute_path(temp);

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

            if let Some(list) = inverses {


                //list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(destination_path.as_path().to_path_buf()))]));
                //list.insert(1, (String::from("delete"), vec![]));


            }


        }
    }


}


// Not actually an instruction, it is instead used by the other create methods
pub fn create(uninstaller: Option<& mut OakWrite>, inverses: Option<& mut Vec<String>>) {

}


pub fn zip(inverses: Option<& mut Vec<String>>, archive: PathType, folder: PathType, temp: & TempDir) {

    //let archive = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));
    //let folder = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));

    zip_extensions::write::zip_create_from_directory(&archive.to_absolute_path(temp), &folder.to_absolute_path(temp)).unwrap();

    if !archive.is_temp() {

        if let Some(list) = inverses {

            //list.insert(0, (String::from("push"), vec![Operand::Path(archive)]));
            //list.insert(1, (String::from("delete"), vec![]));

        }
    }
}

pub fn unzip(inverses: Option<& mut Vec<String>>, archive: PathType, folder: PathType, temp: & TempDir) {

    //let archive = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));
    //let folder = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));

    //std::fs::create_dir(&folder.path(temp)).unwrap();

    zip_extensions::read::zip_extract(&archive.to_absolute_path(temp), &folder.to_absolute_path(temp)).unwrap();

    if !archive.is_temp() {
        //Ok(Some(Step::Delete { path: folder.path(temp) }))

        if let Some(list) = inverses {

            //list.insert(0, (String::from("push"), vec![Operand::Path(folder.clone())]));
            //list.insert(1, (String::from("delete"), vec![]));

        }


    }
}


pub fn download(inverses: Option<& mut Vec<String>>, url: & str, destination: PathType, temp: & TempDir) {


    //let url = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    //let destination = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));

    let response = reqwest::blocking::get(url).unwrap();

    let file_name = if destination.to_absolute_path(temp).is_dir() {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() {None} else {Some(name)})
            .unwrap_or("tmp.bin");

        destination.to_absolute_path(temp).join( fname)
    } else if destination.to_absolute_path(temp).is_file() {
        destination.to_absolute_path(temp)
    } else {
        panic!("Download destination must be a directory or file")
    };




    let mut dest = std::fs::File::create(file_name.clone()).unwrap();

    if !destination.is_temp() {

        if let Some(list) = inverses {

            //list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(file_name.clone()))]));
            //list.insert(1, (String::from("delete"), vec![]));

        }
    }

    let content = response.text().unwrap();
    std::io::copy(&mut content.as_bytes(), &mut dest).unwrap();

}

pub fn edit(uninstaller: Option<& mut OakWrite>, inverses: Option<& mut Vec<String>>, s: PathType, command: & str, temp: & TempDir) {

    use std::io::Write;

    //let command = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));

    //let s = PathType::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to path"));

    let source = s.to_absolute_path(temp);


    //Load `source`
    let content = std::fs::read_to_string(source.as_path()).unwrap();


    //Perform find and replace
    let res = sedregex::find_and_replace(content.as_str(), &[command]).unwrap();


    match uninstaller {
        None => {  }
        Some(archive) => {

            let name = archive.archive(&source);

            if !s.is_temp() {


                if let Some(list) = inverses {
                    //list.insert(0, (String::from("push"), vec![Operand::Path(s.clone())]));
                    //list.insert(1, (String::from("delete"), vec![]));


                    //list.insert(2, (String::from("push"), vec![Operand::Path(s.clone())]));
                    //list.insert(3, (String::from("data"), vec![Operand::String(name)]));
                }

            }


        }
    }

    //Save back to `source`
    let mut fh = std::fs::OpenOptions::new().write(true).open(source.as_path()).unwrap();

    fh.write_all(res.as_ref().as_bytes()).unwrap();


}


pub fn write_reg_key(inverses: Option<& mut Vec<String>>, root: & str, key: & str) {


    //let root = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    //let key = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));

    let reg = registry::Hive::from(&RootKey::from(root)); //.open(key, Security::AllAccess)?;



    //Look for the oldest ancestor that was newly created as part of this call.
    //For example, if a registry key looks like 'example\path\to\' before, and
    //'example\path\to\demonstrate\inverse' after, then you want to delete 'example\path\to\demonstrate'
    let common = {
        let ancestors = Path::new(key).ancestors().collect::<Vec<_>>();

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

    reg.create(key, Security::AllAccess).unwrap();


    if let Some(p) = common {
        if let Some(list) = inverses {
            //list.insert(0, (String::from("push"), vec![Operand::String(String::from(p.to_str().unwrap()))]));
            //list.insert(1, (String::from("push"), vec![Operand::String(root.clone())]));
            //list.insert(2, (String::from("reg_delete_key"), vec![]));
        }
    }
}


pub fn write_reg_value(inverses: Option<& mut Vec<String>>, root: &str, key: &str, value: &str, data: registry::Data) {


    //let root = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    //let key = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    //let value = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    //let data = registry::Data::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to registry data type"));

    let reg = registry::Hive::from(&RootKey::from(root)).open(key, Security::AllAccess).unwrap();

    //For inverses, there are two cases. If the value already exists (i.e. we are modifying it)
    //and if the value does not already exist (i.e. we are creating it). In the first case,
    //the inverse is to revert to the previous value. In the second case, the inverse is
    //to delete the value.

    if let Some(list) = inverses {
        if let Err(registry::value::Error::NotFound(_,_)) = reg.value(value) {

            //list.insert(0, (String::from("push"), vec![Operand::String(value.clone())]));
            //list.insert(1, (String::from("push"), vec![Operand::String(key.clone())]));
            //list.insert(2, (String::from("push"), vec![Operand::String(root.clone())]));
            //list.insert(3, (String::from("reg_delete_value"), vec![]));
        } else {

            let old_value = reg.value(value).unwrap();

            //list.insert(0, (String::from("push"), vec![Operand::try_from(old_value).unwrap()]));
            //list.insert(1, (String::from("push"), vec![Operand::String(value.clone())]));
            //list.insert(2, (String::from("push"), vec![Operand::String(key.clone())]));
            //list.insert(3, (String::from("push"), vec![Operand::String(root.clone())]));
            //list.insert(4, (String::from("reg_write_value"), vec![]));

        }
    }

    reg.set_value(value, &data).unwrap();


    //Ok(Some(inverse))
}

pub fn delete_reg_value(inverses: Option<& mut Vec<String>>, root: &str, key: &str, value: &str) {

    //let root = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    //let key = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    //let value = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));

    let reg = registry::Hive::from(&RootKey::from(root)).open(key, Security::AllAccess).unwrap();

    let old_value = reg.value(value).unwrap();

    reg.delete_value(value).unwrap();

    if let Some(list) = inverses {

        //list.insert(0, (String::from("push"), vec![Operand::try_from(old_value).unwrap()]));
        //list.insert(1, (String::from("push"), vec![Operand::String(value.clone())]));
        //list.insert(2, (String::from("push"), vec![Operand::String(key.clone())]));
        //list.insert(3, (String::from("push"), vec![Operand::String(root.clone())]));
        //list.insert(4, (String::from("reg_write_value"), vec![]));
    }


}

fn recursive_recover(
    regkey: &registry::RegKey,
    rootkey: & str,
    inverses: & mut Vec<String>,
    index: & mut usize) {

    let name = regkey.to_string();
    let name = name.split_once("\\").unwrap().1;

    //list.insert(*index, (String::from("push"), vec![Operand::String(name.to_string())]));
    //list.insert(*index + 1, (String::from("push"), vec![rootkey.clone()]));
    //list.insert(*index + 2, (String::from("reg_write_key"), vec![]));

    *index = *index + 3;

    for value in regkey.values().map(|x| x.unwrap()) {

        //list.insert(*index, (String::from("push"), vec![Operand::try_from(value.data().clone()).unwrap()]));
        //list.insert(*index+1, (String::from("push"), vec![Operand::String(value.name().to_string().unwrap())]));
        //list.insert(*index+2, (String::from("push"), vec![Operand::String(name.to_string())]));
        //list.insert(*index + 3, (String::from("push"), vec![rootkey.clone()]));
        //list.insert(*index + 4, (String::from("reg_write_value"), vec![]));

        *index = *index + 5;

    }

    for key in regkey.keys().map(|x| x.unwrap()) {
        recursive_recover(&key.open(Security::Read).unwrap(), rootkey, inverses, index);
    }
}

pub fn delete_reg_key(inverses: Option<& mut Vec<String>>, root: &str, key: &str) {


    //let root = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));
    //let key = String::try_from(machine.operand_pop()).unwrap_or_else(|_| panic!("Oak Script Error: Could not convert argument to string"));

    let reg = registry::Hive::from(&RootKey::from(root)).open(key, Security::AllAccess).unwrap();


    if let Some(list) = inverses {
        let mut index = 0;
        recursive_recover(&reg, root, list, & mut index);
    }

    reg.delete("", true).unwrap(); //Delete the contents of the key
    reg.delete_self(false).unwrap(); //Delete the key itself





}