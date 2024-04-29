use crate::helpers::{repo, config};
use std::{ffi::OsStr, fmt, fs::create_dir, path::PathBuf};
use file_owner::Group;

#[derive(Debug)]
pub struct InitError {
    pub error_type: String,
    pub error_message:String,
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.error_message)
    }
}

#[derive(Clone, PartialEq)]
enum InitErrorType {
    ProjAlreadyInited,
    StorageDirNotCreated,
    StorageDirNotADir,
    StorageDirAbsPathNotFound,
    GitRepoNotFound,
    ConfigNotCreated,
    GroupNotFound,
    PermissionsInvalid,
    DirEmptyNotChecked
}

impl InitErrorType {
    fn init_error_type_to_string(&self) -> String {
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
        }
    }
}

impl std::error::Error for InitError {}
pub type Result<T> = core::result::Result<T, InitError>;
// pub type Error = Box<dyn std::error::Error>;

pub fn dvs_init(storage_dir: &PathBuf, octal_permissions: &i32, group_name: &str) -> Result<()> { 
    // Get git root
    let git_dir = repo::get_nearest_repo_dir(&PathBuf::from(".")).map_err(|e|
        InitError{
            error_type: InitErrorType::GitRepoNotFound.init_error_type_to_string(),
            error_message: format!("make sure you're in an active git repository. {e}")
        }
    )?;

    // if already initialized
    if git_dir.join(PathBuf::from(r"dvs.yaml")).exists() {
        return Err(
            InitError{
                error_type: InitErrorType::ProjAlreadyInited.init_error_type_to_string(),
                error_message: format!("already initialized project with dvs. to change initialization settings, manually update dvs.yaml in project root")
            }
        )
    }

    // get absolute path, but don't check if it exists yet
    let storage_dir_abs = repo::absolutize_result(&storage_dir).map_err(|e|
        InitError{
            error_type: InitErrorType::StorageDirAbsPathNotFound.init_error_type_to_string(),
            error_message: e.to_string()
        }
    )?;
    
    if storage_dir_abs.extension().and_then(OsStr::to_str).is_some() {
        println!("warning: file path inputted as storage directory. Is this intentional?")
    }
    
    // create storage directory if it doesn't exist
    if !storage_dir_abs.exists() { 
        println!("storage directory doesn't exist\ncreating storage directory...");
        // create storage dir
        create_dir(&storage_dir_abs).map_err(|e|
            InitError{
                error_type: InitErrorType::StorageDirNotCreated.init_error_type_to_string(),
                error_message: format!("{} not created. {e}", storage_dir.display())
            }
        )?;
    } 

    else { // else, storage directory exists
        if !storage_dir_abs.is_dir() {
            return Err(InitError{
                error_type: InitErrorType::StorageDirNotADir.init_error_type_to_string(), 
                error_message: format!("file path inputted")
            });
        }

        println!("storage directory already exists");

        //  Warn if storage dir is not empty
        if !repo::is_directory_empty(&storage_dir_abs).map_err(|e|
            InitError{
                error_type: InitErrorType::DirEmptyNotChecked.init_error_type_to_string(),
                error_message: e.to_string()
            }
        )? {
            println!("warning: storage directory not empty")
        }

    } // else

    // warn if storage directory is in git repo
    if repo::dir_in_git_repo(&storage_dir_abs, &git_dir) {
        println!("warning: the storage directory is located in the git repo directory.\nfiles added to the storage directory will be uploaded directly to git.")
    }

    // check group exists
    if group_name != "" {
        Group::from_name(group_name).map_err(|e|
            InitError{
                error_type: InitErrorType::GroupNotFound.init_error_type_to_string(),
                error_message: format!("could not find group {group_name}. {e}")
            }
        )?;
    }

    // check permissions are convertible to u32
    u32::from_str_radix(&octal_permissions.to_string(), 8).map_err(|e|
        InitError{
            error_type: InitErrorType::PermissionsInvalid.init_error_type_to_string(),
            error_message: format!("linux permissions: {octal_permissions} not valid. {e}")
        }
    )?;

    // write config
    config::write(
        &config::Config{
            storage_dir: storage_dir_abs.clone(), 
            permissions: octal_permissions.clone(),
            group: group_name.to_string()
        }, 
        &git_dir).map_err(|e|
            InitError{
                error_type: InitErrorType::ConfigNotCreated.init_error_type_to_string(),
                error_message: e.to_string()

            }
        )?;
    
    println!("initialized storage directory: {}", storage_dir.display());
    Ok(())
}
