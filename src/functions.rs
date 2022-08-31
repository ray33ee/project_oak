use std::path::{Path};
use registry::Security;
use tempfile::TempDir;
use crate::{OakRead, OakWrite};
use crate::registry_ex::{Data, RootKey};
use crate::path_type::{Inverse, PathType};
use crate::error::{Error, Result};
use crate::mlc::data_to_code;

pub fn data(installer: & OakRead, inverses: Option<& Inverse>, name: & str, destination: &PathType, temp: & TempDir) -> Result<()>  {

    let destination_path = destination.to_absolute_path(temp);

    installer.extract(name, &destination_path)?;

    if !destination.is_temp() {
        if let Some(list) = inverses {
            //list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(destination_path.as_path().to_path_buf()))]));
            //list.insert(1, (String::from("delete"), vec![]));


            list.insert(0, format!("__delete(pathtype.absolute({:?}))", destination_path));

        }
    }

    Ok(())
}

pub fn _move(inverses: Option<& Inverse>, source: & PathType, destination: & PathType, temp: & TempDir) -> Result<()> {

    let d = destination;

    let destination = destination.to_absolute_path(temp);

    let source_path = source.to_absolute_path(temp);

    if destination.exists() {
        return Err(Error::AlreadyExists)
    } else {
        if source_path.is_dir() {
            let mut options = fs_extra::dir::CopyOptions::default();
            options.content_only = true;

            fs_extra::dir::move_dir(&source_path, &destination, &options)?;

        } else if source_path.is_file() {
            let options = fs_extra::file::CopyOptions::default();

            fs_extra::file::move_file(&source_path, &destination, &options)?;

        } else {
            return Err(Error::DoesntExist);
        }

        if !d.is_temp() {
            if let Some(list) = inverses {
                match &source {
                    PathType::Absolute(s) => {


                        //list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(s.clone()))]));
                        //list.insert(1, (String::from("push"), vec![Operand::Path(PathType::Absolute(destination))]));
                        //list.insert(2, (String::from("move"), vec![]));

                        //Ok(Some(Step::Move { source: SpecialPath::Path(destination.clone()), destination: s.clone() }))

                        list.insert(0, format!("__move(pathtype.absolute({:?}), pathtype.absolute({:?}))", destination, s));
                    }
                    PathType::Temporary(_) => {


                        //list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(destination))]));
                        //list.insert(1, (String::from("delete"), vec![]));

                        //Ok(Some(Step::Delete { path: destination.clone() }))

                        list.insert(0, format!("__delete(pathtype.absolute({:?}))", destination));
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn delete(mut uninstaller: Option<& OakWrite>, inverses: Option<&Inverse>, path: & PathType, temp: & TempDir) -> Result<()> {

    let path = path.to_absolute_path(temp);

    let name = if path.exists() {
        /*match uninstaller.as_mut() {
            None => { None }
            Some(archive) => { Some(archive.archive(&path)) }
        };*/

        uninstaller.as_mut().map(|archive| archive.archive(&path))
    } else {
        return Err(Error::DoesntExist);
    };

    if path.is_dir() {
        std::fs::remove_dir_all(&path)?;
    } else if path.is_file() {
        std::fs::remove_file(&path)?;
    } else {
        panic!("Path is not a directory or file")
    };

    if let Some(list) = inverses {

        //list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(path.clone()))]));
        //list.insert(1, (String::from("data"), vec![Operand::String(name.unwrap())]));


        list.insert(0, format!("__data({:?}, pathtype.absolute({:?}))", name.unwrap(), path))

    }


    Ok(())
}

pub fn copy(inverses: Option<&  Inverse>, source: &PathType, destination: &PathType, temp: & TempDir) -> Result<()> {

    let source_path = source.to_absolute_path(temp);
    let destination_path = destination.to_absolute_path(temp);

    if destination_path.exists() {
        return Err(Error::AlreadyExists);
    } else {
        if source_path.is_file() {

            std::fs::copy(&source_path, &destination_path)?;
        } else if source_path.is_dir() {
            let mut options = fs_extra::dir::CopyOptions::default();
            options.content_only = true;

            fs_extra::dir::copy(&source_path, &destination_path, &options)?;
        } else {
            panic!("Source is not a file or directory");
        }

        if !destination.is_temp() {

            if let Some(list) = inverses {


                //list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(destination_path.as_path().to_path_buf()))]));
                //list.insert(1, (String::from("delete"), vec![]));

                list.insert(0, format!("__delete(pathtype.absolute({:?}))", destination_path));

            }


        }
    }

    Ok(())


}


pub fn create(inverses: Option<& Inverse>, path: PathType, temp: & TempDir) -> Result<()>  {

    let abs_path = path.to_absolute_path(temp);

    std::fs::File::create(&abs_path)?;

    if !path.is_temp() {
        if let Some(list) = inverses {

            //list.insert(0, (String::from("push"), vec![Operand::Path(archive)]));
            //list.insert(1, (String::from("delete"), vec![]));

            list.insert(0, format!("__delete(pathtype.absolute({:?}))", abs_path));
        }
    }

    Ok(())
}


pub fn mkdir(inverses: Option<& Inverse>, path: PathType, temp: & TempDir) -> Result<()>  {

    let abs_path = path.to_absolute_path(temp);

    std::fs::create_dir(&abs_path)?;

    if !path.is_temp() {
        if let Some(list) = inverses {

            //list.insert(0, (String::from("push"), vec![Operand::Path(archive)]));
            //list.insert(1, (String::from("delete"), vec![]));

            list.insert(0, format!("__delete(pathtype.absolute({:?}))", abs_path));
        }
    }

    Ok(())
}






pub fn zip(inverses: Option<& Inverse>, archive: &PathType, folder: &PathType, temp: & TempDir) -> Result<()>  {

    let archive_path = archive.to_absolute_path(temp);

    zip_extensions::write::zip_create_from_directory(&archive_path, &folder.to_absolute_path(temp))?;

    if !archive.is_temp() {

        if let Some(list) = inverses {

            //list.insert(0, (String::from("push"), vec![Operand::Path(archive)]));
            //list.insert(1, (String::from("delete"), vec![]));

            list.insert(0, format!("__delete(pathtype.absolute({:?}))", archive_path));
        }
    }

    Ok(())
}

pub fn unzip(inverses: Option<& Inverse>, archive: &PathType, folder: &PathType, temp: & TempDir) -> Result<()>  {

    let folder_path = folder.to_absolute_path(temp);

    //std::fs::create_dir(&folder.path(temp)).unwrap();

    zip_extensions::read::zip_extract(&archive.to_absolute_path(temp), &folder_path)?;

    if !archive.is_temp() {
        //Ok(Some(Step::Delete { path: folder.path(temp) }))

        if let Some(list) = inverses {

            //list.insert(0, (String::from("push"), vec![Operand::Path(folder.clone())]));
            //list.insert(1, (String::from("delete"), vec![]));

            list.insert(0, format!("__delete(pathtype.absolute({:?}))", folder_path));
        }


    }

    Ok(())
}


pub fn download(inverses: Option<& Inverse>, url: & str, destination: &PathType, temp: & TempDir) -> Result<String>  {

    println!("url: {}", url);

    let response = reqwest::blocking::get(url)?;

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




    let mut dest = std::fs::File::create(file_name.clone())?;

    if !destination.is_temp() {

        if let Some(list) = inverses {

            //list.insert(0, (String::from("push"), vec![Operand::Path(PathType::Absolute(file_name.clone()))]));
            //list.insert(1, (String::from("delete"), vec![]));

            list.insert(0, format!("__delete(pathtype.absolute({:?}))", file_name));
        }
    }

    let content = response.text()?;
    std::io::copy(&mut content.as_bytes(), &mut dest)?;

    Ok(file_name.to_str().unwrap().to_string())
}

pub fn edit(uninstaller: Option<& OakWrite>, inverses: Option<& Inverse>, s: &PathType, command: & str, temp: & TempDir) -> Result<()>  {


    use std::io::Write;

    let source = s.to_absolute_path(temp);


    //Load `source`
    let content = std::fs::read_to_string(source.as_path())?;


    //Perform find and replace
    let res = sedregex::find_and_replace(content.as_str(), &[command])?;


    match uninstaller {
        None => {  }
        Some(archive) => {

            let name = archive.archive(&source);

            if !s.is_temp() {


                if let Some(list) = inverses {
                    //list.insert(0, (String::from("push"), vec![Operand::Path(s.clone())]));
                    //list.insert(1, (String::from("delete"), vec![]));

                    list.insert(0, format!("__delete(pathtype.absolute({:?}))", source));

                    //list.insert(2, (String::from("push"), vec![Operand::Path(s.clone())]));
                    //list.insert(3, (String::from("data"), vec![Operand::String(name)]));

                    list.insert(1, format!("__data({:?}, pathtype.absolute({:?}))", name, source))

                }

            }


        }
    }

    //Save back to `source`
    let mut fh = std::fs::OpenOptions::new().write(true).open(source.as_path())?;

    fh.write_all(res.as_ref().as_bytes())?;

    Ok(())

}


pub fn write_reg_key(inverses: Option<& Inverse>, root: & RootKey, key: & str) -> Result<()>  {

    let reg = registry::Hive::from(root); //.open(key, Security::AllAccess)?;



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

    reg.create(key, Security::AllAccess)?;


    if let Some(p) = common {
        if let Some(list) = inverses {
            //list.insert(0, (String::from("push"), vec![Operand::String(String::from(p.to_str().unwrap()))]));
            //list.insert(1, (String::from("push"), vec![Operand::String(root.clone())]));
            //list.insert(2, (String::from("reg_delete_key"), vec![]));


            list.insert(0, format!("__reg_delete_key(\"{:?}\", {:?})", root, p));

        }
    }

    Ok(())
}


pub fn write_reg_value(inverses: Option<& Inverse>, root: &RootKey, key: &str, value: &str, data: &registry::Data) -> Result<()>  {

    let reg = registry::Hive::from(root).open(key, Security::AllAccess)?;

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

            list.insert(0, format!("__reg_delete_value(\"{:?}\", {:?}, {:?})", root, key, value));

        } else {

            let old_value = reg.value(value).unwrap();

            //list.insert(0, (String::from("push"), vec![Operand::try_from(old_value).unwrap()]));
            //list.insert(1, (String::from("push"), vec![Operand::String(value.clone())]));
            //list.insert(2, (String::from("push"), vec![Operand::String(key.clone())]));
            //list.insert(3, (String::from("push"), vec![Operand::String(root.clone())]));
            //list.insert(4, (String::from("reg_write_value"), vec![]));

            //todo!()

            list.insert(0, format!("__reg_write_value(\"{:?}\", {:?}, {:?}, {})", root, key, value, data_to_code(&Data::from(old_value))));
        }
    }

    reg.set_value(value, &data)?;


    //Ok(Some(inverse))

    Ok(())
}

pub fn delete_reg_value(inverses: Option<& Inverse>, root: &RootKey, key: &str, value: &str) -> Result<()>  {

    let reg = registry::Hive::from(root).open(key, Security::AllAccess)?;

    let old_value = reg.value(value).unwrap();

    reg.delete_value(value)?;

    if let Some(list) = inverses {

        //list.insert(0, (String::from("push"), vec![Operand::try_from(old_value).unwrap()]));
        //list.insert(1, (String::from("push"), vec![Operand::String(value.clone())]));
        //list.insert(2, (String::from("push"), vec![Operand::String(key.clone())]));
        //list.insert(3, (String::from("push"), vec![Operand::String(root.clone())]));
        //list.insert(4, (String::from("reg_write_value"), vec![]));


        list.insert(0, format!("__reg_write_value(\"{:?}\", {:?}, {:?}, {})", root, key, value, data_to_code(&Data::from(old_value))));

    }


    Ok(())

}

fn recursive_recover(
    regkey: &registry::RegKey,
    rootkey: & RootKey,
    list: & Inverse,
    index: & mut usize) {

    let name = regkey.to_string();
    let name = name.split_once("\\").unwrap().1;

    //list.insert(*index, (String::from("push"), vec![Operand::String(name.to_string())]));
    //list.insert(*index + 1, (String::from("push"), vec![rootkey.clone()]));
    //list.insert(*index + 2, (String::from("reg_write_key"), vec![]));

    list.insert(*index, format!("__reg_write_key(\"{:?}\", {:?})", rootkey, name));

    *index = *index + 1;

    for value in regkey.values().map(|x| x.unwrap()) {

        //list.insert(*index, (String::from("push"), vec![Operand::try_from(value.data().clone()).unwrap()]));
        //list.insert(*index+1, (String::from("push"), vec![Operand::String(value.name().to_string().unwrap())]));
        //list.insert(*index+2, (String::from("push"), vec![Operand::String(name.to_string())]));
        //list.insert(*index + 3, (String::from("push"), vec![rootkey.clone()]));
        //list.insert(*index + 4, (String::from("reg_write_value"), vec![]));

        list.insert(*index, format!("__reg_write_value(\"{:?}\", {:?}, {:?}, {})", rootkey, name, value.name().to_string().unwrap(), data_to_code(&Data::from(value.data().clone()))));

        *index = *index + 1;

    }

    for key in regkey.keys().map(|x| x.unwrap()) {
        recursive_recover(&key.open(Security::Read).unwrap(), rootkey, list, index);
    }
}

pub fn delete_reg_key(inverses: Option<& Inverse>, root: &RootKey, key: &str) -> Result<()>  {

    let reg = registry::Hive::from(root).open(key, Security::AllAccess)?;


    if let Some(list) = inverses {
        let mut index = 0;
        recursive_recover(&reg, root, list, & mut index);
    }

    reg.delete("", true)?; //Delete the contents of the key
    reg.delete_self(false)?; //Delete the key itself



    Ok(())



}