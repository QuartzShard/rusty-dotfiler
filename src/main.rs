use std::fs;
use std::io::{self, Result, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::vec::IntoIter;
use std::process;

use dirs::home_dir;
use sudo::with_env;
use clap::{Parser, Subcommand};

pub mod filemap;
use crate::filemap::Filemap;

// A program to hardlink configuration files to their homes on the filesystem
// from a central repository: or yet another dotfile manager


// Struct to outline command line options
#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    /// Sets a config file other than ./filemap
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}
// Enum to outline subcommands
#[derive(Subcommand)]
enum Commands {
    /// Reads the filemap and hardlinks files to their destinations
    Install {},
    /// Checks filesystem from unlinked config files, prints differences
    Check {},
    /// Reads the subdirectories of the currect directory and adds files to the filemap
    Configure {},
}

// Parse options, run command
fn main() {
    let cli = Cli::parse();

    let filemap_path: String;
    if let Some(config) = cli.config.as_deref() {
        filemap_path = String::from(config);
    } else {
        filemap_path = String::from("./filemap.toml");
    }
    let filemap: Filemap = Filemap::from(&clean_path(filemap_path.clone()));
    match &cli.command {
        Some(Commands::Install {}) => install(filemap, &clean_path(filemap_path)),
        Some(Commands::Check {}) => check(filemap, &clean_path(filemap_path)),
        Some(Commands::Configure {}) => configure(filemap, &clean_path(filemap_path)),
        None => {}
    }
}

///// Commands

// Link files in list to install dirs
fn install(filemap: Filemap, config_path: &Path) {
    if filemap.check_empty() {
        println!("No files specified! Edit your config at {}\nExample can be found at https://github.com/QuartzShard/rusty-dotfiler/blob/main/example-filemap.toml", config_path.display());
        process::exit(0);
    }
    println!("Installing your dotfiles:");
    for i in 0..filemap.names.len() {
        println!(
            "Installing {} at {}",
            filemap.names[i], filemap.install_paths[i]
        );

        let install_path: &Path = &clean_path(filemap.install_paths[i].clone());
        let source_path: &Path = &clean_path(filemap.source_paths[i].clone());

        if install_path.exists() {
            if !Filemap::is_hard_linked(install_path.to_str(), source_path.to_str()) {
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
                ErrorKind::PermissionDenied => {with_env(&["HOME"]).unwrap();},
                // println!("Permission denied for {}, make sure you can access both source and install directories (Beware '~' if running as root) ", install_path.display()),
                other_error => panic!("{}", other_error),
            }
        };
    }
}

// Check filelist for links
fn check(filemap: Filemap, config_path: &Path) {
    if filemap.check_empty() {
        println!("No files specified! Edit your config at {}\nExample can be found at https://github.com/QuartzShard/rusty-dotfiler/blob/main/example-filemap.toml", config_path.display());
        process::exit(0);
    }
    println!("Checking your dotfiles: ");
    let mut all_clear: bool = true;
    for i in 0..filemap.names.len() {
        let install_path: &Path = &clean_path(filemap.install_paths[i].clone());
        let source_path: &Path = &clean_path(filemap.source_paths[i].clone());
        if !Filemap::is_hard_linked(install_path.to_str(), source_path.to_str()) {
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

fn configure(mut filemap: Filemap, config_path: &Path) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    filemap.names.clear();
    filemap.install_paths.clear();
    filemap.source_paths.clear();

    for path in read_dir_tree(".").unwrap() {
        let mut entry_name = String::new();
        let mut entry_install_path = String::new();

        writeln!(handle, "Found:  {}", path).unwrap();
        write!(handle, "Please enter a name for the config, or ! to skip: ").unwrap();
        handle.flush().unwrap();

        stdin.read_line(&mut entry_name).expect("Can't read input!");
        if entry_name.trim() == "!" {
            continue;
        }
        write!(handle, "Please enter a path to link to, or ! to skip: ").unwrap();
        handle.flush().unwrap();

        stdin.read_line(&mut entry_install_path).expect("Can't read input!");
        if entry_install_path == "!" {
            continue;
        }
        let entry_name: String = String::from(entry_name.trim());
        let entry_install_path: String = String::from(entry_install_path.trim());

        filemap.names.push(entry_name);
        filemap.install_paths.push(entry_install_path);
        filemap.source_paths.push(path.clone());
    }
    match filemap.save(config_path) {
        Ok(()) => {
            writeln!(handle, "Saved config at {}", config_path.display()).unwrap();
            handle.flush().unwrap();
        },
        Err(_) => {
            writeln!(handle, "Failed to save config at {}", config_path.display()).unwrap();
            handle.flush().unwrap();
        }
    }
    
}

///// Functions

// Ensure path is canonical & parent is real
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

fn read_dir_tree(path: &str) -> Result<IntoIter<String>> {
    let mut paths: Vec<String> = vec![];
    paths = read_dir_tree_branch(Path::new(path), paths)?;
    Ok(paths.into_iter())
    
}

fn read_dir_tree_branch(path: &Path, mut paths: Vec<String>) -> Result<Vec<String>> {
    for entry in fs::read_dir(path)? {
        let entrypath = entry?.path();
        if entrypath.is_dir() {
            if &entrypath.file_name().unwrap().to_str().unwrap()[0..1] == "."{
                println!("Skipping hidden dir: {}", entrypath.display());
                continue;
            }
            paths = read_dir_tree_branch(&entrypath, paths)?;
            continue;
        }
        if entrypath.file_name().unwrap().to_str().unwrap() == "filemap.toml"{
            continue;
        }
        let pathstr = String::from(entrypath.to_str().unwrap());
        paths.push(pathstr);
    }
    Ok(paths)
}
