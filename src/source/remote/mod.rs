use std::error::Error;

use super::Source;
use crate::update::Update;

mod deb;

#[derive(Debug, Clone)]
pub enum Remote {
    Deb,
    GithubRelease,
    Unknown,
}

impl Remote {
    pub fn guess(source: String) -> Remote {
        if source.starts_with("http://") || source.starts_with("https://") {
            if source.contains("github.com") && source.contains("/archive/") {
                return Remote::GithubRelease;
            } else if source.ends_with(".deb") {
                return Remote::Deb;
            }
        }
        Remote::Unknown
    }

    pub fn latest(&self, source: &Source) -> Result<Update, Box<dyn Error>> {
        match self {
            Self::Deb => {
                if let Some(latest) = deb::latest(source)? {
                    Ok(latest)
                } else {
                    let error: Box<dyn Error> =
                        String::from("Unable to find any remote versions").into();
                    Err(error)
                }
            }
            Self::GithubRelease => {
                let error: Box<dyn Error> =
                    String::from("Github Release parsing is not implemented yet").into();
                Err(error)
            }
            Self::Unknown => {
                let error: Box<dyn Error> =
                    String::from("Unknown source type, unable to check updates").into();
                Err(error)
            }
        }
    }
}
