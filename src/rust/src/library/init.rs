use crate::helpers::{config, error::{InitError, InitErrorType}, repo};
use std::{ffi::OsStr, fs::{self, create_dir}, os::unix::fs::PermissionsExt, path::PathBuf};
use file_owner::Group;

pub type Result<T> = core::result::Result<T, InitError>;

#[derive(Clone, Debug, PartialEq)]
pub struct Init {
    pub storage_directory: PathBuf,
    pub group: String,
    pub file_permissions: i32
}

pub fn dvs_init(storage_dir: &PathBuf, octal_permissions: &i32, group_name: &str) -> Result<Init> { 
    // Get git root
    let git_dir = repo::get_nearest_repo_dir(&PathBuf::from(".")).map_err(|e|
        InitError{
            error: InitErrorType::GitRepoNotFound,
            error_message: format!("make sure you're in an active git repository. {e}")
        }
    )?;

    // if already initialized
    if git_dir.join(PathBuf::from(r"dvs.yaml")).exists() {
        return Err(
            InitError{
                error: InitErrorType::ProjAlreadyInited,
                error_message: format!("already initialized project with dvs. to change initialization settings, manually update dvs.yaml in project root")
            }
        )
    }

    // get absolute path, but don't check if it exists yet
    let storage_dir_abs = repo::absolutize_result(&storage_dir).map_err(|e|
        InitError{
            error: InitErrorType::StorageDirAbsPathNotFound,
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
                error: InitErrorType::StorageDirNotCreated,
                error_message: format!("{} not created. {e}", storage_dir.display())
            }
        )?;
    } 

    else { // else, storage directory exists
        if !storage_dir_abs.is_dir() {
            return Err(InitError{
                error: InitErrorType::StorageDirNotADir, 
                error_message: format!("file path inputted")
            });
        }

        println!("storage directory already exists");

        //  Warn if storage dir is not empty
        if !repo::is_directory_empty(&storage_dir_abs).map_err(|e|
            InitError{
                error: InitErrorType::DirEmptyNotChecked,
                error_message: e.to_string()
            }
        )? {
            println!("warning: storage directory not empty")
        }
    } // else, storage directory exists

    // set permissions for storage dir
    let storage_dir_perms = fs::Permissions::from_mode(0o770);
    fs::set_permissions(&storage_dir_abs, storage_dir_perms).map_err(|e| {
        InitError{
            error: InitErrorType::StorageDirPermsNotSet,
            error_message: e.to_string()
        }
    })?;

    // warn if storage directory is in git repo
    if repo::dir_in_git_repo(&storage_dir_abs, &git_dir) {
        println!("warning: the storage directory is located in the git repo directory.\nfiles added to the storage directory will be uploaded directly to git.")
    }

    // check group exists
    if group_name != "" {
        Group::from_name(group_name).map_err(|e|
            InitError{
                error: InitErrorType::GroupNotFound,
                error_message: format!("could not find group {group_name}. {e}")
            }
        )?;
    }

    // check permissions are convertible to u32
    u32::from_str_radix(&octal_permissions.to_string(), 8).map_err(|e|
        InitError{
            error: InitErrorType::PermissionsInvalid,
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
                error: InitErrorType::ConfigNotCreated,
                error_message: e.to_string()

            }
        )?;
    
    println!("initialized storage directory: {}", storage_dir.display());
    return Ok(
        Init{
            storage_directory: storage_dir_abs,
            group: String::from(group_name),
            file_permissions: octal_permissions.clone()
        }
    )
    
    
}
