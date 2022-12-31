use std::fs;
use std::io::ErrorKind;
use std::path::{Path,PathBuf};
use std::os::linux::fs::MetadataExt;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Filemap {
    pub names: Vec<String>,
    pub source_paths: Vec<String>,
    pub install_paths: Vec<String>,
}

impl Filemap {  
    pub fn new(filemap: &Path) -> Filemap {
        let default_filemap = Filemap {
            names: vec!["".to_string()],
            source_paths: vec!["".to_string()],
            install_paths: vec!["".to_string()],
        };
        default_filemap.save(filemap).unwrap();
        return default_filemap
    }
    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let filemap_read = toml::to_string(&self).unwrap();
        fs::write(path,&filemap_read)
    }
    pub fn is_hard_linked(a: Option<&str>, b: Option<&str>) -> bool {
        let meta_a = match fs::metadata(a.unwrap()) {
            Ok(meta) => meta,
            Err(_error) => return false,
        };
        let meta_b = match fs::metadata(b.unwrap()) {
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
                },
                other_error => panic!("{}", other_error)
            }
        };
        toml::from_str(&filemap_read).unwrap()
    }
}