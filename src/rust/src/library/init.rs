use crate::helpers::{config, error::{InitError, InitErrorType}, repo};
use std::{ffi::OsStr, fs::{self, create_dir}, os::unix::fs::PermissionsExt, path::PathBuf};
use file_owner::Group;
use std::env;

pub type Result<T> = core::result::Result<T, InitError>;

#[derive(Clone, Debug, PartialEq)]
pub struct Init {
    pub storage_directory: PathBuf,
    pub group: String,
    pub permissions: i32
}

pub fn dvs_init(storage_dir: &PathBuf, octal_permissions: Option<i32>, group_name: Option<&str>) -> Result<Init> { 
    // Get git root
    let path = env::current_dir().unwrap_or(PathBuf::from("."));
    let git_dir = repo::get_nearest_repo_dir(&path).map_err(|e|
        InitError{
            error: InitErrorType::GitRepoNotFound,
            error_message: format!("make sure you're in an active git repository. {e}")
        }
    )?;

    // get group
    let group: String = {
        if let Some(some_name) = group_name {
            Group::from_name(some_name).map_err(|e|
                InitError{
                    error: InitErrorType::GroupNotFound,
                    error_message: format!("could not find group {some_name}. {e}")
                }
            )?;
            String::from(some_name)
        }
        else {
            String::from("")
        }
     };

     // get permissions
    let permissions = {
        if let Some(some_perms) = octal_permissions {
            u32::from_str_radix(&some_perms.to_string(), 8).map_err(|e|
                InitError{
                    error: InitErrorType::PermissionsInvalid,
                    error_message: format!("linux permissions: {some_perms} not valid. {e}")
                }
            )?;
            some_perms
        }
        else { 
            // default value
            664
        }
    };

    // get storage_dir absolute path, but don't check if it exists yet
    let storage_dir_abs = repo::absolutize_result(&storage_dir).map_err(|e|
        InitError{
            error: InitErrorType::StorageDirAbsPathNotFound,
            error_message: e.to_string()
        }
    )?;

    // if already initialized
    if let Ok(conf) = config::read(&git_dir) {
        // no-op if the same
        if conf.storage_dir == storage_dir_abs && conf.group == group && conf.permissions == permissions {
            return Ok(
                Init{
                    storage_directory: storage_dir_abs,
                    group: group.clone(),
                    permissions: permissions.clone()
                }
            )
        }
        // error if config attributes are different
        else { 
            return Err(
                InitError{
                    error: InitErrorType::ProjAlreadyInited,
                    error_message: format!("dvs configuration settings already set in project; change manually by updating dvs.yaml in project root: {}", git_dir.join(PathBuf::from("dvs.yaml")).display())
                }
            )
        }
    }
    
    if storage_dir_abs.extension().and_then(OsStr::to_str).is_some() {
        println!("warning: file path inputted as storage directory. Is this intentional?") // [MAN-INI-002]
    }
    
    // create storage directory if it doesn't exist
    if !storage_dir_abs.exists() { 
        println!("storage directory doesn't exist\ncreating storage directory..."); // [MAN-INI-004]
        // create storage dir
        create_dir(&storage_dir_abs).map_err(|e|
            InitError{
                error: InitErrorType::StorageDirNotCreated,
                error_message: format!("{} not created. {e}", storage_dir.display()) // [MAN-INI-006]
            }
        )?;

        // set permissions for storage dir
        let storage_dir_perms = fs::Permissions::from_mode(0o770);
        fs::set_permissions(&storage_dir_abs, storage_dir_perms).map_err(|e| {
            InitError{
                error: InitErrorType::StorageDirPermsNotSet, // [MAN-INI-007]
                error_message: e.to_string()
            }
        })?;
    } 

    else { // else, storage directory exists
        if !storage_dir_abs.is_dir() {
            return Err(InitError{
                error: InitErrorType::StorageDirNotADir, 
                error_message: format!("file path inputted")
            });
        }

        println!("storage directory already exists"); // [MAN-INI-005]

        //  Warn if storage dir is not empty
        if !repo::is_directory_empty(&storage_dir_abs).map_err(|e|
            InitError{
                error: InitErrorType::DirEmptyNotChecked,
                error_message: e.to_string()
            }
        )? {
            println!("warning: storage directory not empty") // [MAN-INI-001]
        }
    } // else, storage directory exists

    // warn if storage directory is in git repo
    if repo::dir_in_git_repo(&storage_dir_abs, &git_dir) {
        println!("warning: the storage directory is located in the git repo directory.\nfiles added to the storage directory will be uploaded directly to git.") // [MAN-INI-003]
    }

    // write config
    config::write(
        &config::Config{
            storage_dir: storage_dir_abs.clone(), 
            permissions: permissions.clone(),
            group: group.clone()
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
            group: group.clone(),
            permissions: permissions.clone()
        }
    )
    
    
}
