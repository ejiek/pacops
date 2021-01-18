extern crate clap;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;

use clap::{App, Arg, SubCommand};

mod pkgbuild;

fn main() {
    let matches = App::new("PacOps")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("PKGBUILD")
                .help("Sets the PKGBUILD file to use")
                .required(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();
    // get PKGBUILD
    // Parse it
    // find sources
    // analyze them (is it remote -> get version)
    // compare version

    match matches.value_of("PKGBUILD") {
        Some(pkg) => {
            let path = Path::new(pkg);
            let display = path.display();

            let mut file = match File::open(&path) {
                Err(why) => panic!("couldn't open {}: {}", display, why),
                Ok(file) => file,
            };

            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Err(why) => panic!("couldn't read {}: {}", display, why),
                Ok(_) => {
                    let pkgbuild = pkgbuild::Pkgbuild::new(s).unwrap();
                    println!("{}", pkgbuild.version());
                }
            }
            println!("Value for PKGBUILD: {}", display);
        }
        None => {
            eprintln!("No PKGBUILD file found");
            process::exit(0x0001);
        }
    }
    println!("End of the program");
}
