use xdg;
use std::{fs::{self, File}, path::PathBuf, time::SystemTime};
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Context, Result};
use crate::helpers::repo;

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
    
    // open the cache
    let cache_path = get_cache_path(&abs_path)?;

    let contents = fs::read_to_string(&cache_path)?;

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

    if current_modification_time != cache_data.modification_time {
        let _ = fs::remove_file(cache_path);
        return Err(anyhow!("file modification time does not match cache (invalidating)"));
    }

    
    return Ok(cache_data.hash);

}

pub fn write_hash_to_cache(path: &PathBuf, hash: &String) -> Result<()> {
    let abs_path = path.canonicalize().with_context(|| format!("could not get absolute path: {}", path.display()))?;
    // cache_path = $HOME/.cache/dvs/<project_name>/<relative path between file and git directory>
    let cache_path = get_cache_path(&abs_path)?;
    //let cache_path: PathBuf = [cache_home, abs_path].iter().collect();
    // create directories in path
    fs::create_dir_all(&cache_path.parent().unwrap())?;

    // create file
    File::create(&cache_path).with_context(|| "could not create cache file")?;

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

    

    // serialize file contents
    let contents = serde_json::to_string_pretty(&cached_data).unwrap();

    // write contents to file
    fs::write(&cache_path, contents).with_context(|| "could not write to cache file")?;
    //write!(&cache_path, &contents.as_str)?;

    Ok(())
}

fn get_cache_path(abs_path: &PathBuf) -> Result<PathBuf> {
    let git_dir = repo::get_nearest_repo_dir(&abs_path)?;
    let rel_path = repo::get_relative_path(&git_dir, &abs_path)?;
    let project_name = PathBuf::from(git_dir.file_name().unwrap().to_str().unwrap().to_string());
    // partial_cache_path = project_name/<relative path between file and git directory>
    let partial_cache_path = project_name.join(&rel_path);

    let xdg_dirs = xdg::BaseDirectories::with_prefix("dvs").with_context(|| "could not get xdg directories")?;

    // cache_path = $HOME/.cache/dvs/<project_name>/<relative path between file and git directory>
    let cache_path = xdg_dirs.place_cache_file(&partial_cache_path).with_context(|| "could not get path for cache file")?;
    Ok(cache_path)
}