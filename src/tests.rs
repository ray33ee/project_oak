
mod tests {
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;
    use crate::hlc;

    fn generic_test<S, I, U>(mut source_function: S, mut installer_validator: I, mut uninstaller_validator: U)
    where
        S: FnMut(&Path) -> String,
        I: FnMut(&Path) -> (),
        U: FnMut(&Path) -> (),

    {
        let working = TempDir::new().unwrap();
        let working_path = working.path();

        //Closure to create code
        let source_code = source_function(working_path);

        let source_path = working_path.join("source");

        let mut source = std::fs::File::create(&source_path).unwrap();

        source.write_all(source_code.as_bytes()).unwrap();

        let installer_path = working_path.join("installer");

        hlc::create_installer(source_path.as_path(), installer_path.as_path()).unwrap();

        let uninstaller_path = working_path.join("uninstaller");

        hlc::install(installer_path.as_path(), uninstaller_path.as_path());

        //Closure to perform installer tests
        installer_validator(working_path);

        hlc::uninstall(uninstaller_path.as_path());

        //Closure to perform uninstaller tests
        uninstaller_validator(working_path);
    }
/*
    #[test]
    fn instruction_data() {

        let file_data = "this is some


        random data to load into the file.";

        generic_test(|working_path| {

            let data_path = working_path.join("test_data");

            let output_path = working_path.join("extracted");

            let mut data_file = std::fs::File::create(data_path.as_path()).unwrap();

            data_file.write_all(file_data.as_bytes()).unwrap();

            format!("
.main
    push p\"{}\"
    data p\"{}\"", output_path.to_string_lossy().as_ref(), data_path.to_string_lossy().as_ref())
        }, |working_path|{

            assert!(working_path.join("extracted").exists());

            assert_eq!(std::fs::read_to_string(working_path.join("extracted")).unwrap().as_str(), file_data);

        }, |working_path|{
            assert!(!working_path.join("extracted").exists());
        });

    }
*/

    #[test]
    fn instruction_delete() {

        let file_data = "this is some


        random data to load into the file.";

        generic_test(|working_path| {

            let sample_path = working_path.join("sample");

            let mut data_file = std::fs::File::create(sample_path.as_path()).unwrap();

            data_file.write_all(file_data.as_bytes()).unwrap();

            format!("
    __delete(pathtype.absolute({:?}))
", sample_path)

        }, |working_path|{

            assert!(!working_path.join("sample").exists());

        }, |working_path|{
            assert!(working_path.join("sample").exists());

            assert_eq!(std::fs::read_to_string(working_path.join("sample")).unwrap().as_str(), file_data);
        });

    }



    #[test]
    fn instruction_move() {

        let file_data = "this is some


        random data to load into the file.";

        generic_test(|working_path| {

            let from_path = working_path.join("from");

            let to_path = working_path.join("to");

            let mut data_file = std::fs::File::create(from_path.as_path()).unwrap();

            data_file.write_all(file_data.as_bytes()).unwrap();

            println!("{}", format!("


    __move(pathtype.absolute({:?}), pathtype.absolute({:?}))


", from_path, to_path));

            format!("


    __move(pathtype.absolute({:?}), pathtype.absolute({:?}))


", from_path, to_path)
        }, |working_path|{

            assert!(!working_path.join("from").exists());
            assert!(working_path.join("to").exists());

            assert_eq!(std::fs::read_to_string(working_path.join("to")).unwrap().as_str(), file_data);

        }, |working_path|{
            assert!(working_path.join("from").exists());
            assert!(!working_path.join("to").exists());

            assert_eq!(std::fs::read_to_string(working_path.join("from")).unwrap().as_str(), file_data);
        });

    }



    #[test]
    fn instruction_copy() {

        let file_data = "this is some


        random data to load into the file.";

        generic_test(|working_path| {

            let from_path = working_path.join("from");

            let to_path = working_path.join("to");

            let mut data_file = std::fs::File::create(from_path.as_path()).unwrap();

            data_file.write_all(file_data.as_bytes()).unwrap();

            format!("
    __copy(pathtype.absolute({:?}), pathtype.absolute({:?}))
", from_path, to_path)
        }, |working_path|{

            assert!(working_path.join("from").exists());
            assert!(working_path.join("to").exists());

            assert_eq!(std::fs::read_to_string(working_path.join("to")).unwrap().as_str(), file_data);

        }, |working_path|{
            assert!(working_path.join("from").exists());
            assert!(!working_path.join("to").exists());

            assert_eq!(std::fs::read_to_string(working_path.join("from")).unwrap().as_str(), file_data);
        });

    }



    /*#[test]
    fn instruction_rename() {

        let file_data = "this is some


        random data to load into the file.";

        generic_test(|working_path| {

            let from_path = working_path.join("from");

            let to_path = working_path.join("to");

            let mut data_file = std::fs::File::create(from_path.as_path()).unwrap();

            data_file.write_all(file_data.as_bytes()).unwrap();

            format!("
.main
    push p\"{}\"
    push p\"{}\"
    rename", to_path.to_string_lossy().as_ref(), from_path.to_string_lossy().as_ref())
        }, |working_path|{

            assert!(!working_path.join("from").exists());
            assert!(working_path.join("to").exists());

            assert_eq!(std::fs::read_to_string(working_path.join("to")).unwrap().as_str(), file_data);

        }, |working_path|{
            assert!(working_path.join("from").exists());
            assert!(!working_path.join("to").exists());

            assert_eq!(std::fs::read_to_string(working_path.join("from")).unwrap().as_str(), file_data);
        });

    }*/

    #[test]
    fn instruction_zip() {

        let file_data = "this is some


        random data to load into the file.";

        generic_test(|working_path| {

            let archive = working_path.join("arch");

            let directory = working_path.join("dir");

            std::fs::create_dir(directory.as_path()).unwrap();

            let file = directory.join("file");

            let mut data_file = std::fs::File::create(file.as_path()).unwrap();

            data_file.write_all(file_data.as_bytes()).unwrap();

            format!("

    __zip(pathtype.absolute({:?}), pathtype.absolute({:?}))

", archive, directory)
        }, |working_path|{

            assert!(working_path.join("arch").exists());


        }, |working_path|{

            assert!(!working_path.join("arch").exists());


        });

    }

    #[test]
    fn instruction_unzip() {

        let file_data = "this is some


        random data to load into the file.";

        generic_test(|working_path| {

            let archive = working_path.join("arch");

            let directory = working_path.join("dir");

            std::fs::create_dir(directory.as_path()).unwrap();

            let file = directory.join("file");

            let mut data_file = std::fs::File::create(file.as_path()).unwrap();

            data_file.write_all(file_data.as_bytes()).unwrap();

            zip_extensions::zip_create_from_directory(&archive.clone(), &directory.clone()).unwrap();

            std::fs::remove_dir_all(directory.as_path()).unwrap();

            format!("

    __unzip(pathtype.absolute({:?}), pathtype.absolute({:?}))

", archive, directory)
        }, |working_path|{

            assert!(working_path.join("dir").exists());


        }, |working_path|{

            assert!(!working_path.join("dir").exists());


        });

    }



    #[test]
    fn instruction_download() {

        let file_data = "[package]
name = \"learn1\"
version = \"0.1.0\"
authors = [\"ray33ee <30669752+ray33ee@users.noreply.github.com>\"]
edition = \"2018\"

[dependencies]
num-rational = \"0.1.0\"
num-traits = \"0.2\"
regex = \"1\"
lazy_static=\"0.1.0\"
num=\"0.1.0\"
clap=\"1\"";

        generic_test(|working_path| {

            let file = working_path.join("download");

            std::fs::File::create(file.as_path()).unwrap();

            format!("


    __download(\"https://raw.githubusercontent.com/ray33ee/chembal/68b9402ff8c00e7fc041a9f95164ba0003c87d7a/Cargo.toml\", pathtype.absolute({:?}))

", file)
        }, |working_path|{

            assert!(working_path.join("download").exists());

            assert_eq!(std::fs::read_to_string(working_path.join("download")).unwrap().as_str(), file_data);

        }, |working_path|{
            assert!(!working_path.join("download").exists());
        });

    }

    #[test]
    fn instruction_edit() {

        let file_data = "file contains certain repetitions in it";
        let replaced_data = "file contaINs certaIN repetitions IN it";

        generic_test(|working_path| {

            let file_path = working_path.join("file");

            let mut file = std::fs::File::create(file_path.as_path()).unwrap();

            file.write_all(file_data.as_bytes()).unwrap();


            format!("

    __edit(pathtype.absolute({:?}), \"s/in/IN/g\")

", file_path)
        }, |working_path|{

            assert!(working_path.join("file").exists());

            assert_eq!(std::fs::read_to_string(working_path.join("file")).unwrap().as_str(), replaced_data);

        }, |working_path|{
            assert!(working_path.join("file").exists());

            assert_eq!(std::fs::read_to_string(working_path.join("file")).unwrap().as_str(), file_data);
        });

    }
/*
    #[test]
    fn instruction_reg_write_key() {



        generic_test(|_working_path| {

            format!("
.main
    push \"SOFTWARE\\key_test\"
    push \"hklm\"
    reg_write_key")
        }, |_working_path|{

            Hive::LocalMachine.open("SOFTWARE\\key_test", Security::Read).unwrap();


        }, |_working_path|{

            Hive::LocalMachine.open("SOFTWARE\\key_test", Security::Read).err().unwrap();

        });

    }

    #[test]
    fn instruction_reg_write_val() {


        let rkey = Hive::LocalMachine.open("SOFTWARE", Security::AllAccess).unwrap();

        rkey.create("val_test", Security::AllAccess).unwrap();

        generic_test(|_working_path| {


            format!("
.main
    push 100
    push \"val_name\"
    push \"SOFTWARE\\val_test\"
    push \"hklm\"
    reg_write_value")
        }, |_working_path|{

            let rkey = Hive::LocalMachine.open("SOFTWARE\\val_test", Security::AllAccess).unwrap();

            let val = rkey.value("val_name").unwrap();

            if let registry::Data::U32(n) = val {
                assert_eq!(n, 100);
            } else {
                panic!("Data not a number");
            }

        }, |_working_path|{

            let rkey = Hive::LocalMachine.open("SOFTWARE\\val_test", Security::AllAccess).unwrap();


            match rkey.value("val_name") {
                Ok(_) => {
                    rkey.delete_self(false).unwrap();

                    panic!("Uninstall failed. val_name value still exists")
                }
                Err(_) => {
                    rkey.delete_self(false).unwrap();
                }
            }


        });

    }

    #[test]
    fn instruction_reg_write_val2() {


        let rkey = Hive::LocalMachine.open("SOFTWARE", Security::AllAccess).unwrap();

        let bkey = rkey.create("val_test2", Security::AllAccess).unwrap();

        bkey.set_value("val_name", &registry::Data::U32(400)).unwrap();

        generic_test(|_working_path| {


            format!("
.main
    push 100
    push \"val_name\"
    push \"SOFTWARE\\val_test2\"
    push \"hklm\"
    reg_write_value")
        }, |_working_path|{

            let rkey = Hive::LocalMachine.open("SOFTWARE\\val_test2", Security::AllAccess).unwrap();

            let val = rkey.value("val_name").unwrap();

            if let registry::Data::U32(n) = val {
                assert_eq!(n, 100);
            } else {
                panic!("Data not a number");
            }

        }, |_working_path|{

            let rkey = Hive::LocalMachine.open("SOFTWARE\\val_test2", Security::AllAccess).unwrap();


            let val = rkey.value("val_name").unwrap();

            if let registry::Data::U32(n) = val {
                assert_eq!(n, 400);
            } else {
                panic!("Data not a number");
            }


        });

        bkey.delete("", true).unwrap();
        bkey.delete_self(false).unwrap();
    }

    #[test]
    fn instruction_reg_delete_val() {


        let rkey = Hive::LocalMachine.open("SOFTWARE", Security::AllAccess).unwrap();

        let bkey = rkey.create("val_test_delete", Security::AllAccess).unwrap();

        bkey.set_value("f", &registry::Data::U32(400)).unwrap();

        generic_test(|_working_path| {


            format!("
.main
    push \"f\"
    push \"SOFTWARE\\val_test_delete\"
    push \"hklm\"
    reg_delete_value")
        }, |_working_path|{
            let rkey = Hive::LocalMachine.open("SOFTWARE\\val_test_delete", Security::AllAccess).unwrap();

            assert!(rkey.value("f").is_err())


        }, |_working_path|{


            let rkey = Hive::LocalMachine.open("SOFTWARE\\val_test_delete", Security::AllAccess).unwrap();

            let val = rkey.value("f").unwrap();

            if let registry::Data::U32(n) = val {
                assert_eq!(n, 400);
            } else {
                panic!("Data not a number");
            }


        });

        bkey.delete("", true).unwrap();
        bkey.delete_self(false).unwrap();

    }


    #[test]
    fn instruction_reg_delete_key() {


        let rkey = Hive::LocalMachine.open("SOFTWARE", Security::AllAccess).unwrap();

        let rkey = rkey.create("instruction_reg_delete_key", Security::AllAccess).unwrap();

        let rkey = rkey.create("inner", Security::AllAccess).unwrap();

        rkey.set_value("val", &registry::Data::U32(500)).unwrap();

        generic_test(|_working_path| {

            format!("
.main
    push \"SOFTWARE\\instruction_reg_delete_key\"
    push \"hklm\"
    reg_delete_key")
        }, |_working_path|{

            Hive::LocalMachine.open("SOFTWARE\\instruction_reg_delete_key\\inner", Security::Read).err().unwrap();

        }, |_working_path|{

            let key = Hive::LocalMachine.open("SOFTWARE\\instruction_reg_delete_key\\inner", Security::Read).unwrap();

            if let registry::Data::U32(500) = key.value("val").unwrap() {

            } else {
                panic!("Invalid registry data");
            }

        });

    }

*/

    /*
    #[test]
    fn unwind_test() {

    }
    */

    #[test]
    fn print() {

        generic_test(|working_path| {


            format!("

path = \"E:\\\\Software Projects\\\\IntelliJ\\\\project_oak\\\\tmp\\\\file\\\\wpa_supplicant - Copy.conf\"

__delete(pathtype.absolute(path))



            ")


        }, |working_path|{

            assert!(!PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\file\\wpa_supplicant - Copy.conf").exists())

        }, |working_path|{

            assert!(PathBuf::from("E:\\Software Projects\\IntelliJ\\project_oak\\tmp\\file\\wpa_supplicant - Copy.conf").exists())
        });

    }

}