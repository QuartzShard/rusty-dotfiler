use std::fs;
use std::io::ErrorKind;
use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process;

use serde_derive::{Deserialize, Serialize};

use crate::error::SimpleRunError;

#[derive(Serialize, Deserialize)]
pub struct Filemap {
    pub names: Vec<String>,
    pub source_paths: Vec<String>,
    pub install_paths: Vec<String>,
}

impl Filemap {
    pub fn new(filemap: &Path) -> Filemap {
        let default_filemap: Filemap = Filemap {
            names: vec!["".to_string()],
            source_paths: vec!["".to_string()],
            install_paths: vec!["".to_string()],
        };
        match default_filemap.save(filemap) {
            Ok(()) => (),
            Err(error) => println!("{}", error),
        };
        return default_filemap;
    }
    pub fn save(&self, path: &Path) -> Result<(), SimpleRunError> {
        toml::to_string(&self)
            .map_err(|_| SimpleRunError::SaveError)
            .and_then(|filemap_read| {
                fs::write(path, &filemap_read).map_err(|_| SimpleRunError::SaveError)
            })
    }
    pub fn is_hard_linked(a: Option<&str>, b: Option<&str>) -> bool {
        let a = match a {
            Some(s) => s,
            None => return false,
        };
        let b = match b {
            Some(s) => s,
            None => return false,
        };
        let meta_a = match fs::metadata(a) {
            Ok(meta) => meta,
            Err(_error) => return false,
        };
        let meta_b = match fs::metadata(b) {
            Ok(meta) => meta,
            Err(_error) => return false,
        };

        return meta_a.st_ino() == meta_b.st_ino();
    }
    pub fn check_empty(&self) -> bool {
        if self.names.len() == 0 || self.names[0] == "" {
            return true;
        }
        false
    }
}

impl From<&PathBuf> for Filemap {
    fn from(filemap: &PathBuf) -> Self {
        let filemap_read: String = match fs::read_to_string(filemap) {
            Ok(string) => string,
            Err(error) => match error.kind() {
                ErrorKind::NotFound => {
                    return Filemap::new(filemap);
                }
                ErrorKind::PermissionDenied => {
                    println!("No permissions to open the filemap. Please check file permissions and try again");
                    process::exit(0);
                }
                other_error => panic!("{}", other_error),
            },
        };
        match toml::from_str(&filemap_read) {
            Ok(r) => r,
            Err(error) => {
                println!("Unable to parse filemap, generating new one: {}", error);
                return Filemap::new(filemap);
            }
        }
    }
}
