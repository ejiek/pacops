use crate::pkgbuild::HashAlg;

use std::error::Error;
use std::fs::File;

use blake2::{Blake2b, Blake2s, Digest};
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha224, Sha256, Sha384, Sha512};
use tempfile::Builder;

#[derive(Clone)]
pub struct Update {
    pub version: String,
    pub source_index: usize,
    pub url: String,
}

impl Update {
    pub fn hash(&self, hash_alg: HashAlg) -> Result<String, Box<dyn Error>> {
        let tmp_dir = Builder::new().prefix("example").tempdir()?;
        println!("New source: '{}'", &self.url);
        let client = reqwest::blocking::Client::builder().timeout(None).build()?;
        let resp = client.get(&self.url).send()?;
        let _dest = {
            let fname = resp
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap_or("tmp.bin");

            println!("file to download: '{}'", fname);
            let fname = tmp_dir.path().join(fname);
            println!("will be located under: '{:?}'", fname);
            File::create(fname)?
        };
        let content = resp.bytes()?;
        //copy(&mut content.as_bytes(), &mut dest)?;
        match hash_alg {
            HashAlg::B2 => {
                let mut hasher = Blake2b::new();
                hasher.update(content);
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlg::SHA1 => {
                let mut hasher = Sha1::new();
                hasher.update(content);
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlg::SHA224 => {
                let mut hasher = Sha224::new();
                hasher.update(content);
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlg::SHA256 => {
                let mut hasher = Sha256::new();
                hasher.update(content);
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlg::SHA384 => {
                let mut hasher = Sha384::new();
                hasher.update(content);
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlg::SHA512 => {
                let mut hasher = Sha512::new();
                hasher.update(content);
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlg::MD5 => {
                let mut hasher = Md5::new();
                hasher.update(content);
                Ok(format!("{:x}", hasher.finalize()))
            }
        }
    }
}
