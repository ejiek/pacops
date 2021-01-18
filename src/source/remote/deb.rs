use select::document::Document;
use select::predicate::Name;
use version_compare::{CompOp, VersionCompare};

use std::error::Error;

use crate::source::Source;
use crate::update::Update;

pub fn latest(source: &Source) -> Result<Option<Update>, Box<dyn Error>> {
    let raw = source.raw();
    let repo_url = parent(raw);
    let packages = list(repo_url.clone())?;
    let mut versions: Vec<Update> = packages
        .iter()
        // filter out unrelated packages
        .map(|x| Update {
            version: extract_version_by_template(x.to_string(), &source),
            source_index: source.index,
            url: format!("{}/{}", repo_url, x.to_string()),
        })
        .collect();
    let mut latest_version: Option<Update> = None;
    for version in versions {
        match latest_version {
            None => latest_version = Some(version),
            Some(ref l) => {
                if VersionCompare::compare_to(&version.version, &l.version, &CompOp::Ge).unwrap() {
                    latest_version = Some(version);
                }
            }
        }
    }
    Ok(latest_version)
}

// Extracts URL to a parent directory
fn parent(source: String) -> String {
    let position = source.rfind('/').unwrap();
    let (repo, _) = source.split_at(position);
    repo.to_string()
}

fn filename_part(source: String) -> String {
    let position = source.rfind('/').unwrap();
    let (_, file) = source.split_at(position + 1); // I don't like +1
    file.to_string()
}

// Lists all packages available in the repository
// TODO: filter out other packages to have all available versions of a single one
fn list(url: String) -> Result<Vec<String>, Box<dyn Error>> {
    let resp = reqwest::blocking::get(&url)?;
    let mut packages = Vec::new();
    Document::from_read(resp)
        .unwrap()
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .for_each(|x| {
            if x.contains(".deb") {
                packages.push(x.to_string())
            }
        });
    Ok(packages)
}

// Takes version prefix and postfix from template and removes them from repo filename
// We hope it leaves us with a version of a remote file
fn extract_version_by_template(file: String, source: &Source) -> String {
    //${_pkgname}_${pkgver}-1_amd64.deb
    //microsoft-edge-dev_88.0.680.1-1_amd64.deb
    let mut version = file;
    let template = filename_part(source.raw());
    let surroundings = template.split("${pkgver}");
    for this in surroundings {
        let mut render = this.to_string();
        let vars = crate::pkgbuild::find_variables(this.to_string());
        for var in vars {
            if render.contains(&var) {
                // TODO: look for all variables
                render = render.replace(
                    &format!("${{{}}}", var),
                    &source
                        .pkgbuild
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .render(var.clone())
                        .unwrap(),
                );
            }
        }
        version = version.replace(&render, "");
    }
    version
}
