use crate::context::Context;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

pub fn repo_root(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let git = Command::new("git")
        .current_dir(path.to_str().unwrap())
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .expect("failed to execute process");
    println!("{}", str::from_utf8(&git.stderr).unwrap());
    let repo_path = PathBuf::from(str::from_utf8(&git.stdout).unwrap());

    //file.write_all(stdout)?;
    Ok(repo_path)
}

pub fn commit(context: &Context) -> Result<(), Box<dyn Error>> {
    let message_template = context.config().commit_message();
    let pkgbuild_path = context.pkgbuild_path().unwrap();
    let path = pkgbuild_path.parent().unwrap();
    let message = shellexpand::env_with_context(&message_template, &context.shellexpand_context())
        .unwrap()
        .into_owned();

    // check if staging is empty
    // add PKGBUILD & .SRCINFO if needed
    let git = Command::new("git")
        .current_dir(path.to_str().unwrap())
        .arg("commit")
        .arg("-m")
        .arg(message)
        .arg("PKGBUILD")
        .arg(".SRCINFO")
        .output()
        .expect("failed to execute process");
    println!("{}", str::from_utf8(&git.stdout).unwrap());
    println!("{}", str::from_utf8(&git.stderr).unwrap());
    Ok(())
}
