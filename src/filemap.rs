use std::fs;
use std::io::ErrorKind;
use std::path::{Path,PathBuf};
use std::os::linux::fs::MetadataExt;
use std::process;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Filemap {
    pub names: Vec<String>,
    pub source_paths: Vec<String>,
    pub install_paths: Vec<String>,
}

impl Filemap {  
    pub fn new(filemap: &Path) -> String {
        let default_filemap = Filemap {
            names: vec!["".to_string()],
            source_paths: vec!["".to_string()],
            install_paths: vec!["".to_string()],
        };
        let filemap_read = toml::to_string(&default_filemap).unwrap();
        fs::write(filemap,&filemap_read).unwrap();
        return filemap_read
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
}

impl From<&PathBuf> for Filemap {
    fn from(filemap: &PathBuf) -> Self {
        let filemap_read: String = match fs::read_to_string(filemap) {
            Ok(string) => string,
            Err(error) => match error.kind() {
                ErrorKind::NotFound => Filemap::new(filemap),
                other_error => panic!("{}", other_error)
            }
        };
        let filemap_parsed: Filemap = toml::from_str(&filemap_read).unwrap();
        if filemap_parsed.names.len() == 0 || filemap_parsed.names[0] == "" {
            println!("No files specified! Edit your config at {}\nExample can be found at https://github.com/QuartzShard/rusty-dotfiler/blob/main/example-filemap.toml", filemap.display());
            process::exit(0);
        } else {
            filemap_parsed
        }
    }
}