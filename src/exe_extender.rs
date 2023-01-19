use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write, Read};
use std::path::Path;

///Get total length of the exe, and the archive offset
pub fn get_meta() -> (u64, u64) {

    let exe_path = std::env::current_exe().unwrap();

    let mut fh = OpenOptions::new().read(true).open(&exe_path).unwrap();

    let length = fh.metadata().unwrap().len();

    fh.seek(SeekFrom::End(-8)).unwrap();

    let mut v = [0u8; 8];

    fh.read_exact(& mut v).unwrap();

    (u64::from_be_bytes(v), length)
}

///Take an exe and append an archive to it
pub fn extend_exe(archive_path: &Path, new_exe: &Path, length: u64) {

    let exe_path = std::env::current_exe().unwrap();

    std::fs::copy(&exe_path, new_exe).unwrap();

    let mut exe = OpenOptions::new().write(true).append(true).open(new_exe).unwrap();

    let mut archive = OpenOptions::new().read(true).open(archive_path).unwrap();

    std::io::copy(& mut archive, & mut exe).unwrap();

    exe.write_all(length.to_be_bytes().as_ref()).unwrap();
}

///Take an appended exe and get the archive
pub fn get_archive(archive: &Path, length: u64, offset: u64) {


    let exe_path = std::env::current_exe().unwrap();

    let mut exe = OpenOptions::new().read(true).open(exe_path.as_path()).unwrap();



    exe.seek(SeekFrom::Start(offset)).unwrap();
    let mut t = exe.take(length - offset - 8);


    {let mut archive = OpenOptions::new().create(true).write(true).open(&archive).unwrap();

    std::io::copy(& mut t, & mut archive).unwrap();}

}

