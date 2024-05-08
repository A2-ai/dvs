use file_owner::Group;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use crate::helpers::error::{BatchError, BatchErrorType};

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    pub storage_dir: PathBuf,
    pub permissions: i32,
    pub group: String
}


pub fn read(root_dir: &PathBuf) -> std::result::Result<Config, BatchError> {
    // check if yaml is readable
    let yaml_contents = fs::read_to_string(root_dir.join(PathBuf::from(r"dvs.yaml"))).map_err(|e| {
        BatchError{
            error: BatchErrorType::ConfigNotFound,
            error_message: format!("could not load configuration file, i.e. no dvs.yaml in directory; be sure to initiate dvs: {e}")
        }
    })?;
    // check if yaml is deserializable
    let conf: Config = serde_yaml::from_str(&yaml_contents).map_err(|e| {
        BatchError{
            error: BatchErrorType::ConfigNotFound,
            error_message: format!("could not load configuration file, i.e. no dvs.yaml in directory; be sure to initiate dvs: {e}")
        }
    })?;
    Ok(conf)
} // read

pub fn write(config: &Config, dir: &PathBuf) -> Result<()> {
    let yaml: String = serde_yaml::to_string(&config)?;
    fs::write(dir.join(PathBuf::from(r"dvs.yaml")), yaml)?;
    Ok(())
} // write

pub fn get_mode_u32(permissions: &i32) -> std::result::Result<u32, BatchError> {
    Ok(u32::from_str_radix(&permissions.to_string(), 8).map_err(|e| {
        BatchError{
            error: BatchErrorType::PermissionsInvalid,
            error_message: format!("change permissions: {} in dvs.yaml, {e}", permissions)
        }
    })?)
}

pub fn get_group(group_name: &String) -> std::result::Result<Option<Group>, BatchError> {
    if group_name == "" {
        return Ok(None)
    }
    else {
        return Ok(Some(Group::from_name(&group_name.as_str()).map_err(|e|
            BatchError{
                error: BatchErrorType::GroupNotFound,
                error_message: format!("change group: {} in dvs.yaml, {e}", group_name)
            }
        )?))
    };
}

pub fn get_storage_dir(storage_dir: &PathBuf) -> std::result::Result<PathBuf, BatchError>{
    Ok(storage_dir.canonicalize().map_err(|e|
            BatchError{
                error: BatchErrorType::StorageDirNotFound,
                error_message: format!("change storage_dir: {} in dvs.yaml, {e}", storage_dir.display())
            }
        )?)
}







