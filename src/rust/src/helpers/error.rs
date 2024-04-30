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
    GitIgnoreNotAdded,
    FileNotCopied,
    MetadataNotFound,
}

#[derive(Debug)]
pub struct FileError {
    pub relative_path: Option<PathBuf>,
    pub absolute_path: Option<PathBuf>,
    pub error_type: FileErrorType,
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
    pub fn file_error_type_to_string(&self) -> String {
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
            FileErrorType::GitIgnoreNotAdded => String::from("gitignore entry not added"),
            FileErrorType::FileNotCopied => String::from("file not copied"),
            FileErrorType::MetadataNotFound => String::from("metadata file not found"),
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
    pub error_type: BatchErrorType,
    pub error_message: String,
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error_message)
    }
}

impl std::error::Error for BatchError {}

impl BatchErrorType {
    pub fn batch_error_type_to_string(&self) -> String {
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

