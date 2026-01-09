# DVS File Index

This document provides a detailed index of every file in the DVS codebase, organized by directory with descriptions of purpose, key functions, and importance ratings.

## Importance Ratings

- **CRITICAL**: Core functionality, must understand for any changes
- **HIGH**: Important for feature work and bug fixes
- **MEDIUM**: Supporting functionality, understand when relevant
- **LOW**: Configuration, tests, or generated files

---

## Root Directory

| File | Importance | Description |
|------|------------|-------------|
| `DESCRIPTION` | CRITICAL | R package metadata: name, version (0.0.2.9000), authors, dependencies, license |
| `NAMESPACE` | HIGH | R exports: dvs_init, dvs_add, dvs_get, dvs_status |
| `README.md` | MEDIUM | User documentation, workflow examples, screenshots |
| `NEWS.md` | MEDIUM | Changelog: breaking changes, version history |
| `LICENSE` / `LICENSE.md` | LOW | MIT license text |
| `dvs.Rproj` | LOW | RStudio project configuration |
| `configure` | LOW | Unix build configuration script |
| `configure.win` | LOW | Windows build configuration script |
| `.Rbuildignore` | LOW | Files excluded from R package builds |
| `.gitignore` | LOW | Git ignore patterns |

---

## R/ Directory - R Interface Layer

| File | Importance | Lines | Key Functions | Description |
|------|------------|-------|---------------|-------------|
| `init.R` | CRITICAL | 31 | `dvs_init(storage_directory, permissions, group)` | Initialize DVS configuration, creates dvs.yaml, validates storage directory |
| `add.R` | CRITICAL | 50 | `dvs_add(files, message, split_output)` | Add files to versioned storage, creates .dvs metadata and .gitignore entries |
| `get.R` | CRITICAL | 47 | `dvs_get(files, split_output)` | Retrieve files from storage, restore to local project |
| `status.R` | CRITICAL | 49 | `dvs_status(files, split_output)` | Report file statuses (current/absent/unsynced/error) |
| `set_functions.R` | LOW | 16 | `normalize_paths(files)` | Path expansion utility (~), commented-out dvs_info |
| `extendr-wrappers.R` | HIGH | 33 | Auto-generated FFI bindings | .Call wrappers for Rust functions |

### R Function Parameters Summary

```r
dvs_init(storage_directory,           # Path to external storage
         permissions = NULL,          # Linux octal permissions (e.g., 664)
         group = NULL)                 # Linux group name

dvs_add(files,                         # File paths or glob patterns
        message = NULL,                # Commit-style message for metadata
        split_output = FALSE)          # Return single df or list(successes, failures)

dvs_get(files,                         # File paths or glob patterns
        split_output = FALSE)          # Return single df or list(successes, failures)

dvs_status(files = c(""),              # File paths/globs (empty = all tracked)
           split_output = FALSE)       # Return single df or list(successes, failures)
```

---

## src/rust/src/ Directory - Rust Core

### lib.rs - FFI Entry Point

| File | Importance | Lines | Description |
|------|------------|-------|-------------|
| `lib.rs` | CRITICAL | 578 | Main entry point, extendr FFI exports, R data frame conversion |

**Key Structs:**
- `RFile`: Combined success/error result for add/get operations
- `RFileSuccess`: Success-only result with all fields
- `RFileError`: Error-only result with error details
- `RStatusFile`: Status result with metadata fields
- `RInit`: Initialization result

**Exported Functions (extendr_module!):**
1. `dvs_init_impl(storage_dir, mode, group)` -> Result<Robj>
2. `dvs_add_impl(files_string, message, strict, split_output)` -> Result<Robj>
3. `dvs_get_impl(files_string, split_output)` -> Result<Robj>
4. `dvs_status_impl(files, split_output)` -> Result<Robj>
5. `get_file_info_impl(paths, split_output)` -> Robj
6. `parse_files_from_globs_add_impl(globs)` -> Vec<String>
7. `parse_files_from_globs_get_impl(globs)` -> Result<Vec<String>>
8. `parse_files_from_globs_status_impl(globs)` -> Result<Vec<String>>
9. `is_explicit_path_impl(entry)` -> bool

---

### library/ Directory - Core Business Logic

| File | Importance | Lines | Key Functions | Description |
|------|------------|-------|---------------|-------------|
| `mod.rs` | HIGH | 5 | Module exports | Declares init, add, get, status, info modules |
| `init.rs` | CRITICAL | 166 | `dvs_init(storage_dir, octal_permissions, group_name)` | Creates dvs.yaml config, validates/creates storage directory, checks git repo |
| `add.rs` | CRITICAL | 143 | `add(files, message_in, strict)`, `add_file(...)` | Hashes files, creates metadata, copies to storage |
| `get.rs` | CRITICAL | 84 | `get(files)`, `get_file(local_path, storage_dir, git_dir)` | Reads metadata, restores files from storage |
| `status.rs` | CRITICAL | 81 | `status(files)`, `status_file(local_path)` | Compares local hash vs metadata, returns status |
| `info.rs` | MEDIUM | 56 | `info(paths)` | Gets file ownership, permissions, timestamps |

**Key Structs by File:**

`init.rs`:
```rust
pub struct Init {
    pub storage_directory: PathBuf,
    pub group: String,
    pub permissions: i32
}
```

`add.rs`:
```rust
pub struct AddedFile {
    pub relative_path: PathBuf,
    pub outcome: Outcome,       // Copied, Present, Error
    pub size: u64,
    pub blake3_checksum: String,
    pub absolute_path: PathBuf,
}
```

`get.rs`:
```rust
pub struct RetrievedFile {
    pub relative_path: PathBuf,
    pub outcome: Outcome,
    pub size: u64,
    pub absolute_path: PathBuf,
    pub blake3_checksum: String,
}
```

`status.rs`:
```rust
pub struct FileStatus {
    pub relative_path: Option<PathBuf>,
    pub status: Status,         // Absent, Unsynced, Current, Error
    pub size: u64,
    pub add_time: String,
    pub saved_by: String,
    pub message: String,
    pub absolute_path: Option<PathBuf>,
    pub blake3_checksum: String
}
```

---

### helpers/ Directory - Utility Modules

| File | Importance | Lines | Key Functions | Description |
|------|------------|-------|---------------|-------------|
| `mod.rs` | HIGH | 10 | Module exports | Declares all helper modules |
| `error.rs` | CRITICAL | 152 | Error types and conversions | `FileError`, `FileErrorType`, `BatchError`, `BatchErrorType`, `InitError`, `InitErrorType` |
| `hash.rs` | CRITICAL | 115 | `hash_file_with_blake3(file_path)`, `get_file_hash(local_path)`, `get_storage_path(storage_dir, file_hash)` | Blake3 hashing with memmap optimization, cache integration |
| `copy.rs` | HIGH | 100 | `copy_impl(src, dest)`, `copy(local, storage)`, `copy_file_to_storage_directory(...)`, `set_file_permissions(mode, path)`, `set_group(group, path)` | File copying with permission/group setting |
| `file.rs` | HIGH | 193 | `Metadata` struct, `save(metadata, path)`, `load(path)`, `metadata_path(path)`, `get_user_name(path)`, `get_file_size(path)` | Metadata file operations (.dvs JSON files) |
| `config.rs` | HIGH | 81 | `Config` struct, `read(root_dir)`, `write(config, dir)`, `get_mode_u32(perms)`, `get_group(name)`, `get_storage_dir(path)` | dvs.yaml configuration handling |
| `ignore.rs` | HIGH | 56 | `add_gitignore_entry(local_path)`, `add_gitignore_entry_helper(path)` | .gitignore file management |
| `parse.rs` | HIGH | 244 | `parse_files_from_globs_add(globs)`, `parse_files_from_globs_get(globs)`, `parse_files_from_globs_status(globs)`, `is_explicit_path(entry)`, `get_all_meta_files()` | Glob pattern expansion, file filtering |
| `repo.rs` | HIGH | 117 | `get_nearest_repo_dir(dir)`, `get_relative_path(root, file)`, `check_file_in_git_repo(path, git_dir)`, `dir_in_git_repo(path, git_dir)`, `is_directory_empty(dir)` | Git repository detection and path utilities |
| `cache.rs` | MEDIUM | 89 | `CacheData` struct, `get_cached_hash(path)`, `write_hash_to_cache(path, hash)`, `get_cache_path(abs_path)` | XDG-compliant hash caching (~/.cache/dvs/) |
| `outcome.rs` | MEDIUM | 41 | `Outcome` enum (Copied/Present/Error), `Status` enum (Absent/Unsynced/Current/Error) | Operation result types |

**Key Data Structures:**

`file.rs - Metadata`:
```rust
pub struct Metadata {
    pub blake3_checksum: String,
    pub size: u64,
    pub add_time: String,       // ISO 8601 format
    pub message: String,
    pub saved_by: String        // Username who added
}
```

`config.rs - Config`:
```rust
pub struct Config {
    pub storage_dir: PathBuf,
    pub permissions: Option<i32>,    // Octal permissions
    pub group: Option<String>        // Linux group name
}
```

`error.rs - Error Types`:
```rust
pub enum FileErrorType {
    RelativePathNotFound, FileNotInGitRepo, AbsolutePathNotFound,
    PathIsDirectory, HashNotFound, SizeNotFound, OwnerNotFound,
    GroupNotSet, PermissionsNotSet, MetadataNotSaved,
    GitIgnoreNotAdded, FileNotCopied, MetadataNotLoaded, FileNotAdded
}

pub enum BatchErrorType {
    AnyFilesDNE, GitRepoNotFound, ConfigNotFound, GroupNotFound,
    StorageDirNotFound, PermissionsInvalid, AnyMetaFilesDNE
}

pub enum InitErrorType {
    ProjAlreadyInited, StorageDirNotCreated, StorageDirPermsNotSet,
    StorageDirNotADir, StorageDirAbsPathNotFound, GitRepoNotFound,
    ConfigNotCreated, GroupNotFound, PermissionsInvalid, DirEmptyNotChecked
}
```

---

## tests/testthat/ Directory - Test Suite

| File | Importance | Lines | Test Count | Description |
|------|------------|-------|------------|-------------|
| `helpers.R` | HIGH | 111 | N/A | Test utilities: `create_project()`, `create_project_and_initialize_dvs()`, `create_project_and_initialize_real_repo()` |
| `test-dvs_init.R` | HIGH | 412 | 27 | Initialization tests: first run, idempotency, error cases |
| `test-dvs_add.R` | HIGH | 955 | 55+ | Add file tests: single/multiple files, globs, errors, output formats |
| `test-dvs_get.R` | HIGH | 362 | 21 | Get file tests: retrieval, error handling, split output |
| `test-dvs_status.R` | HIGH | 469 | 20 | Status tests: current/absent/unsynced states |
| `test-integrated1.R` | MEDIUM | 72 | 3 | Integration: update workflow (add→modify→add) |
| `test-integrated2.R` | MEDIUM | 61 | 1 | Integration: get workflow (add→delete→get) |
| `test-integrated3.R` | MEDIUM | - | - | Additional integration tests |
| `test-integrated4.R` | MEDIUM | - | - | Additional integration tests |
| `testthat.R` | LOW | - | - | Test runner setup |

**Test ID Convention:**
- `UNI-*`: Unit tests (automated)
- `MAN-*`: Manual review tests (skipped)
- `INT-*`: Integration tests

---

## man/ Directory - Generated Documentation

| File | Importance | Description |
|------|------------|-------------|
| `dvs_init.Rd` | LOW | roxygen2 docs for dvs_init |
| `dvs_add.Rd` | LOW | roxygen2 docs for dvs_add |
| `dvs_get.Rd` | LOW | roxygen2 docs for dvs_get |
| `dvs_status.Rd` | LOW | roxygen2 docs for dvs_status |

---

## .github/workflows/ Directory - CI/CD

| File | Importance | Description |
|------|------------|-------------|
| `R-CMD-check.yaml` | MEDIUM | R package check workflow |
| `pkgdown.yml` | LOW | Documentation site generation |

---

## src/ Directory - Build Configuration

| File | Importance | Description |
|------|------------|-------------|
| `entrypoint.c` | HIGH | C entry point for R native routines |
| `Makevars` / `Makevars.in` | MEDIUM | Unix build flags |
| `Makevars.win` / `Makevars.win.in` / `Makevars.ucrt` | MEDIUM | Windows build flags |
| `dvs-win.def` / `Rdevious-win.def` | LOW | Windows DLL exports |
| `rust/Cargo.toml` | HIGH | Rust dependencies and build configuration |
| `rust/Cargo.lock` | MEDIUM | Locked dependency versions |

---

## Summary: Files to Read for Common Tasks

### Understanding Core Functionality
1. `R/init.R`, `R/add.R`, `R/get.R`, `R/status.R`
2. `src/rust/src/lib.rs`
3. `src/rust/src/library/*.rs`

### Debugging Errors
1. `src/rust/src/helpers/error.rs`
2. `src/rust/src/lib.rs` (error conversion)
3. Relevant `library/*.rs` file

### Adding New Features
1. Review `DESCRIPTION` for dependencies
2. Add R wrapper in `R/`
3. Add Rust impl in `library/`
4. Use helpers from `helpers/`
5. Add tests in `tests/testthat/`

### Performance Optimization
1. `src/rust/src/helpers/hash.rs` (Blake3, memmap, rayon)
2. `src/rust/src/helpers/cache.rs` (hash caching)

### Git/Repository Integration
1. `src/rust/src/helpers/repo.rs`
2. `src/rust/src/helpers/ignore.rs`
