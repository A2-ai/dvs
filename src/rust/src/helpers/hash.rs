use std::{fs::File, path::PathBuf, io::{self, Read, Result}};
use crate::helpers::cache;
use blake3::Hasher;
use crate::helpers::{error::{FileError, FileErrorType}, file::{get_absolute_path, get_relative_path_to_wd}};


pub fn hash_file_with_blake3(file_path: &PathBuf) -> io::Result<Option<String>> {
    let file = File::open(file_path)?;

    let mmap = match maybe_memmap_file(&file) {
        Ok(Some(mmap)) => mmap,
        Ok(None) => {
            // Fallback to reading the file traditionally if memory mapping isn't possible
            return hash_file_with_blake3_direct(file_path);
        }
        Err(e) => return Err(e),
    };
    let mut hasher = Hasher::new();
    hasher.update_rayon(&mmap);
    Ok(Some(hasher.finalize().to_string()))
}

fn hash_file_with_blake3_direct(file_path: &PathBuf) -> io::Result<Option<String>> {
    let mut file = File::open(file_path)?;

    let mut hasher = Hasher::new();
    let mut buffer = [0u8; 16384]; // 16 KB buffer size

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash_result = hasher.finalize();
    Ok(Some(hash_result.to_string()))
}

// Mmap a file, if it looks like a good idea. Return None in cases where we
// know mmap will fail, or if the file is short enough that mmapping isn't
// worth it. However, if we do try to mmap and it fails, return the error.
fn maybe_memmap_file(file: &File) -> Result<Option<memmap2::Mmap>> {
    let metadata = file.metadata()?;
    let file_size = metadata.len();
    Ok(if !metadata.is_file() {
        // Not a real file.
        None
    } else if file_size > isize::max_value() as u64 {
        // Too long to safely map.
        // https://github.com/danburkert/memmap-rs/issues/69
        None
    } else if file_size == 0 {
        // Mapping an empty file currently fails.
        // https://github.com/danburkert/memmap-rs/issues/72
        None
    } else if file_size < 16 * 1024 {
        // Mapping small files is not worth it.
        None
    } else {
        // Explicitly set the length of the memory map, so that filesystem
        // changes can't race to violate the invariants we just checked.
        let map = unsafe {
            memmap2::MmapOptions::new()
                .len(file_size as usize)
                .map(file)?
        };
        Some(map)
    })
}


pub fn get_file_hash(local_path: &PathBuf) -> std::result::Result<String, FileError> {
    // get cache if possible
    if let Ok(cached_hash) = cache::get_cached_hash(local_path) {
        return Ok(cached_hash); // Return cached hash if found
    }

    // if no cached hash, try hashing with blake3
    match hash_file_with_blake3(local_path) {
        Ok(Some(hash)) => {
            // Cache the hash and return it
            let _ = cache::write_hash_to_cache(local_path, &hash);
            Ok(hash)
        },
        Ok(None) => {
            Err(FileError{
                relative_path: get_relative_path_to_wd(local_path).ok(), 
                absolute_path: get_absolute_path(local_path).ok(),
                error: FileErrorType::HashNotFound,
                error_message: None,
                input: local_path.clone()
            })
        }
        Err(e) => {
            // if there's no hash or an error, return None
            Err(FileError{
                relative_path: get_relative_path_to_wd(local_path).ok(),
                absolute_path: get_absolute_path(local_path).ok(),
                error: FileErrorType::HashNotFound,
                error_message: Some(e.to_string()),
                input: local_path.clone()
            })
        },
    }
}


pub fn get_storage_path(storage_dir: &PathBuf, file_hash: &String) -> PathBuf {
    let first_hash_segment: &str = &file_hash[..2];
    let second_hash_segment: &str = &file_hash[2..];
    return storage_dir.join(first_hash_segment).join(second_hash_segment);

}