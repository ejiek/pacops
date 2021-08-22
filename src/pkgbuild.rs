extern crate shellexpand;

use crate::chroot;
use crate::settings::{Build, Settings};
use crate::source::{Origin, Source};
use crate::update::Update;

use std::cell::RefCell;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::rc::{Rc, Weak};
use std::str;

use blake2::{Blake2b, Blake2s, Digest};
use md5::{Digest as md5digest, Md5};
use regex::Regex;
use sha1::{Digest as sha1digest, Sha1};
use sha2::{Digest as sha2digest, Sha224, Sha256, Sha384, Sha512};
use version_compare::{CompOp, VersionCompare};

pub struct Pkgbuild {
    raw: String,
    pkgname: String,
    version: Option<String>,
    sources: Vec<Source>,
    hashsums: HashSums,
    path: Option<PathBuf>,
}

pub struct HashSums {
    hashes: Vec<String>,
    alg: HashAlg,
}

#[derive(Clone, Copy)]
pub enum HashAlg {
    B2,
    SHA1,
    SHA224,
    SHA256,
    SHA384,
    SHA512,
    MD5,
}

impl HashSums {
    fn new(line_prefix: String, hashes: Vec<String>) -> Option<HashSums> {
        match line_prefix.as_str() {
            "md5" => Some(HashSums {
                hashes,
                alg: HashAlg::MD5,
            }),
            "b2" => Some(HashSums {
                hashes,
                alg: HashAlg::B2,
            }),
            "sha1" => Some(HashSums {
                hashes,
                alg: HashAlg::SHA1,
            }),
            "sha224" => Some(HashSums {
                hashes,
                alg: HashAlg::SHA224,
            }),
            "sha256" => Some(HashSums {
                hashes,
                alg: HashAlg::SHA256,
            }),
            "sha384" => Some(HashSums {
                hashes,
                alg: HashAlg::SHA384,
            }),
            "sha512" => Some(HashSums {
                hashes,
                alg: HashAlg::SHA512,
            }),
            _ => None,
        }
    }
}

impl Pkgbuild {
    fn new(raw: String, path: Option<PathBuf>) -> Result<Rc<RefCell<Pkgbuild>>, Box<dyn Error>> {
        let sources: Vec<Source> = Vec::new();
        let raw_double = raw.clone();
        let version = Pkgbuild::parse_version(&raw);
        let pkgname = Pkgbuild::parse_pkgname(&raw).unwrap();
        let hashsums = Pkgbuild::parse_hashsums(&raw).unwrap();
        let pkgb = Rc::new(RefCell::new(Pkgbuild {
            raw,
            version,
            pkgname,
            sources,
            hashsums,
            path,
        }));
        let tmp_value = pkgb.clone();
        pkgb.borrow_mut()
            .set_sources(Pkgbuild::parse_sources(raw_double, tmp_value));
        Ok(pkgb)
    }

    pub fn from_file(path: &str) -> Result<Rc<RefCell<Pkgbuild>>, Box<dyn Error>> {
        let path = Path::new(path);
        let display = path.display();

        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };

        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => Err(Box::new(why)),
            Ok(_) => Ok(Pkgbuild::new(s, Some(path.to_path_buf()))?),
        }
    }

    pub fn hash_alg(&self) -> HashAlg {
        self.hashsums.alg
    }

    pub fn set_version(&mut self, new_version: String) {
        self.raw = self
            .raw
            .replace(self.version.as_ref().unwrap(), &new_version);
        self.version = Some(new_version);
    }

    pub fn version(&self) -> &Option<String> {
        &self.version
    }

    pub fn path(&self) -> &Option<PathBuf> {
        &self.path
    }

    pub fn pkgname(&self) -> &String {
        &self.pkgname
    }

    pub fn set_hash(&mut self, index: usize, new_hash: String) -> Result<(), Box<dyn Error>> {
        let current_hash = &self.hashsums.hashes[index];
        self.raw = self.raw.replace(current_hash, &new_hash);
        self.hashsums.hashes[index] = new_hash;
        Ok(())
    }

    fn parse_hashsums(pkg: &str) -> Option<HashSums> {
        let hash_types = ["md5", "b2", "sha1", "sha224", "sha256", "sha348", "sha512"];
        for hash_type in hash_types.iter() {
            let line_prefix = hash_line_prefix(hash_type.to_string());
            if pkg.contains(&line_prefix) {
                return Pkgbuild::parse_typed_hashes(pkg.to_string(), hash_type.to_string());
            }
        }
        None
    }

    fn parse_typed_hashes(pkg_string: String, hash_type: String) -> Option<HashSums> {
        let mut hashes: Vec<String> = Vec::new();
        let mut in_hashes = false;
        let line_prefix = hash_line_prefix(hash_type.clone());
        let lines = pkg_string.split('\n');
        for line in lines {
            if line.starts_with(&line_prefix) {
                in_hashes = true;
                let tokens = line.split('=');
                if tokens.clone().count() == 2 {
                    let hashes_right = tokens.last().unwrap().to_string();
                    Pkgbuild::parse_hash(hashes_right, &mut hashes);
                }
            } else if in_hashes {
                Pkgbuild::parse_hash(line.to_string(), &mut hashes);
            }
            if line.contains(')') {
                in_hashes = false;
            }
        }
        HashSums::new(hash_type, hashes)
    }

    fn parse_hash(string: String, result: &mut Vec<String>) {
        let hashes_dirty = string.split('\'');
        for candidate in hashes_dirty {
            let candidate_trimmed = candidate.trim(); // TODO: also trim tabulation
            if candidate_trimmed.len() > 1 {
                // this check just happens to work
                // gets rid of braces '(' & ')'
                result.push(candidate_trimmed.to_string());
            }
        }
    }

    // TODO: wrap in result
    // TODO: come up with Enum for type of version
    fn parse_version(pkg: &str) -> Option<String> {
        let lines = pkg.split('\n');
        for line in lines {
            if line.starts_with("pkgver=") {
                let tokens = line.split('=');
                if tokens.clone().count() == 2 {
                    let pkgver = tokens.last().unwrap();
                    return Some(pkgver.to_string());
                }
            }
        }
        None
    }

    fn parse_pkgname(pkg: &str) -> Option<String> {
        let lines = pkg.split('\n');
        for line in lines {
            if line.starts_with("pkgname=") {
                let tokens = line.split('=');
                if tokens.clone().count() == 2 {
                    let pkgver = tokens.last().unwrap();
                    return Some(pkgver.to_string());
                }
            }
        }
        None
    }

    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(path);
        let display = path.display();

        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };

        match file.write_all(self.raw.as_bytes()) {
            Err(why) => Err(Box::new(why)),
            Ok(_) => Ok(()),
        }
    }

    // Gets a value of a variable
    pub fn render(&self, variable: String) -> Option<String> {
        let lines = self.raw.split('\n');
        for line in lines {
            let mut var_with_eq_sign = variable.clone();
            var_with_eq_sign.push('=');
            if line.starts_with(&var_with_eq_sign) {
                let tokens = line.split('=');
                if tokens.clone().count() == 2 {
                    let render = tokens.last().unwrap();
                    return Some(render.to_string());
                }
            }
        }
        None
    }

    pub fn sources(&self) -> &Vec<Source> {
        &self.sources
    }

    fn set_sources(&mut self, sources: Vec<Source>) {
        self.sources = sources;
    }

    fn parse_sources(raw: String, pkgb: Rc<RefCell<Pkgbuild>>) -> Vec<Source> {
        let mut sources = Vec::new();
        let lines = raw.split('\n');
        let mut in_sources = false;
        for line in lines {
            if line.starts_with("source") {
                in_sources = true;
                let tokens = line.split('=');
                if tokens.clone().count() == 2 {
                    let sources_right = tokens.last().unwrap();
                    Pkgbuild::parse_source(
                        sources_right,
                        &mut sources,
                        Rc::downgrade(&pkgb.clone()),
                    );
                }
            } else if in_sources {
                Pkgbuild::parse_source(line, &mut sources, Rc::downgrade(&pkgb.clone()));
            }
            if line.contains(')') {
                in_sources = false;
            }
        }
        sources
    }

    fn parse_source(source: &str, result: &mut Vec<Source>, pkgb: Weak<RefCell<Pkgbuild>>) {
        let sources_dirty = source.split('"');
        for candidate in sources_dirty {
            let candidate_trimmed = candidate.trim(); // TODO: also trim tabulation
            if candidate_trimmed.len() > 1 {
                // this check just happens to work
                // gets rid of braces '(' & ')'
                let source_type = Origin::guess(candidate_trimmed.to_string());
                result.push(Source::new(
                    candidate_trimmed.to_string(),
                    source_type,
                    pkgb.clone(),
                    result.len(),
                ));
            }
        }
    }

    // Returns update if it's newer than current version
    pub fn check_for_updates(&self) -> Result<Vec<Update>, Box<dyn Error>> {
        let mut updates: Vec<Update> = Vec::new();
        for source in &self.sources {
            if let Some(update) = source.update_available()? {
                updates.push(update)
            }
        }
        Ok(updates)
    }
}

fn hash_line_prefix(hash_type: String) -> String {
    format!("{}sums=", hash_type)
}

pub fn dir(path_str: &str) -> &Path {
    let path = Path::new(path_str);
    if path.is_file() {
        return path.parent().unwrap();
    }
    path
}

pub fn build(pkgbuild_dir: &Path, settings: &Settings) {
    match settings.build_type() {
        Build::Chroot => {
            match settings.chroot() {
                Some(chroot_path) => {
                    println!(
                        "Starting build for \"{}\" in \"{}\"",
                        &pkgbuild_dir.display(),
                        &chroot_path.display()
                    );
                    // change string into a path & check it
                    let mut chroot_path = chroot_path.to_str().unwrap().to_string();
                    if chroot_path.contains('~') {
                        chroot_path = shellexpand::tilde(&chroot_path).into_owned();
                    }
                    //makechrootpkg -c -r ~/hobby/chroot -n -C -T
                    let mkchrtpkg = Command::new("makechrootpkg")
                        .current_dir(pkgbuild_dir.to_str().unwrap())
                        .arg("-c") // Clean the chroot before building
                        .arg("-r") // The chroot dir to use
                        .arg(chroot_path)
                        //.arg(-n) // Run namcap on the package
                        //.arg(-C) // Run checkpkg on the package
                        .arg("-T") // Build in a temporary directory
                        .stdout(Stdio::inherit())
                        .output()
                        .expect("failed to execute process");
                    println!("::group::Building package in chroot");
                    println!("{}", str::from_utf8(&mkchrtpkg.stdout).unwrap());
                    println!("{}", str::from_utf8(&mkchrtpkg.stderr).unwrap());
                    println!("::endgroup::");
                }
                None => {
                    println!("No chroot path");
                }
            }
        }
        Build::Local => {
            let mkpkg = Command::new("makepkg")
                .current_dir(pkgbuild_dir.to_str().unwrap())
                .arg("--syncdeps") // install dependencies
                .arg("--cleanbuild") // remove `srcdir` dir before the build
                .arg("--clean") // clean up after the build
                .arg("--force") // allows to build package even with existing one in PKGDEST
                .arg("--needed") // pass to pacman
                .arg("--noconfirm") // pass to pacman
                .output()
                .expect("failed to execute process");
            println!("::group::Building package locally");
            println!("{}", str::from_utf8(&mkpkg.stdout).unwrap());
            println!("{}", str::from_utf8(&mkpkg.stderr).unwrap());
            println!("::endgroup::");
        }
        _ => println!("We don't support this build method, yet. Sorry!"),
    }
}

pub fn update_build_env(settings: Settings) -> Result<(), Box<dyn Error>> {
    match settings.build_type() {
        Build::Chroot => match settings.chroot() {
            Some(chroot_path) => chroot::update(chroot_path),
            None => {
                let error: Box<dyn std::error::Error> =
                    String::from("The chroot path is not specified").into();
                Err(error)
            }
        },
        Build::Local => {
            let mkpkg = Command::new("sudo")
                .arg("pacman")
                .arg("-Syu")
                .arg("--noprogressbar")
                .arg("--noconfirm")
                .output()
                .expect("failed to execute process");
            println!("{}", str::from_utf8(&mkpkg.stdout).unwrap());
            println!("{}", str::from_utf8(&mkpkg.stderr).unwrap());
            Ok(())
        }
        _ => {
            let error: Box<dyn std::error::Error> = String::from("Unsupported build method").into();
            Err(error)
        }
    }
}

// take PKGBUILD path and writes
pub fn srcinfo(pkgbuild_path: &Path) -> Result<(), Box<dyn Error>> {
    let pkgbuild_dir = pkgbuild_path.parent().unwrap();
    let mkpkg = Command::new("makepkg")
        .current_dir(pkgbuild_dir.to_str().unwrap())
        .arg("--printsrcinfo")
        .output()
        .expect("failed to start `makepkg` process for .SRCINFO generation");
    if mkpkg.status.success() {
        let stdout = &mkpkg.stdout;
        //let data = format!("{}", str::from_utf8(&mkpkg.stdout).unwrap());

        let file_path = pkgbuild_dir.join(".SRCINFO");
        let mut file = File::create(file_path)?;
        //println!("{}", &data);
        file.write_all(stdout)?;
        return Ok(());
    }

    let error: Box<dyn std::error::Error> = format!(
        "Unable to generate .SRCINFO:\n {}",
        str::from_utf8(&mkpkg.stderr).unwrap()
    )
    .into();
    Err(error)
}

pub fn srcinfo_path(pkgbuild_path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let pkgbuild_dir = pkgbuild_path.parent().unwrap();
    Ok(pkgbuild_dir.join(".SRCINFO"))
}

pub fn find_variables(string: String) -> Vec<String> {
    let re = Regex::new(r"\$\{.*?\}").unwrap();
    re.captures_iter(&string)
        .map(|var_capture| {
            var_capture
                .get(0)
                .unwrap()
                .as_str()
                .to_string()
                .replace("${", "")
                .replace("}", "")
        })
        .collect()
}
