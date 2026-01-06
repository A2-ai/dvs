# DVS Function Index

This document provides a comprehensive index of all public functions and their relationships, organized for quick lookup when investigating specific functionality.

---

## Quick Reference: Entry Points

| R Function | Rust FFI | Rust Implementation | Primary Helpers |
|------------|----------|---------------------|-----------------|
| `dvs_init()` | `dvs_init_impl()` | `library/init.rs::dvs_init()` | config, repo |
| `dvs_add()` | `dvs_add_impl()` | `library/add.rs::add()` | hash, copy, file, ignore |
| `dvs_get()` | `dvs_get_impl()` | `library/get.rs::get()` | hash, copy, file |
| `dvs_status()` | `dvs_status_impl()` | `library/status.rs::status()` | hash, file |

---

## R Layer Functions

### R/init.R

```r
dvs_init(storage_directory, permissions = NULL, group = NULL) -> data.frame
```
**Purpose**: Initialize DVS in a git repository
**Parameters**:
- `storage_directory`: Path to external storage (created if doesn't exist)
- `permissions`: Optional octal permissions for stored files (e.g., 664)
- `group`: Optional Linux group for stored files

**Returns**: Data frame with columns: `storage_directory`, `permissions`, `group`

**Errors**: `dvs_init_error` class with messages for:
- Not in git repo
- Invalid permissions
- Invalid group
- Already initialized with different settings

---

### R/add.R

```r
dvs_add(files, message = NULL, split_output = FALSE) -> data.frame | list
```
**Purpose**: Version files by copying to storage and creating metadata
**Parameters**:
- `files`: Vector of file paths or glob patterns
- `message`: Optional message stored in metadata
- `split_output`: If TRUE, returns `list(successes=df, failures=df)`

**Returns**:
- Single df: `relative_path`, `outcome`, `size`, `blake3_checksum`, `absolute_path`, `input`, `error`, `error_message`
- Split: `successes` df (without error cols) + `failures` df (with error cols)

**Outcomes**: `"copied"` (new), `"present"` (already current), `"error"`

---

### R/get.R

```r
dvs_get(files, split_output = FALSE) -> data.frame | list
```
**Purpose**: Retrieve files from storage to local project
**Parameters**:
- `files`: Vector of file paths or glob patterns (can use `.dvs` extension)
- `split_output`: If TRUE, returns `list(successes=df, failures=df)`

**Returns**: Same structure as `dvs_add()`

**Outcomes**: `"copied"` (restored), `"present"` (already local), `"error"`

---

### R/status.R

```r
dvs_status(files = c(""), split_output = FALSE) -> data.frame | list
```
**Purpose**: Report synchronization status of versioned files
**Parameters**:
- `files`: File paths/globs (empty string = all tracked files)
- `split_output`: If TRUE, returns `list(successes=df, failures=df)`

**Returns**:
- `relative_path`, `status`, `size`, `blake3_checksum`, `add_time`, `saved_by`, `message`, `absolute_path`, `error`, `error_message`, `input`

**Statuses**:
- `"current"`: Local file matches stored version
- `"absent"`: Metadata exists but local file missing
- `"unsynced"`: Local file differs from stored version
- `"error"`: Could not determine status

---

### R/set_functions.R

```r
normalize_paths(files) -> character
```
**Purpose**: Expand `~` in file paths
**Internal use only**

---

## Rust FFI Layer (lib.rs)

### Initialization

```rust
#[extendr]
fn dvs_init_impl(storage_dir: &str, mode: Nullable<i32>, group: Nullable<&str>) -> Result<Robj>
```
**Converts**: `Init` struct -> `RInit` -> R data.frame

---

### Add Files

```rust
#[extendr]
fn dvs_add_impl(files_string: Vec<String>, message: Nullable<&str>, strict: bool, split_output: bool) -> Result<Robj>
```
**Note**: `strict=true` means failed files cleanup their partial state (metadata + storage file)

---

### Get Files

```rust
#[extendr]
fn dvs_get_impl(files_string: Vec<String>, split_output: bool) -> Result<Robj>
```

---

### Status

```rust
#[extendr]
fn dvs_status_impl(files: Vec<String>, split_output: bool) -> Result<Robj>
```

---

### File Info (Not exported to R)

```rust
#[extendr]
fn get_file_info_impl(paths: Vec<String>, split_output: bool) -> Robj
```
**Purpose**: Get file metadata (owner, group, permissions, timestamps)

---

### Glob Parsing

```rust
#[extendr]
fn parse_files_from_globs_add_impl(globs: Vec<String>) -> Vec<String>

#[extendr]
fn parse_files_from_globs_get_impl(globs: Vec<String>) -> Result<Vec<String>>

#[extendr]
fn parse_files_from_globs_status_impl(globs: Vec<String>) -> Result<Vec<String>>

#[extendr]
fn is_explicit_path_impl(entry: String) -> bool
```
**Purpose**: Expand glob patterns before core operations

---

## Rust Library Layer

### library/init.rs

```rust
pub fn dvs_init(
    storage_dir: &PathBuf,
    octal_permissions: Option<i32>,
    group_name: Option<&str>
) -> Result<Init>
```

**Logic Flow**:
1. Find git root directory
2. Validate group exists (if provided)
3. Validate permissions format (if provided)
4. Check for existing config - no-op if same, error if different
5. Create storage directory if needed (with 0o770 permissions)
6. Warn if storage is in git repo
7. Write dvs.yaml config

---

### library/add.rs

```rust
pub fn add(
    files: &Vec<PathBuf>,
    message_in: Option<&str>,
    strict: bool
) -> Result<Vec<Result<AddedFile, FileError>>, BatchError>

fn add_file(
    local_path: &PathBuf,
    git_dir: &PathBuf,
    group: &Option<Group>,
    storage_dir: &PathBuf,
    permissions: &u32,
    message: &String,
    strict: bool
) -> Result<AddedFile, FileError>
```

**Logic Flow per file**:
1. Get absolute and relative paths
2. Check not a directory
3. Compute blake3 hash (with cache check)
4. If already added & current -> return Present (no-op)
5. Check file is in git repo
6. Get file size and owner
7. Create metadata struct with timestamp
8. Save .dvs metadata file
9. Add .gitignore entry
10. Compute storage path from hash
11. Copy to storage (if not already there)
12. Set permissions and group on stored file

---

### library/get.rs

```rust
pub fn get(files: &Vec<PathBuf>) -> Result<Vec<Result<RetrievedFile, FileError>>, BatchError>

pub fn get_file(
    local_path: &PathBuf,
    storage_dir: &PathBuf,
    git_dir: &PathBuf
) -> Result<RetrievedFile, FileError>
```

**Logic Flow per file**:
1. Check metadata file is in git repo
2. Load metadata (.dvs file)
3. Compute local hash (if file exists)
4. Compare with metadata hash
5. If different or missing -> copy from storage
6. Return file info

---

### library/status.rs

```rust
pub fn status(files: &Vec<String>) -> Result<Vec<Result<FileStatus, FileError>>, BatchError>

fn status_file(local_path: &PathBuf) -> Result<FileStatus, FileError>
```

**Logic Flow per file**:
1. Check metadata file exists
2. Load metadata
3. Determine status:
   - File doesn't exist -> Absent
   - File hash matches metadata -> Current
   - File hash differs -> Unsynced

---

### library/info.rs

```rust
pub fn info(paths: &Vec<String>) -> Vec<Result<FileInfo>>
```

**Returns**: File ownership (uid, username, gid, group), timestamps, permissions

---

## Rust Helper Functions

### helpers/hash.rs

```rust
pub fn hash_file_with_blake3(file_path: &PathBuf) -> io::Result<Option<String>>
fn hash_file_with_blake3_direct(file_path: &PathBuf) -> io::Result<Option<String>>
fn maybe_memmap_file(file: &File) -> Result<Option<memmap2::Mmap>>
pub fn get_file_hash(local_path: &PathBuf) -> Result<String, FileError>
pub fn get_storage_path(storage_dir: &PathBuf, file_hash: &String) -> PathBuf
```

**Optimization**: Uses memory-mapped I/O with rayon parallelization for files > 16KB

**Storage Path Format**: `{storage_dir}/{first_2_chars}/{remaining_chars}`

---

### helpers/copy.rs

```rust
pub fn copy_impl(src_path: &PathBuf, dest_path: &PathBuf) -> Result<()>
pub fn copy(local_path: &PathBuf, storage_path: &PathBuf) -> Result<(), FileError>
pub fn set_file_permissions(mode: &u32, local_path: &PathBuf) -> Result<(), FileError>
pub fn set_group(group: &Option<Group>, local_path: &PathBuf) -> Result<(), FileError>
pub fn copy_file_to_storage_directory(
    local_path: &PathBuf,
    storage_path: &PathBuf,
    permissions: &u32,
    group: &Option<Group>
) -> Result<(), FileError>
```

---

### helpers/file.rs

```rust
pub fn save(metadata: &Metadata, local_path: &PathBuf) -> Result<(), FileError>
pub fn load(local_path: &PathBuf) -> Result<Metadata, FileError>
pub fn metadata_path(path: &PathBuf) -> PathBuf  // Adds .dvs extension
pub fn path_without_metadata(path: &PathBuf) -> PathBuf  // Removes .dvs extension
pub fn get_user_name(local_path: &PathBuf) -> Result<String, FileError>
pub fn get_absolute_path(local_path: &PathBuf) -> Result<PathBuf, FileError>
pub fn get_relative_path_to_wd(local_path: &PathBuf) -> Result<PathBuf, FileError>
pub fn check_if_dir(local_path: &PathBuf) -> Result<(), FileError>
pub fn get_file_size(local_path: &PathBuf) -> Result<u64, FileError>
pub fn check_meta_files_exist(queued_paths: &Vec<PathBuf>) -> Result<(), BatchError>
pub fn try_to_get_abs_path(local_path: &PathBuf) -> Option<PathBuf>
pub fn try_to_get_rel_path(local_path: &PathBuf) -> Option<PathBuf>
```

---

### helpers/config.rs

```rust
pub fn read(root_dir: &PathBuf) -> Result<Config, BatchError>
pub fn write(config: &Config, dir: &PathBuf) -> Result<()>
pub fn get_mode_u32(permissions: &i32) -> Result<u32, BatchError>
pub fn get_group(group_name: &String) -> Result<Option<Group>, BatchError>
pub fn get_storage_dir(storage_dir: &PathBuf) -> Result<PathBuf, BatchError>
```

---

### helpers/ignore.rs

```rust
pub fn add_gitignore_entry(local_path: &PathBuf) -> Result<(), FileError>
fn add_gitignore_entry_helper(path: &PathBuf) -> Result<()>
```

**Creates entries**:
```
# dvs entry
/filename.csv
!/filename.csv.dvs
```

---

### helpers/parse.rs

```rust
pub fn get_all_meta_files() -> Result<Vec<PathBuf>, BatchError>
pub fn parse_files_from_globs_add(globs: &Vec<String>) -> Vec<PathBuf>
pub fn parse_files_from_globs_status(globs: &Vec<String>) -> Result<Vec<PathBuf>, BatchError>
pub fn parse_files_from_globs_get(globs: &Vec<String>) -> Result<Vec<PathBuf>, BatchError>
pub fn is_explicit_path(entry: &String) -> bool
pub fn check_metafiles_for_explicit_paths(files: &Vec<String>) -> Result<(), BatchError>
fn filter_path(path: &PathBuf, queued_paths: &Vec<PathBuf>) -> Option<PathBuf>
fn filter_meta_path(path: &PathBuf, queued_paths: &Vec<PathBuf>) -> Option<PathBuf>
```

**Key Behavior**:
- `add`: Expands globs, filters .gitignore and .dvs files
- `get`: Requires metadata files to exist for explicit paths
- `status`: Empty input returns all tracked files; filters to files with .dvs

---

### helpers/repo.rs

```rust
pub fn absolutize_result(path: &PathBuf) -> Result<PathBuf>
pub fn get_relative_path(root_dir: &PathBuf, file_path: &PathBuf) -> Result<PathBuf>
pub fn get_relative_path_to_wd(local_path: &PathBuf) -> Result<PathBuf, FileError>
pub fn get_nearest_repo_dir(dir: &PathBuf) -> Result<PathBuf, BatchError>
pub fn check_file_in_git_repo(local_path: &PathBuf, git_dir: &PathBuf) -> Result<(), FileError>
pub fn dir_in_git_repo(path: &PathBuf, git_dir: &PathBuf) -> bool
pub fn is_directory_empty(directory: &Path) -> Result<bool>
fn is_git_repo(dir: &PathBuf) -> bool  // Checks for .git directory
```

---

### helpers/cache.rs

```rust
pub fn get_cached_hash(path: &PathBuf) -> Result<String>
pub fn write_hash_to_cache(path: &PathBuf, hash: &String) -> Result<()>
fn get_cache_path(abs_path: &PathBuf) -> Result<PathBuf>
```

**Cache Location**: `~/.cache/dvs/{project_name}/{relative_path}`
**Invalidation**: File modification time check

---

### helpers/outcome.rs

```rust
pub enum Outcome { Copied, Present, Error }
pub enum Status { Absent, Unsynced, Current, Error }

impl Outcome { pub fn outcome_to_string(&self) -> String }
impl Status { pub fn outcome_to_string(&self) -> String }
```

---

## Error Function Reference

### helpers/error.rs

```rust
impl FileErrorType {
    pub fn file_error_to_string(&self) -> String
}

impl BatchErrorType {
    pub fn batch_error_to_string(&self) -> String
}

impl InitErrorType {
    pub fn init_error_to_string(&self) -> String
}
```

**Error Hierarchy**:
- `InitError`: Project-level init failures
- `BatchError`: Operation-level failures (config missing, storage missing)
- `FileError`: Per-file failures (returned in results, not thrown)
