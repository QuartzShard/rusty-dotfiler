use std::fs;
use std::io::ErrorKind;
use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process;

use dirs::home_dir;

use clap::{Parser, Subcommand};
use serde_derive::{Deserialize, Serialize};

// A program to hardlink configuration files to their homes on the filesystem
// from a central repository: or yet another dotfile manager
#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    /// Sets a config file other than ./filemap
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Serialize, Deserialize)]
struct Filemap {
    names: Vec<String>,
    source_paths: Vec<String>,
    install_paths: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Reads the filemap and hardlinks files to their destinations
    Install {},
    /// Checks filesystem from unlinked config files, prints differences
    Check {},
}

fn main() {
    let cli = Cli::parse();

    let filemap: String;
    if let Some(config) = cli.config.as_deref() {
        filemap = String::from(config);
    } else {
        filemap = String::from("./filemap.toml");
    }
    let filemap: Filemap = get_filemap(&clean_path(filemap));
    match &cli.command {
        Some(Commands::Install {}) => install(filemap),
        Some(Commands::Check {}) => check(filemap),
        None => {}
    }
}

fn install(filemap: Filemap) {
    println!("Installing your dotfiles:");
    for i in 0..filemap.names.len() {
        println!(
            "Installing {} at {}",
            filemap.names[i], filemap.install_paths[i]
        );

        let install_path: &Path = &clean_path(filemap.install_paths[i].clone());
        let source_path: &Path = &clean_path(filemap.source_paths[i].clone());

        if install_path.exists() {
            if !is_linked(install_path.to_str(), source_path.to_str()) {
                println!("Removing existing file at {}", install_path.display());
                fs::remove_file(&install_path).unwrap();
            } else {
                println!("File already installed! ");
                continue;
            }
        }
        match fs::hard_link(&source_path, &install_path) {
            Ok(_res) => println!("Linked {} \nto {}", source_path.display(),install_path.display()),
            Err(error) => match error.kind() {
                ErrorKind::NotFound => println!("No file at {}, skipping, ", source_path.display()),
                ErrorKind::PermissionDenied => println!("Permission denied for {}, make sure you can access both source and install directories (Beware '~' if running as root) ", install_path.display()),
                other_error => panic!("{}", other_error),
            }
        };
    }
}

fn check(filemap: Filemap) {
    println!("Checking your dotfiles: ");
    let mut all_clear: bool = true;
    for i in 0..filemap.names.len() {
        let install_path: &Path = &clean_path(filemap.install_paths[i].clone());
        let source_path: &Path = &clean_path(filemap.source_paths[i].clone());
        if !is_linked(install_path.to_str(), source_path.to_str()) {
            all_clear = false;
            println!("File at {} is not linked.", install_path.display());
        } else {
            println!("File at {} is linked.", install_path.display());
        }
    }
    if all_clear {
        println!("All your dotfiles are linked! Changes made in your dotfiles repo will automatically effect the system.")
    } else {
        println!("Some dotfiles aren't hardlinked. Please run again with `install` to link them.")
    }
}

fn is_linked(a: Option<&str>, b: Option<&str>) -> bool {
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

fn get_filemap(filemap: &Path) -> Filemap {
    // println!(
    //     "Using filemap at {}",
    //     filemap.canonicalize().unwrap().display()
    // );
    let filemap_read: String = match fs::read_to_string(filemap) {
        Ok(string) => string,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => gen_filemap(filemap),
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

fn gen_filemap(filemap: &Path) -> String {
    let default_filemap = Filemap {
        names: vec!["".to_string()],
        source_paths: vec!["".to_string()],
        install_paths: vec!["".to_string()],
    };
    let filemap_read = toml::to_string(&default_filemap).unwrap();
    fs::write(filemap,&filemap_read).unwrap();
    return filemap_read
}

fn clean_path(mut target: String) -> PathBuf {
    if &target[..1] == "~" {
        target = target.replace("~", home_dir().unwrap().to_str().unwrap());
        PathBuf::from(&target)
    } else {
        let path = PathBuf::from(&target);
        let mut canon: PathBuf = match path.parent().unwrap().canonicalize() {
            Ok(pathbuf) => pathbuf,
            Err(_error) => panic!("Parent directory does not exist"),
        };
        canon.push(path.file_name().unwrap());
        return canon;
    }
}
