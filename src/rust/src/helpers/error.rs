use core::fmt;
use std::path::PathBuf;

// FILE ERROR
#[derive(Clone, PartialEq, Debug)]
pub enum FileErrorType {
    RelativePathNotFound,
    FileNotInGitRepo,
    AbsolutePathNotFound,
    PathIsDirectory,
    HashNotFound,
    SizeNotFound,
    OwnerNotFound,
    GroupNotSet,
    PermissionsNotSet,
    MetadataNotSaved,
    DirGitIgnoreNotAdded,
    ProjectGitIgnoreNotAdded,
    FileNotCopied,
    MetadataNotFound,
    FileAlreadyAddedMetadataNotLoaded,
}

#[derive(Debug)]
pub struct FileError {
    pub relative_path: Option<PathBuf>,
    pub absolute_path: Option<PathBuf>,
    pub error: FileErrorType,
    pub error_message: Option<String>,
    pub input: PathBuf,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.error_message.clone() {
            Some(message) => {
                write!(f, "{message}")
            }
            None => {
                write!(f, "NA")
            }
        }
    }
}

impl std::error::Error for FileError {}

impl FileErrorType {
    pub fn file_error_to_string(&self) -> String {
        match self {
            FileErrorType::RelativePathNotFound => String::from("relative path not found"),
            FileErrorType::FileNotInGitRepo => String::from("file not in git repo"),
            FileErrorType::AbsolutePathNotFound => String::from("file not found"),
            FileErrorType::PathIsDirectory => String::from("path is a directory"),
            FileErrorType::HashNotFound => String::from("hash not found"),
            FileErrorType::SizeNotFound => String::from("size not found"),
            FileErrorType::OwnerNotFound => String::from("owner not found"),
            FileErrorType::GroupNotSet => String::from("group not set"),
            FileErrorType::PermissionsNotSet => String::from("group not set"),
            FileErrorType::MetadataNotSaved => String::from("metadata file not saved"),
            FileErrorType::DirGitIgnoreNotAdded => String::from("file directory gitignore entry not added"),
            FileErrorType::ProjectGitIgnoreNotAdded => String::from("project level gitignore entry not added"),
            FileErrorType::FileNotCopied => String::from("file not copied"),
            FileErrorType::MetadataNotFound => String::from("metadata file not found"),
            FileErrorType::FileAlreadyAddedMetadataNotLoaded => String::from("file already added, but metadata not parsed from metadata file (.dvsmeta)"),
        }
    }
}

// BATCH ERROR
#[derive(Clone, PartialEq, Debug)]
pub enum BatchErrorType {
    AnyFilesDNE,
    GitRepoNotFound,
    ConfigNotFound,
    GroupNotFound,
    StorageDirNotFound,
    PermissionsInvalid,
    AnyMetaFilesDNE,
}


#[derive(Debug)]
pub struct BatchError {
    pub error: BatchErrorType,
    pub error_message: String,
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error_message)
    }
}

impl std::error::Error for BatchError {}

impl BatchErrorType {
    pub fn batch_error_to_string(&self) -> String {
        match self {
            BatchErrorType::AnyFilesDNE => String::from("at least one inputted file not found"),
            BatchErrorType::GitRepoNotFound => String::from("git repository not found"),
            BatchErrorType::ConfigNotFound => String::from("configuration file not found"),
            BatchErrorType::GroupNotFound => String::from("linux primary group not found"),
            BatchErrorType::StorageDirNotFound => String::from("storage directory not found"),
            BatchErrorType::PermissionsInvalid => String::from("linux file permissions invalid"),
            BatchErrorType::AnyMetaFilesDNE => String::from("metadata file not found for at least one file"),
        }
    }
}

#[derive(Debug)]
pub struct InitError {
    pub error: InitErrorType,
    pub error_message:String,
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.error.init_error_to_string(), self.error_message)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum InitErrorType {
    ProjAlreadyInited,
    StorageDirNotCreated,
    StorageDirPermsNotSet,
    StorageDirNotADir,
    StorageDirAbsPathNotFound,
    GitRepoNotFound,
    ConfigNotCreated,
    GroupNotFound,
    PermissionsInvalid,
    DirEmptyNotChecked
}

impl InitErrorType {
    pub fn init_error_to_string(&self) -> String {
        match self {
            InitErrorType::ProjAlreadyInited => String::from("project already initialized"),
            InitErrorType::GitRepoNotFound => String::from("git repo not found"),
            InitErrorType::StorageDirNotADir => String::from("storage directory input is not a directory"),
            InitErrorType::StorageDirAbsPathNotFound => String::from("could not get absolute path for storage directory"),
            InitErrorType::ConfigNotCreated => String::from("configuration file not found"),
            InitErrorType::GroupNotFound => String::from("linux primary group not found"),
            InitErrorType::StorageDirNotCreated => String::from("storage directory not created"),
            InitErrorType::PermissionsInvalid => String::from("linux file permissions invalid"),
            InitErrorType::DirEmptyNotChecked => String::from("could not check if storage directory is empty"),
            InitErrorType::StorageDirPermsNotSet => String::from("storage directory permissions not set"),
        }
    }
}

impl std::error::Error for InitError {}