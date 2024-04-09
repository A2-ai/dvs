use serde::{Serialize, Deserialize};
use std::fs;
use std::num::ParseIntError;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    pub storage_dir: PathBuf,
    pub permissions: i32,
    pub group: String
}

pub fn read(root_dir: &PathBuf) -> Result<Config, serde_yaml::Error> {
    // check if yaml is readable
    let yaml_contents = fs::read_to_string(root_dir.join(PathBuf::from(r"dvs.yaml"))).unwrap();
    // check if yaml is deserializable
    let conf: Config = serde_yaml::from_str(&yaml_contents)?;
    Ok(conf)
} // read

pub fn write(config: &Config, dir: &PathBuf) -> std::io::Result<()> {
    let yaml: String = serde_yaml::to_string(&config).unwrap();
    fs::write(dir.join(PathBuf::from(r"dvs.yaml")), yaml)?;
    Ok(())
} // write

pub fn get_mode_u32(permissions: &i32) -> Result<u32, ParseIntError> {
    match u32::from_str_radix(&permissions.to_string(), 8) {
        Ok(mode) => return Ok(mode),
        Err(e) => return Err(e)
    };
}





