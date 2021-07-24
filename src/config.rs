use serde::Deserialize;
extern crate dirs;

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

// Layered config
#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    build: Option<Build>,
    chroot: Option<PathBuf>,
    commit: Option<bool>,
    commit_message: Option<String>,
    #[serde(skip_deserializing)]
    parent: Option<Rc<Config>>,
    srcinfo: Option<bool>,
}

impl Config {
    pub fn defaults() -> Config {
        Config {
            build: Some(Build::Local),
            chroot: None, // Look for one in XDG_something
            commit: Some(false),
            commit_message: Some(String::from("${pkgname}: ${old_version} -> ${new_version}")),
            parent: None,
            srcinfo: Some(false),
        }
    }
    pub fn global_config_lookup() -> Result<Config, Box<dyn Error>> {
        if let Some(mut config_file) = dirs::config_dir() {
            config_file.push("pacops.toml");
            return Config::read_from_file(config_file);
        }

        let error: Box<dyn std::error::Error> =
            String::from("Unable locate configuration directory").into();
        return Err(error);
    }

    pub fn read_from_file(path: PathBuf) -> Result<Config, Box<dyn Error>> {
        let contents = fs::read_to_string(path)?;

        let decoded: Config = toml::from_str(&contents)?;
        return Ok(decoded);
    }

    pub fn set_parent_rc(&mut self, parent: Rc<Config>) {
        self.parent = Some(parent);
    }

    pub fn set_chroot(&mut self, chroot: PathBuf) {
        self.chroot = Some(chroot);
    }

    pub fn set_commit(&mut self, commit: bool) {
        self.commit = Some(commit);
    }

    pub fn set_srcinfo(&mut self, srcinfo: bool) {
        self.srcinfo = Some(srcinfo);
    }

    pub fn set_build(&mut self, build: Build) {
        self.build = Some(build);
    }

    // Look into dedup of get functions
    pub fn chroot(&self) -> Option<PathBuf> {
        if let Some(actual_chroot) = self.chroot.clone() {
            return Some(actual_chroot);
        }
        if let Some(actual_parent) = self.parent.clone() {
            return actual_parent.chroot();
        }
        None
    }

    pub fn commit(&self) -> Option<bool> {
        if let Some(actual_commit) = self.commit.clone() {
            return Some(actual_commit);
        }
        if let Some(actual_parent) = self.parent.clone() {
            return actual_parent.commit();
        }
        None
    }

    pub fn commit_message(&self) -> Option<String> {
        if let Some(actual_commit_message) = self.commit_message.clone() {
            return Some(actual_commit_message);
        }
        if let Some(actual_parent) = self.parent.clone() {
            return actual_parent.commit_message();
        }
        None
    }

    pub fn build_type(&self) -> Option<Build> {
        if let Some(actual_build) = self.build.clone() {
            return Some(actual_build);
        }
        if let Some(actual_parent) = self.parent.clone() {
            return actual_parent.build_type();
        }
        None
    }

    pub fn srcinfo(&self) -> Option<bool> {
        if let Some(actual_srcinfo) = self.srcinfo.clone() {
            return Some(actual_srcinfo);
        }
        if let Some(actual_parent) = self.parent.clone() {
            return actual_parent.srcinfo();
        }
        None
    }
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        self.build
            .as_ref()
            .unwrap()
            .eq(&other.build.as_ref().unwrap())
            && self.chroot == other.chroot
            && self.commit_message == other.commit_message
    }
}

#[derive(Clone, Deserialize, PartialEq, Debug)]
pub enum Build {
    Local,
    Chroot,
    Docker,
    Podman,
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let config_string = r#"build = 'Chroot'
chroot = '~/hobby/chroot/root'"#;

        let decoded: Config = toml::from_str(config_string).unwrap();

        let created = Config {
            build: Some(Build::Chroot),
            chroot: Some(PathBuf::from("~/hobby/chroot/root")),
            commit: None,
            commit_message: None,
            parent: None,
            srcinfo: Some(false),
        };
        assert_eq!(decoded, created);
    }
}
