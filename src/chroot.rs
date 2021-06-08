use std::error::Error;
use std::path::PathBuf;
use std::process::{Command, Stdio};

//#[derive(Clone, Debug)]
//pub struct Chroot {
//    path: PathBuf,
//}

pub fn create(path: PathBuf) -> Result<(), Box<dyn Error>> {
    // check if devtools are installed
    // check if path exists (create if not, retreat if path/root exists)

    // mkarchroot $CHROOT/root base-devel

    Ok(())
}

pub fn update(path: PathBuf) -> Result<(), Box<dyn Error>> {
    println!("[Updating chroot]");
    // change string into a path & check it
    let mut path = path.to_str().unwrap().to_string();
    if path.contains('~') {
        path = shellexpand::tilde(&path).into_owned();
    }
    let _mkchrtpkg = Command::new("sudo")
        .arg("arch-nspawn")
        .arg(format!("{}/root", path))
        .arg("pacman")
        .arg("-Syu")
        .arg("--noprogressbar")
        .arg("--noconfirm")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute process");
    // TODO: add actual result check & error handling
    Ok(())
}
