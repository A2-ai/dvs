use xdg;
use std::{fs::{self, create_dir_all, File}, path::PathBuf, time::SystemTime};
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Context, Result};

#[derive(Serialize, Deserialize)]
pub struct CacheData {
    hash: String,
    modification_time: Option<SystemTime>,
}

pub fn get_cached_hash(path: &PathBuf) -> Result<String> {
    // get absolute path
    let abs_path = match path.canonicalize() {
        Ok(path) => path,
        Err(e) => return Err(anyhow!(e)),
    };
    
    let xdg_dirs = xdg::BaseDirectories::with_prefix("dvs").with_context(|| "could not get xdg directories")?;

    // open the cache
    let cache_path = match xdg_dirs.find_cache_file(&abs_path) {
        Some(path) => path,
        None => return Err(anyhow!("could not get cache path")),
    };

    // get cache file contents
    let contents = fs::read_to_string(&cache_path)?;

    // deserialize to struct
    let cache_data: CacheData = match serde_json::from_str(&contents) {
        Ok(data) => data,
        Err(e) => return Err(anyhow!(format!("could not get metadata for {}: \n{e}", path.display())))
    };

    // get modification_time - using option for serde serialization
    let current_modification_time: Option<SystemTime> = match abs_path.metadata() {
        Ok(data) => {
            let mod_time: Option<SystemTime> = match data.modified() {
                Ok(time) => Some(time),
                Err(_) => None,
            };
            mod_time
        }
        Err(e) =>  return Err(anyhow!(e)),
    };

    // ensure modification time matches
    if current_modification_time != cache_data.modification_time {
        let _ = fs::remove_file(cache_path);
        return Err(anyhow!("file modification time does not match cache (invalidating)"));
    }

    // return hash
    return Ok(cache_data.hash);
}

pub fn write_hash_to_cache(path: &PathBuf, hash: &String) -> Result<()> {
    // get absolute path
    let abs_path = path.canonicalize().with_context(|| format!("could not get absolute path: {}", path.display()))?;

    // get modification_time - using option for serde serialization
    let modification_time: Option<SystemTime> = match abs_path.metadata() {
        Ok(data) => {
            let mod_time: Option<SystemTime> = match data.modified() {
                Ok(time) => Some(time),
                Err(_) => None,
            };
            mod_time
        }
        Err(e) =>  return Err(anyhow!(e)),
    };

    // put modification_time and hash into struct
    let cached_data = CacheData{
        modification_time,
        hash: hash.clone()
    };

    // get xdg directories
    let xdg_dirs = xdg::BaseDirectories::with_prefix("Rdevious").with_context(|| "could not get xdg directories")?;

    // get path for cache file
    let cache_path = xdg_dirs.place_cache_file(abs_path).with_context(|| "could not get path for cache file")?;

    create_dir_all(&cache_path)?;

    // create file
    File::create(&cache_path).with_context(|| "could not create cache file")?;

    // serialize file contents
    let contents = serde_json::to_string_pretty(&cached_data).unwrap();

    // write contents to file
    fs::write(&cache_path, contents).with_context(|| "could not write to cache file")?;

    Ok(())
}