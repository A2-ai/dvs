# DVS Product Specification

**Version**: 0.0.2.9000
**Status**: Development
**Authors**: Jenna Johnson, Devin Pastoor, A2-Ai
**Repository**: https://github.com/A2-ai/dvs

---

## Executive Summary

DVS (Data Versioning System) is an R package that enables teams to version large or sensitive data files in Git-managed projects without uploading the actual file contents to Git repositories. It provides a transparent workflow where data files are stored in an external shared directory while only lightweight metadata files are tracked in Git.

---

## Problem Statement

Teams collaborating on data-intensive projects face several challenges:

1. **Large File Limitations**: Git is not designed for large binary files; they bloat repositories and slow operations
2. **Sensitive Data Exposure**: Uploading sensitive data to Git (especially remote hosts) creates compliance and security risks
3. **Version Tracking Need**: Teams still need to track which version of data files are being used
4. **Collaboration Friction**: Without a system, teams resort to manual file sharing via email, Slack, or shared drives
5. **Reproducibility**: It's hard to know which data version was used for specific analyses

---

## Solution Overview

DVS solves these problems by:

1. **Separating storage from tracking**: Data files are copied to an external shared storage directory while Git tracks only small metadata files
2. **Content-addressable storage**: Files are stored by their content hash, enabling efficient deduplication
3. **Automatic Git integration**: DVS manages .gitignore entries to ensure data files aren't accidentally committed
4. **Simple R interface**: Four functions cover the complete workflow

---

## Target Users

### Primary Persona: Data Scientist / Analyst
- Works in R within RStudio or command line
- Collaborates with team via Git
- Handles datasets ranging from MB to GB
- Needs version tracking without Git complexity

### Secondary Persona: Team Lead / Data Engineer
- Sets up shared infrastructure
- Manages access controls via Linux groups
- Ensures reproducibility across team

---

## Core Features

### 1. Project Initialization (`dvs_init`)

**Purpose**: Configure a project to use DVS

**User Story**: As a data scientist, I want to set up DVS for my project so my team can share large files.

**Functionality**:
- Creates `dvs.yaml` configuration file in project root
- Validates/creates external storage directory
- Optionally configures Linux file permissions for stored files
- Optionally configures Linux group ownership for stored files

**Behavior**:
- Must be run from within a Git repository
- Storage directory can be outside the Git repo (recommended) or inside (with warning)
- Idempotent: running with same parameters is a no-op
- Re-running with different parameters returns an error (must edit dvs.yaml manually)
- Creates storage directory if it doesn't exist (with 0o770 permissions)
- Warns if storage directory is non-empty, has file extension, or is in git repo

**API**:
```r
dvs_init(
  storage_directory,      # Required: path to shared storage
  permissions = NULL,     # Optional: octal permissions (e.g., 664)
  group = NULL            # Optional: Linux group name
)
# Returns: data.frame with storage_directory, permissions, group
```

---

### 2. File Versioning (`dvs_add`)

**Purpose**: Version files by copying to storage and creating metadata

**User Story**: As a data scientist, I want to add my processed data files to DVS so my team can access them.

**Functionality**:
- Computes blake3 hash of file contents
- Creates `.dvs` metadata file containing:
  - `blake3_checksum`: Content hash
  - `size`: File size in bytes
  - `add_time`: ISO 8601 timestamp
  - `message`: User-provided description
  - `saved_by`: Username of person adding file
- Copies file to storage directory organized by hash prefix
- Updates/creates `.gitignore` to exclude data file and include metadata file
- Supports glob patterns and explicit paths
- Can reference files by their metadata path (e.g., `data.csv.dvs`)

**Behavior**:
- Requires initialized project (`dvs.yaml` exists)
- Requires files to be inside Git repository
- Filters out `.dvs` and `.gitignore` files from glob patterns
- Updates file if already tracked and content changed
- No-op if file content matches existing version (`outcome: "present"`)
- On error during copy, cleans up partial state (metadata + copied file)
- All explicit paths must exist (batch error if any missing)
- File-level errors appear in output data frame, not as thrown errors

**API**:
```r
dvs_add(
  files,                  # Required: file paths or glob patterns
  message = NULL,         # Optional: version message
  split_output = FALSE    # Optional: return list vs single df
)
# Returns: data.frame or list(successes=df, failures=df)
```

**Output Columns**:
- `relative_path`: Path relative to working directory
- `outcome`: "copied" | "present" | "error"
- `size`: File size in bytes
- `blake3_checksum`: Content hash
- `absolute_path`: Full path
- `input`: Original input (for errors)
- `error`: Error type (for errors)
- `error_message`: Detailed message (for errors)

---

### 3. File Retrieval (`dvs_get`)

**Purpose**: Restore files from storage to local project

**User Story**: As a data scientist, I want to retrieve the latest data files after pulling from Git.

**Functionality**:
- Reads `.dvs` metadata file to get expected hash
- Computes local file hash (if exists) and compares
- If file missing or outdated: copies from storage
- If file current: no-op (`outcome: "present"`)
- Supports glob patterns and explicit paths
- Can reference files by their metadata path

**Behavior**:
- Requires initialized project
- Requires metadata file to exist for explicit paths (batch error otherwise)
- Glob patterns return 0 rows for non-tracked files (no error)
- File-level errors appear in output, not as thrown errors

**API**:
```r
dvs_get(
  files,                  # Required: file paths or glob patterns
  split_output = FALSE    # Optional: return list vs single df
)
# Returns: same structure as dvs_add()
```

---

### 4. Status Reporting (`dvs_status`)

**Purpose**: Show synchronization status of tracked files

**User Story**: As a data scientist, I want to see which files need to be updated.

**Functionality**:
- Reads `.dvs` metadata files
- Compares local file hash with stored hash
- Reports status for each file
- No file modifications (read-only operation)
- Empty input returns status of all tracked files in repository

**Statuses**:
- `current`: Local file exists and matches stored version
- `absent`: Metadata exists but local file is missing
- `unsynced`: Local file exists but differs from stored version
- `error`: Could not determine status

**Behavior**:
- Requires initialized project
- Glob patterns filter to files with existing metadata
- Explicit non-tracked files return error status
- Never modifies files, metadata, or gitignore

**API**:
```r
dvs_status(
  files = c(""),          # Optional: empty = all tracked files
  split_output = FALSE    # Optional: return list vs single df
)
# Returns: data.frame or list(successes=df, failures=df)
```

**Output Columns** (in addition to add/get columns):
- `status`: "current" | "absent" | "unsynced" | "error"
- `add_time`: When file was last added
- `saved_by`: Who last added the file
- `message`: Last add message

---

## User Journeys

### Journey 1: Initial Setup

**Actor**: Team Lead setting up infrastructure

```
1. Create shared storage directory on network drive
   $ mkdir -p /data/shared/project-x-dvs
   $ chgrp project-team /data/shared/project-x-dvs
   $ chmod 2770 /data/shared/project-x-dvs

2. Initialize DVS in project repository
   > dvs_init("/data/shared/project-x-dvs", permissions = 664, group = "project-team")

3. Commit configuration
   $ git add dvs.yaml
   $ git commit -m "Initialize DVS"
   $ git push
```

### Journey 2: Adding Data Files

**Actor**: Data Scientist after processing data

```
1. Complete data processing in R
   > write.csv(pk_data, "data/derived/pk_data.csv")

2. Version the output file
   > dvs_add("data/derived/pk_data.csv", message = "Initial PK dataset v1")
   # Returns: outcome="copied", blake3_checksum="abc123..."

3. Commit the metadata
   $ git add data/derived/pk_data.csv.dvs data/derived/.gitignore
   $ git commit -m "Add processed PK data"
   $ git push

4. Verify status
   > dvs_status("data/derived/pk_data.csv")
   # Returns: status="current"
```

### Journey 3: Getting Latest Files

**Actor**: Team Member after pulling changes

```
1. Pull latest from Git
   $ git pull
   # .dvs files updated, but data files not present

2. Check what needs to be retrieved
   > dvs_status()
   # Returns: status="absent" for new files

3. Retrieve all tracked files
   > dvs_get("data/derived/*")
   # Returns: outcome="copied" for each file

4. Verify
   > dvs_status()
   # Returns: status="current" for all files
```

### Journey 4: Updating Data Files

**Actor**: Data Scientist updating existing data

```
1. Modify data processing
   > pk_data_v2 <- update_processing(pk_data)
   > write.csv(pk_data_v2, "data/derived/pk_data.csv")

2. Check status
   > dvs_status("data/derived/pk_data.csv")
   # Returns: status="unsynced"

3. Update the version
   > dvs_add("data/derived/pk_data.csv", message = "Updated PK dataset v2")
   # Returns: outcome="copied" (new hash)

4. Commit updated metadata
   $ git add data/derived/pk_data.csv.dvs
   $ git commit -m "Update PK data with new processing"
   $ git push
```

### Journey 5: Working with Multiple Files

**Actor**: Data Scientist with batch operations

```
1. Process multiple outputs
   > write.csv(pk_data, "data/derived/pk.csv")
   > write.csv(pd_data, "data/derived/pd.csv")
   > write.csv(summary_stats, "data/derived/summary.csv")

2. Add all at once using glob
   > dvs_add("data/derived/*.csv", message = "Analysis outputs batch 1")
   # Returns: 3 rows, all outcome="copied"

3. Later, retrieve all
   > dvs_get("data/derived/*.csv")

4. Check status of everything
   > dvs_status()  # Empty input = all tracked files
```

---

## Technical Specifications

### Configuration File (dvs.yaml)

```yaml
storage_dir: /absolute/path/to/storage
permissions: 664          # Optional, octal format (default: 664)
group: team-name          # Optional (default: no group assignment)
```

**Location**: Project root (same directory as .git)

**Defaults**:
- `permissions`: 664 (octal) - owner rw, group rw, others r
- `group`: Empty string (no group assignment to stored files)

---

### Metadata File Format (.dvs)

```json
{
  "blake3_checksum": "64-character-hex-string",
  "size": 12345,
  "add_time": "2024-01-15T10:30:45.123Z",
  "message": "User-provided description",
  "saved_by": "username"
}
```

**Location**: Same directory as data file, with `.dvs` extension appended

**Field Details**:
- `blake3_checksum`: 64-character hexadecimal string (256-bit hash)
- `size`: File size in bytes (unsigned 64-bit integer)
- `add_time`: ISO 8601 timestamp with millisecond precision (`%Y-%m-%dT%H:%M:%S%.3fZ`)
- `message`: User-provided string (empty string if not provided)
- `saved_by`: System username retrieved from file ownership

---

### Storage Directory Structure

```
/storage/directory/           # Created with 0o770 permissions
├── ab/
│   └── cdef1234...5678       # Full file contents (62 remaining chars)
├── cd/
│   └── 5678efgh...1234
└── ...
```

- **Directory permissions**: Always 0o770 (rwxrwx---)
- **File structure**: First 2 characters of hash → subdirectory name
- **File naming**: Remaining 62 characters of hash → filename
- **File permissions**: Configured per project (default 664)
- **Content-addressed deduplication**: Identical file contents share the same storage location

---

### Gitignore Entry Format

```gitignore
# dvs entry
/datafile.csv
!/datafile.csv.dvs
```

**Location**: Same directory as data file

**Behavior**:
- Creates `.gitignore` if not present
- Appends entries if `.gitignore` exists
- Does not duplicate entries on repeated operations

---

### Hash Caching System

**Purpose**: Avoid re-computing hashes for unchanged files

**Location**: `~/.cache/dvs/{project-name}/{relative-path}` (XDG Base Directory spec)

**Contents**:
```json
{
  "hash": "blake3-hash",
  "modification_time": "system-timestamp"
}
```

**Cache Behavior**:
- Cache is checked before computing hash
- Cache is invalidated if file modification time differs
- No built-in cache clearing mechanism (manual deletion: `rm -rf ~/.cache/dvs/`)

---

### Glob Pattern Behavior

**Important**: Glob handling differs between operations:

| Operation | Glob Behavior | Explicit Path Behavior |
|-----------|---------------|----------------------|
| `dvs_add` | Expands to existing files, filters `.dvs`/`.gitignore` | Error if file doesn't exist |
| `dvs_get` | Only matches files with existing `.dvs` metadata | Error if no metadata file |
| `dvs_status` | Only matches files with existing `.dvs` metadata | Error status if no metadata |

**Glob symbols**: `*`, `?`, `[`, `]`, `{`, `}` indicate a glob pattern (vs explicit path)

**Tilde expansion**: `~` is expanded to home directory before processing

---

### Console Warnings

DVS prints warnings to the console (not errors) for certain conditions:

| Condition | Warning Message |
|-----------|-----------------|
| Storage dir has file extension | `"warning: file path inputted as storage directory. Is this intentional?"` |
| Storage dir not empty | `"warning: storage directory not empty"` |
| Storage dir inside git repo | `"warning: the storage directory is located in the git repo directory..."` |
| No files queued for operation | `"warning: no paths queued to add/get to dvs"` |

---

## Non-Functional Requirements

### Performance
- Blake3 hashing with memory-mapped I/O for files >= 16KB
- Files < 16KB use traditional read for efficiency
- Parallel hash computation using rayon for large files
- Local hash caching (XDG cache) avoids re-hashing unchanged files

### Security
- File permissions configurable per project
- Group ownership configurable per project
- No data stored in Git (only hashes)
- Storage directory should be on trusted infrastructure

### Compatibility
- Rust toolchain >= 1.78.0
- R with extendr FFI support
- Linux/Unix for permission/group features
- Git repository required

---

## Advanced Behavior Details

### Idempotent Operations

**dvs_add**:
- If file already in storage with matching hash: `outcome="present"`, no copy occurs
- If file content changed: `outcome="copied"`, new version stored (old version remains)

**dvs_get**:
- If local file exists with matching hash: `outcome="present"`, no copy occurs
- If local file missing or differs: `outcome="copied"`, file restored from storage

### Error Recovery (Strict Mode)

DVS operates in "strict mode" which ensures atomicity:
- If metadata save succeeds but file copy fails, metadata is rolled back (deleted)
- If file copy partially completes, the partial file is deleted
- This prevents inconsistent states where metadata exists but storage file doesn't

### Hash-Based Deduplication

Two users adding identical file content:
- Both files share the same storage location (same hash → same path)
- This is transparent to users
- Reduces storage requirements for duplicate content

### Multi-User Considerations

**Group membership**:
- User must be a member of the configured group to set group ownership
- Error if user not in group: `"linux primary group not set: {group} *nix error"`

**File ownership**:
- `saved_by` field records the Unix username of the user who ran `dvs_add`
- Retrieved from file ownership, not from environment variables

### Working Directory Dependency

All DVS operations are relative to the current working directory:
- Git repository is found by walking up from current directory
- Relative paths in output are relative to current directory
- For predictable behavior, always run from project root

### Output Data Frame Structure

**Single output (`split_output=FALSE`)**:
- One data frame with all results
- Error rows have `outcome="error"`, `error`, `error_message` columns populated
- Success rows have `NA` in error columns

**Split output (`split_output=TRUE`)**:
- `$successes`: Data frame of successful operations (no error columns)
- `$failures`: Data frame of failed operations (with error columns)
- Empty categories may be omitted from the list

### Metadata File Reference Syntax

Users can reference files by their metadata filename:
```r
# These are equivalent:
dvs_add("data.csv")
dvs_add("data.csv.dvs")  # .dvs is stripped to find actual file

dvs_get("data.csv")
dvs_get("data.csv.dvs")  # Gets the actual data file
```

---

## Constraints and Limitations

### Current Limitations

1. **Platform Support**: Linux permissions/groups; Windows support limited
2. **Git Requirement**: Must be within a Git repository
3. **Single Storage**: One storage directory per project
4. **No Remote Storage**: Storage must be filesystem-accessible
5. **No History Browsing**: Can't retrieve old versions directly (need Git history)
6. **No Deletion**: No built-in way to remove files from storage
7. **No Garbage Collection**: Storage can accumulate orphaned files

### Design Decisions

1. **Hash Algorithm**: Blake3 chosen for speed and security
2. **Metadata Format**: JSON for human readability
3. **Storage Partitioning**: 2-char prefix to avoid directory limits
4. **Rust Core**: Performance-critical operations in Rust
5. **R Interface**: Target audience uses R

---

## Future Considerations

### Potential Enhancements

1. **Remote Storage**: S3, GCS, or other cloud storage backends
2. **Version History**: Ability to retrieve previous versions
3. **Garbage Collection**: Clean up orphaned storage files
4. **File Deletion**: Remove files from tracking
5. **Partial Sync**: Download only specific files
6. **Compression**: Optional compression in storage
7. **Encryption**: Optional encryption for sensitive data
8. **CLI Tool**: Command-line interface for non-R workflows
9. **Python Bindings**: Cross-language support

---

## Glossary

| Term | Definition |
|------|------------|
| Storage Directory | External directory where file contents are stored |
| Metadata File | `.dvs` JSON file containing file version information |
| Hash | Blake3 cryptographic hash of file contents |
| Tracked File | A file with an associated `.dvs` metadata file |
| Current | File exists locally and matches stored version |
| Absent | Metadata exists but local file is missing |
| Unsynced | Local file differs from stored version |
| Outcome | Result of add/get operation (copied, present, error) |
