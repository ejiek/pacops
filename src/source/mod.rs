use version_compare::{CompOp, VersionCompare};

use std::cell::RefCell;
use std::rc::Weak;

use crate::context::Context;
use crate::pkgbuild::Pkgbuild;
use crate::update::Update;

mod remote;

pub struct Source {
    raw: String,
    origin: Origin,
    pkgbuild: Weak<RefCell<Pkgbuild>>,
    pub index: usize,
}

impl Source {
    pub fn new(
        raw: String,
        origin: Origin,
        pkgbuild: Weak<RefCell<Pkgbuild>>,
        index: usize,
    ) -> Source {
        Source {
            raw,
            origin,
            pkgbuild,
            index,
        }
    }
    pub fn origin(&self) -> Origin {
        self.origin.clone()
    }

    pub fn raw(&self) -> String {
        self.raw.clone()
    }

    pub fn update_available(&self) -> Result<Option<Update>, Box<dyn std::error::Error>> {
        match &self.origin {
            Origin::Local => Ok(None),
            Origin::Remote(remote) => {
                //get latest
                let latest = remote.latest(self)?;
                let current;
                if self.raw.contains("${pkgver}") {
                    current = self
                        .pkgbuild
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .version()
                        .as_ref()
                        .unwrap()
                        .clone();
                } else {
                    let error: Box<dyn std::error::Error> =
                        String::from("Unable to extract current version of a source").into();
                    return Err(error);
                }
                if VersionCompare::compare_to(&latest.version, &current, &CompOp::Gt).unwrap() {
                    Ok(Some(latest))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Origin {
    Local,
    Remote(remote::Remote),
}

impl Origin {
    pub fn guess(source: String) -> Origin {
        // TODO: How is it done in libalpm?
        if source.contains("http://") || source.contains("https://") {
            let remote = remote::Remote::guess(source);
            return Origin::Remote(remote);
        }
        Origin::Local
    }
}
