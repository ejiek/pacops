extern crate clap;
extern crate reqwest;
extern crate version_compare;

use std::error::Error;
use std::{path::PathBuf, str::FromStr};

use clap::{App, AppSettings, Arg, SubCommand};

use crate::settings::Settings;

mod chroot;
mod context;
mod git;
mod pkgbuild;
mod settings;
mod source;
mod update;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("PacOps")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity."),
        )
        .subcommand(
            SubCommand::with_name("package")
                .about("Which, How & Why of package building")
                .arg(
                    Arg::with_name("PKGBUILD")
                        .help("Sets the PKGBUILD file to use.")
                        .required(true), // TODO: check if it's present in current dir instead
                )
                .arg(
                    Arg::with_name("commit")
                        .long("commit")
                        .help("Commits the change to a local git repo."),
                )
                .arg(
                    Arg::with_name("srcinfo")
                        .long("srcinfo")
                        .help("Generates .SRCINFO, useful for AUR packages."),
                )
                .arg(
                    Arg::with_name("chroot")
                        .help("Path to a \"clean\" chroot. Build will happen in the chroot.")
                        .short("r")
                        .long("chroot")
                        .value_name("PATH")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("local-build")
                        .short("l")
                        .long("local-build")
                        .help("Builds package locally. Useful when used inside a container.")
                        .takes_value(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("chroot")
                .about("Manipulate chroots")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("update")
                        .about("updates build environment")
                        .arg(
                            Arg::with_name("CHROOT")
                                .help("Path to the chroot to update.")
                                .required(true), // TODO: use a default one or one from config
                        ),
                ),
        )
        .get_matches();

    let mut config;

    if let Some(config_path) = matches.value_of("config") {
        config = settings::Settings::builder(Some(config_path.to_string())).unwrap();
    } else {
        config = settings::Settings::builder(None).unwrap();
    }

    if let Some(path) = matches.value_of("chroot") {
        config.set("chroot", path)?;
    }

    if let Some(matches) = matches.subcommand_matches("chroot") {
        if let Some(matches) = matches.subcommand_matches("update") {
            println!(
                "Using chroot located in: {}",
                matches.value_of("CHROOT").unwrap()
            );
        };
    };

    if let Some(matches) = matches.subcommand_matches("package") {
        if matches.is_present("commit") {
            config.set("commit", true)?;
        }

        if matches.is_present("push") {
            config.set("push", true)?;
        }

        if matches.is_present("srcinfo") {
            config.set("srcinfo", true)?;
        }

        if matches.is_present("local-build") {
            config.set("build", "local")?;
        }

        if let Some(path) = matches.value_of("PKGBUILD") {
            let pkgbuild = pkgbuild::Pkgbuild::from_file(path).unwrap();
            let mut context = context::Context::new(config.clone().try_into().unwrap());
            context = context.set_pkgbuild(pkgbuild);
            context = context.set_pkgbuild_path(PathBuf::from_str(path).unwrap());

            let config: Settings = config.try_into().unwrap();
            println!("{:?}", config);

            update(context)
        }
    };
    Ok(())
}

fn update(context: context::Context) {
    let pkgbuild = context.pkgbuild().unwrap();
    let config = context.config();
    let path = pkgbuild.borrow().path().as_ref().unwrap().clone();
    let current_version = pkgbuild.borrow().version().as_ref().unwrap().clone();
    let updates = pkgbuild.borrow().check_for_updates().unwrap();
    for update in &updates {
        println!(
            "Update available\n\t{} over {}",
            update.version, current_version
        );
        let mut pkgbuild = pkgbuild.borrow_mut();
        pkgbuild.set_version(update.version.clone());
        let new_hash = update.hash(pkgbuild.hash_alg()).unwrap();
        pkgbuild.set_hash(update.source_index, new_hash).unwrap();
        pkgbuild.to_file(path.as_path().to_str().unwrap()).unwrap();
    }
    if !updates.is_empty() {
        // test build
        let pkgbuild_dir = path.parent().unwrap();
        pkgbuild::update_build_env(config.clone()).unwrap();
        pkgbuild::build(pkgbuild_dir, &config);
        if let true = config.srcinfo() {
            pkgbuild::srcinfo(&path).unwrap();
        }
        let pkgbuild = pkgbuild.borrow_mut();
        let mut context = context::Context::new(config.clone())
            .set_pkgbuild_path(path)
            // TODO: figure out a way to compose a commit message for minor updates
            .set_pkgname(pkgbuild.pkgname().clone())
            .set_current_version(current_version);
        for update in &updates {
            context = context.set_update(update);
            if config.commit() {
                git::commit(&context).unwrap();
                if config.push() {
                    git::push(&context).unwrap();
                }
            }
        }
    }
    if !updates.is_empty() {
        // publish (git, git subtee)
        //
        // build
        // sign
        //    `gpg --sign --detach-sign --local-user 'userk@mail' `
        // publish to a repo
        //   rsync repo.db locally
        //   save it as .old
        //   rsync repo.db back so server
        //   rsync new package there
    } else {
        println!("No update available")
    }
}
