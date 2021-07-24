extern crate clap;
extern crate reqwest;
extern crate version_compare;

use std::path::PathBuf;
use std::rc::Rc;

use clap::{App, AppSettings, Arg, SubCommand};

use crate::config::Build;

mod chroot;
mod config;
mod context;
mod git;
mod pkgbuild;
mod source;
mod update;

fn main() {
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

    let mut config = Rc::new(config::Config::defaults());

    if let Some(config_path) = matches.value_of("config") {
        let mut given_config = config::Config::read_from_file(PathBuf::from(config_path)).unwrap();
        given_config.set_parent_rc(config.clone());
        config = Rc::new(given_config);
    } else {
        match config::Config::global_config_lookup() {
            Ok(mut global_config) => {
                global_config.set_parent_rc(config);
                config = Rc::new(global_config);
            }
            Err(_) => eprintln!("Unable to find global config"),
        }
    }

    if let Some(path) = matches.value_of("chroot") {
        Rc::get_mut(&mut config)
            .unwrap()
            .set_chroot(PathBuf::from(path));
    }

    let mut context = context::Context::new(config.clone());

    // check if we have any pkgbuild at this point
    context = context.set_config(config);

    if let Some(matches) = matches.subcommand_matches("chroot") {
        if let Some(matches) = matches.subcommand_matches("update") {
            let _config = context.config();
            println!(
                "Using chroot located in: {}",
                matches.value_of("CHROOT").unwrap()
            );
        };
    };

    if let Some(matches) = matches.subcommand_matches("package") {
        if matches.is_present("commit") {
            let mut old_config = context.config().clone();
            Rc::get_mut(&mut old_config).unwrap().set_commit(true);
        }

        if matches.is_present("srcinfo") {
            let mut old_config = context.config().clone();
            Rc::get_mut(&mut old_config).unwrap().set_srcinfo(true);
        }

        if matches.is_present("local-build") {
            let mut old_config = context.config().clone();
            Rc::get_mut(&mut old_config).unwrap().set_build(Build::Local);
        }

        if let Some(path) = matches.value_of("PKGBUILD") {
            let pkgbuild = pkgbuild::Pkgbuild::from_file(&path).unwrap();
            context = context.set_pkgbuild(pkgbuild);

            update(context)
        }
    };
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
    if updates.len() > 0 {
        // test build
        let pkgbuild_dir = path.parent().unwrap();
        pkgbuild::update_build_env(config.clone()).unwrap();
        pkgbuild::build(&pkgbuild_dir, config.clone());
        if let Some(true) = context.config().srcinfo() {
            pkgbuild::srcinfo(&path).unwrap();
        }
        let pkgbuild = pkgbuild.borrow_mut();
        let mut context = context::Context::new(config)
            .set_pkgbuild_path(path)
            // TODO: figure out a way to compose a commit message for minor updates
            .set_pkgname(pkgbuild.pkgname().clone())
            .set_current_version(current_version);
        for update in &updates {
            context = context.set_update(update);
            if let Some(true) = context.config().commit() {
                git::commit(&context).unwrap();
            }
        }
    }
    if updates.len() > 0 {
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
