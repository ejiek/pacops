extern crate dirs;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;
use std::path::PathBuf;

#[derive(Clone, Deserialize, PartialEq, Debug)]
pub enum Build {
    Local,
    Chroot,
    Docker,
    Podman,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Settings {
    build: Build,
    chroot: Option<PathBuf>,
    commit: bool,
    commit_message: String,
    push: bool,
    srcinfo: bool,
}

impl Settings {
    pub fn builder(file: Option<String>) -> Result<Config, ConfigError> {
        let mut s = Config::default();

        s.set_default("build", "Local")?;
        s.set_default("commit", "false")?;
        s.set_default(
            "commit_message",
            "${pkgname}: ${old_version} -> ${new_version}",
        )?;
        s.set_default("push", "false")?;
        s.set_default("srcinfo", "false")?;

        match file {
            Some(f) => {
                s.merge(File::with_name(&f))?;
            }
            None => match env::var("PACOPS_CONFIG") {
                Ok(path) => {
                    s.merge(File::with_name(&path))?;
                }
                Err(_) => {
                    if let Some(mut config_file) = dirs::config_dir() {
                        config_file.push("pacops.toml");
                        if std::path::Path::new(&config_file).exists() {
                            s.merge(File::with_name(config_file.to_str().unwrap()))?;
                        }
                    }
                }
            },
        }

        s.merge(Environment::with_prefix("pacops"))?;

        Ok(s)
    }

    pub fn commit(&self) -> bool {
        self.commit
    }

    pub fn commit_message(&self) -> String {
        self.commit_message.clone()
    }

    pub fn push(&self) -> bool {
        self.push
    }

    pub fn srcinfo(&self) -> bool {
        self.srcinfo
    }

    pub fn build_type(&self) -> Build {
        self.build.clone()
    }

    pub fn chroot(&self) -> Option<PathBuf> {
        self.chroot.clone()
    }
}
