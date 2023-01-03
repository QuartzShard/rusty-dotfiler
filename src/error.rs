use core::fmt;
// Error Struct

#[derive(Debug)]
pub enum SimpleRunError {
    UnparsableFilepath,
    InvalidFilename,
    FailedToSave,
    FailedToLink,
    NoFilesSpecified,
    FailedToRead,
    UnlinkedFilesFound,
    SaveError,
}

impl SimpleRunError {
    pub fn as_str(&self) -> &'static str {
        use SimpleRunError::*;
        match &*self {
            UnparsableFilepath => "Unparsable filepath specified",
            InvalidFilename => "Invalid filename",
            FailedToSave => "Failed to save filemap, chech the directory is accessible and not full.",
            FailedToLink => "Failed to create link",
            NoFilesSpecified => "No files specified! Run `rusty-dotfiler configure` to generate a filemap from your dotfiles.",
            FailedToRead => "Failed to read directory tree.",
            UnlinkedFilesFound => "Some dotfiles aren't linked! Run `rusty-dotfiler install` to link them.",
            SaveError => "Can't save filemap."
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
    fn from(_io_error: std::io::Error) -> Self {
        Self::FailedToSave
    }
}

#[derive(Debug)]
pub struct SaveError;

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Can't save filemap.")
    }
}
