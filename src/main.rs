use core::fmt;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::vec::IntoIter;

use clap::{Parser, Subcommand};
use dirs::home_dir;
use sudo::with_env;

pub mod filemap;
use crate::filemap::Filemap;

// A program to hardlink configuration files to their homes on the filesystem
// from a central repository: or yet another dotfile manager

// Struct to outline command line options
#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    /// Sets a config file other than ./filemap.toml
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
fn main() -> Result<(), SimpleRunError> {
    let cli = Cli::parse();

    let filemap_path: String;
    if let Some(config) = cli.config.as_deref() {
        filemap_path = String::from(config);
    } else {
        filemap_path = String::from("./filemap.toml");
    }
    let filemap_path = &clean_path(filemap_path).unwrap_or_else(|| {
        println!("Cannot parse path, using current directory for filemap.");
        PathBuf::from("./filemap.toml")
    });
    let filemap: Filemap = Filemap::from(filemap_path);
    match &cli.command {
        Some(Commands::Install {}) => install(filemap, filemap_path),
        Some(Commands::Check {}) => check(filemap, filemap_path),
        Some(Commands::Configure {}) => configure(filemap, filemap_path),
        None => Ok(()),
    }
}

///// Commands

// Link files in list to install dirs
fn install(filemap: Filemap, config_path: &Path) -> Result<(), SimpleRunError> {
    if filemap.check_empty() {
        println!("No files specified! Run `rusty-dotfiler configure` to generate a filemap from your dotfiles.");
        println!("Alternatively, manually edit your config at {}\nExample can be found at https://github.com/QuartzShard/rusty-dotfiler/blob/main/example-filemap.toml", config_path.display());
        return Err(SimpleRunError::new("No files specified.".to_owned()));
    }
    println!("Installing your dotfiles:");
    for i in 0..filemap.names.len() {
        println!(
            "Installing {} at {}",
            filemap.names[i], filemap.install_paths[i]
        );

        let install_path: PathBuf = match clean_path(filemap.install_paths[i].clone()) {
            Some(path) => path,
            None => continue,
        };
        let source_path: PathBuf = match clean_path(filemap.source_paths[i].clone()) {
            Some(path) => path,
            None => continue,
        };

        if install_path.exists() {
            if !Filemap::is_hard_linked(install_path.to_str(), source_path.to_str()) {
                match fs::remove_file(&install_path) {
                    Ok(()) => println!("Removing existing file at {}", install_path.display()),
                    Err(error) => println!(
                        "Failed to remove file at {}\nReason: {}",
                        install_path.display(),
                        error
                    ),
                };
            } else {
                println!("File already installed! ");
                continue;
            }
        }
        match fs::hard_link(&source_path, &install_path) {
            Ok(_res) => println!(
                "Linked {} \nto {}",
                source_path.display(),
                install_path.display()
            ),
            Err(error) => match error.kind() {
                ErrorKind::NotFound => {
                    println!("No file at {}, skipping, ", source_path.display())
                }
                ErrorKind::PermissionDenied => match with_env(&["HOME"]) {
                    Ok(_) => (),
                    Err(_error) => {
                        println!("Can't `sudo` to link {}, skipping", source_path.display())
                    }
                },
                _other_error => println!("Unexpected error creating hardlink, skipping"),
            },
        }
    }
    Ok(())
}

// Check filelist for links
fn check(filemap: Filemap, config_path: &Path) -> Result<(), SimpleRunError> {
    if filemap.check_empty() {
        println!("No files specified! Run `rusty-dotfiler configure` to generate a filemap from your dotfiles.");
        println!("Alternatively, manually edit your config at {}\nExample can be found at https://github.com/QuartzShard/rusty-dotfiler/blob/main/example-filemap.toml", config_path.display());
        return Err(SimpleRunError::new("No files specified.".to_owned()));
    }
    println!("Checking your dotfiles: ");
    let mut all_clear: bool = true;
    for i in 0..filemap.names.len() {
        let install_path: PathBuf = match clean_path(filemap.install_paths[i].clone()) {
            Some(path) => path,
            None => continue,
        };
        let source_path: PathBuf = match clean_path(filemap.source_paths[i].clone()) {
            Some(path) => path,
            None => continue,
        };

        if !Filemap::is_hard_linked(install_path.to_str(), source_path.to_str()) {
            all_clear = false;
            println!("File at {} is not linked.", install_path.display());
        } else {
            println!("File at {} is linked.", install_path.display());
        }
    }
    if all_clear {
        Ok(println!("All your dotfiles are linked! Changes made in your dotfiles repo will automatically effect the system."))
    } else {
        Err(SimpleRunError::UnlinkedFilesFound)
    }
}

fn configure(mut filemap: Filemap, config_path: &Path) -> Result<(), SimpleRunError> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    filemap.names.clear();
    filemap.install_paths.clear();
    filemap.source_paths.clear();

    let dir_tree = match read_dir_tree(".") {
        Ok(dir_tree) => dir_tree,
        Err(_error) => Err(SimpleRunError::FailedToRead),
    };

    for path in dir_tree {
        let mut entry_name = String::new();
        let mut entry_install_path = String::new();

        writeln!(handle, "Found:  {}", path)?;
        write!(handle, "Please enter a name for the config, or ! to skip: ")?;
        handle.flush()?;

        match stdin.read_line(&mut entry_name) {
            Ok(_res) => (),
            Err(_error) => {
                println!("Can't read input!");
                continue;
            }
        };
        if entry_name.trim() == "!" {
            continue;
        }
        write!(handle, "Please enter a path to link to, or ! to skip: ")?;
        handle.flush()?;

        match stdin.read_line(&mut entry_install_path) {
            Ok(_res) => (),
            Err(_error) => {
                println!("Can't read input!");
                continue;
            }
        };
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
            writeln!(handle, "Saved config at {}", config_path.display())?;
            handle.flush()?;
            Ok(())
        }
        Err(_) => Err(SimpleRunError::FailedToSave),
    }
}

///// Functions

// Ensure path is canonical & parent is real
fn clean_path(mut target: String) -> Option<PathBuf> {
    if &target[..1] == "~" {
        target = target.replace("~", home_dir()?.to_str()?);
        Some(PathBuf::from(&target))
    } else {
        let path = PathBuf::from(&target);
        let mut canon: PathBuf = match path.parent()?.canonicalize() {
            Ok(pathbuf) => pathbuf,
            Err(_error) => {
                println!("Parent directory does not exist");
                return None;
            }
        };
        canon.push(path.file_name()?);
        return Some(canon);
    }
}

fn read_dir_tree(path: &str) -> Result<IntoIter<String>, SimpleRunError> {
    let mut paths: Vec<String> = vec![];
    paths = read_dir_tree_branch(Path::new(path), paths)?;
    Ok(paths.into_iter())
}

fn read_dir_tree_branch(
    path: &Path,
    mut paths: Vec<String>,
) -> Result<Vec<String>, SimpleRunError> {
    for entry in fs::read_dir(path)? {
        let entrypath = entry?.path();
        if entrypath.is_dir() {
            if &entrypath
                .file_name()
                .ok_or_else(|| SimpleRunError::InvalidFilename)?
                .to_str()
                .ok_or_else(|| SimpleRunError::UnparsableFilepath(entrypath))?[0..1]
                == "."
            {
                println!("Skipping hidden dir: {}", entrypath.display());
                continue;
            }
            paths = read_dir_tree_branch(&entrypath, paths)?;
            continue;
        }
        if entrypath
            .file_name()
            .ok_or_else(|| SimpleRunError::InvalidFilename)?
            .to_str()
            .ok_or_else(|| SimpleRunError::UnparsableFilepath(entrypath))?
            == "filemap.toml"
        {
            continue;
        }
        let pathstr = String::from(
            entrypath
                .to_str()
                .ok_or_else(|| SimpleRunError::UnparsableFilepath(entrypath))?,
        );
        paths.push(pathstr);
    }
    Ok(paths)
}

// Error Struct

#[derive(Debug)]
enum SimpleRunError {
    UnparsableFilepath(PathBuf),
    InvalidFilename,
    FailedToSave,
    FailedToLink(PathBuf),
    NoFilesSpecified,
    FailedToRead,
    UnlinkedFilesFound,
}

impl SimpleRunError {
    fn as_str(&self) -> &'static str {
        match *self {
            UnparsableFilepath => "Unparsable filepath specified",
            InvalidFilename => "Invalid filename",
            FailedToSave => "Failed to save filemap, chech the directory is accessible and not full.",
            FailedToLink => "Failed to create link",
            NoFilesSpecified => "No files specified! Run `rusty-dotfiler configure` to generate a filemap from your dotfiles.",
            FailedToRead => "Failed to read directory tree.",
            UnlinkedFilesFound => "Some dotfiles aren't linked! Run `rusty-dotfiler install` to link them."
        }
    }
}

// impl std::error::Error for SimpleRunError {}
impl fmt::Display for SimpleRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl From<std::io::Error> for SimpleRunError {
    fn from(io_error: std::io::Error) -> Self {
        Self::FailedToSave
    }
}
