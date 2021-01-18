use std::error::Error;

pub struct Pkgbuild {
    content: String,
}

impl Pkgbuild {
    pub fn new(content: String) -> Result<Pkgbuild, Box<dyn Error>> {
        Ok(Pkgbuild { content })
    }

    // TODO: wrap in result
    // TODO: come up with Enum for type of version
    pub fn version(&self) -> String {
        let lines = self.content.split('\n');
        for line in lines {
            if line.starts_with("pkgver") {
                let tokens = line.split('=');
                if tokens.clone().count() == 2 {
                    let pkgver = tokens.last().unwrap();
                    return pkgver.to_string();
                }
            }
        }
        return String::from("");
    }
}
