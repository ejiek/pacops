use std::cell::RefCell;
use std::error::Error;
use std::path::PathBuf;
use std::rc::Rc;

use crate::config::Config;
use crate::pkgbuild::Pkgbuild;
use crate::update::Update;

// context for working with a particular PKGBUILD
pub struct Context {
    config: Rc<Config>,
    pkgbuild: Option<Rc<RefCell<Pkgbuild>>>,
    pkgbuild_path: Option<PathBuf>,
    pkgname: Option<String>,
    current_version: String,
    update: Option<Update>,
}

impl Context {
    pub fn new(config: Rc<Config>) -> Context {
        Context {
            config: config,
            pkgbuild: None,
            pkgbuild_path: None,
            update: None,
            pkgname: None,
            current_version: String::from(""),
        }
    }

    pub fn set_config(mut self, config: Rc<Config>) -> Self {
        self.config = config;
        self
    }

    pub fn set_pkgbuild(mut self, pkgbuild: Rc<RefCell<Pkgbuild>>) -> Self {
        self.pkgbuild = Some(pkgbuild);
        self
    }

    pub fn set_pkgbuild_path(mut self, path: PathBuf) -> Self {
        self.pkgbuild_path = Some(path);
        self
    }

    pub fn set_update(mut self, update: &Update) -> Self {
        self.update = Some(update.clone());
        self
    }

    pub fn set_current_version(mut self, current_version: String) -> Self {
        self.current_version = current_version;
        self
    }

    pub fn set_pkgname(mut self, pkgname: String) -> Self {
        self.pkgname = Some(pkgname);
        self
    }

    pub fn config(&self) -> Rc<Config> {
        self.config.clone()
    }

    pub fn pkgbuild(&self) -> Option<Rc<RefCell<Pkgbuild>>> {
        self.pkgbuild.clone()
    }

    pub fn pkgbuild_path(&self) -> Option<PathBuf> {
        self.pkgbuild_path.clone()
    }

    pub fn shellexpand_context(&self) -> Box<dyn Fn(&str) -> Result<Option<String>, String>> {
        let update = self.update.clone().unwrap();
        let current_version = self.current_version.clone();
        let pkgname = self.pkgname.clone();
        Box::new(move |s| match s {
            "pkgname" => Ok(Some(pkgname.as_ref().unwrap().clone())),
            "old_version" => {
                Ok(Some(current_version.clone())) // too many clones
            }
            "new_version" => Ok(Some(update.version.clone())),
            _ => Ok(None),
        })
    }
}
