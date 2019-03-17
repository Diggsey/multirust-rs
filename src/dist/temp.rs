use crate::utils::raw;
use crate::Verbosity;
use log::{debug, warn};
use std::error;
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::ops;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    CreatingRoot { path: PathBuf, error: io::Error },
    CreatingFile { path: PathBuf, error: io::Error },
    CreatingDirectory { path: PathBuf, error: io::Error },
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub struct Cfg {
    root_directory: PathBuf,
    pub dist_server: String,
    verbosity: Verbosity,
}

#[derive(Debug)]
pub struct Dir<'a> {
    cfg: &'a Cfg,
    path: PathBuf,
}

#[derive(Debug)]
pub struct File<'a> {
    cfg: &'a Cfg,
    path: PathBuf,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            CreatingRoot { .. } => "could not create temp root",
            CreatingFile { .. } => "could not create temp file",
            CreatingDirectory { .. } => "could not create temp directory",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        use self::Error::*;
        match *self {
            CreatingRoot { ref error, .. }
            | CreatingFile { ref error, .. }
            | CreatingDirectory { ref error, .. } => Some(error),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> ::std::result::Result<(), fmt::Error> {
        use self::Error::*;
        match *self {
            CreatingRoot { ref path, error: _ } => {
                write!(f, "could not create temp root: {}", path.display())
            }
            CreatingFile { ref path, error: _ } => {
                write!(f, "could not create temp file: {}", path.display())
            }
            CreatingDirectory { ref path, error: _ } => {
                write!(f, "could not create temp directory: {}", path.display())
            }
        }
    }
}

impl Cfg {
    pub fn new(root_directory: PathBuf, dist_server: &str, verbosity: Verbosity) -> Self {
        Cfg {
            root_directory: root_directory,
            dist_server: dist_server.to_owned(),
            verbosity,
        }
    }

    pub fn create_root(&self) -> Result<bool> {
        raw::ensure_dir_exists(&self.root_directory, |p| {
            match self.verbosity {
                Verbosity::Verbose => debug!("creating temp root: {}", p.display()),
                Verbosity::NotVerbose => (),
            };
        })
        .map_err(|e| Error::CreatingRoot {
            path: PathBuf::from(&self.root_directory),
            error: e,
        })
    }

    pub fn new_directory(&self) -> Result<Dir<'_>> {
        self.create_root()?;

        loop {
            let temp_name = raw::random_string(16) + "_dir";

            let temp_dir = self.root_directory.join(temp_name);

            // This is technically racey, but the probability of getting the same
            // random names at exactly the same time is... low.
            if !raw::path_exists(&temp_dir) {
                match self.verbosity {
                    Verbosity::Verbose => debug!("creating temp directory: {}", temp_dir.display()),
                    Verbosity::NotVerbose => (),
                };
                fs::create_dir(&temp_dir).map_err(|e| Error::CreatingDirectory {
                    path: PathBuf::from(&temp_dir),
                    error: e,
                })?;
                return Ok(Dir {
                    cfg: self,
                    path: temp_dir,
                });
            }
        }
    }

    pub fn new_file(&self) -> Result<File<'_>> {
        self.new_file_with_ext("", "")
    }

    pub fn new_file_with_ext(&self, prefix: &str, ext: &str) -> Result<File<'_>> {
        self.create_root()?;

        loop {
            let temp_name = prefix.to_owned() + &raw::random_string(16) + "_file" + ext;

            let temp_file = self.root_directory.join(temp_name);

            // This is technically racey, but the probability of getting the same
            // random names at exactly the same time is... low.
            if !raw::path_exists(&temp_file) {
                match self.verbosity {
                    Verbosity::Verbose => debug!("creating temp file: {}", temp_file.display()),
                    Verbosity::NotVerbose => (),
                };
                fs::File::create(&temp_file).map_err(|e| Error::CreatingFile {
                    path: PathBuf::from(&temp_file),
                    error: e,
                })?;
                return Ok(File {
                    cfg: self,
                    path: temp_file,
                });
            }
        }
    }
}

impl fmt::Debug for Cfg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cfg")
            .field("root_directory", &self.root_directory)
            .field("notify_handler", &"...")
            .finish()
    }
}

impl<'a> ops::Deref for Dir<'a> {
    type Target = Path;

    fn deref(&self) -> &Path {
        ops::Deref::deref(&self.path)
    }
}

impl<'a> ops::Deref for File<'a> {
    type Target = Path;

    fn deref(&self) -> &Path {
        ops::Deref::deref(&self.path)
    }
}

impl<'a> Drop for Dir<'a> {
    fn drop(&mut self) {
        if raw::is_directory(&self.path) {
            match remove_dir_all::remove_dir_all(&self.path) {
                Ok(_) => debug!("deleted temp directory: {}", self.path.display()),
                Err(_) => warn!("could not delete temp directory: {}", self.path.display()),
            }
        }
    }
}

impl<'a> Drop for File<'a> {
    fn drop(&mut self) {
        if raw::is_file(&self.path) {
            match fs::remove_file(&self.path) {
                Ok(_) => debug!("deleted temp file: {}", self.path.display()),
                Err(_) => warn!("could not delete temp file: {}", self.path.display()),
            }
        }
    }
}
