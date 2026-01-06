# DVS Test Index

This document catalogs all test cases, organized by functionality and test ID for quick lookup and understanding of tested behaviors.

---

## Test ID Convention

- **UNI-***: Unit tests (automated, always run)
- **MAN-***: Manual review tests (skipped, require human verification)
- **INT-***: Integration tests (automated, multi-operation workflows)

---

## Test Summary by File

| File | Test Count | Coverage |
|------|------------|----------|
| test-dvs_init.R | 27 | Initialization, config, errors |
| test-dvs_add.R | 55+ | Add files, metadata, gitignore, errors |
| test-dvs_get.R | 21 | Retrieve files, error handling |
| test-dvs_status.R | 20 | Status reporting, all states |
| test-integrated1.R | 3 | Update workflow |
| test-integrated2.R | 1 | Get workflow |
| test-integrated3.R | TBD | Additional integration |
| test-integrated4.R | TBD | Additional integration |

---

## dvs_init Tests (test-dvs_init.R)

### Successful Initialization

| Test ID | Description | Key Assertions |
|---------|-------------|----------------|
| UNI-INI-001 | Init works first run | Storage dir created, dvs.yaml created, correct df output |
| UNI-INI-002 | Init works second run with same inputs | No-op behavior, yaml not modified |
| UNI-INI-004 | Init works with existing storage_dir | Storage dir permissions unchanged |
| UNI-INI-006 | Init works with permissions and group | Custom settings stored correctly |
| UNI-INI-007 | Init works after updating yaml manually | New settings accepted |
| UNI-INI-020 | Default permissions and group applied to stored files | Mode 664, default group |

### Error Cases

| Test ID | Description | Expected Error |
|---------|-------------|----------------|
| UNI-INI-003 | Init with different attributes fails | "project already initialized" |
| UNI-INI-005 | Init outside git repo fails | "git repository not found" |
| UNI-INI-013 | Storage dir can't be created | "storage directory not created" |
| UNI-INI-014 | Config file can't be created | "configuration file not created" |
| UNI-INI-015 | Invalid group name | "linux primary group not found" |
| UNI-INI-016 | Invalid permissions format | "linux file permissions invalid" |
| UNI-INI-018 | Storage dir can't be checked | "could not check if storage directory is empty" |

### Manual Review Tests

| Test ID | Description |
|---------|-------------|
| MAN-INI-001 | Warning when storage dir not empty |
| MAN-INI-002 | Warning when storage dir has file extension |
| MAN-INI-003 | Warning when storage dir is in project dir |
| MAN-INI-004 | Message when storage dir is created |
| MAN-INI-005 | Message when storage dir already exists |
| MAN-INI-007 | Error when storage dir permissions can't be set |

---

## dvs_add Tests (test-dvs_add.R)

### Successful Addition

| Test ID | Description | Key Assertions |
|---------|-------------|----------------|
| UNI-ADD-001 | Add single file | Metadata created, gitignore updated, file in storage |
| UNI-ADD-002 | Add multiple files (same dir) | Both files versioned, single gitignore |
| UNI-ADD-003 | Add files in different directories | Separate gitignore per directory |
| UNI-ADD-004 | Filters .dvs and .gitignore from globs | Only data files added |
| UNI-ADD-008 | Add via metadata file name (.dvs) | File versioned correctly |
| UNI-ADD-009 | Single df output format | Returns data.frame |
| UNI-ADD-010 | Split df output format | Returns list(successes, failures) |

### Metadata and Gitignore

| Test ID | Description | Key Assertions |
|---------|-------------|----------------|
| UNI-ADD-011 | Creates .gitignore if missing | File created with dvs entries |
| UNI-ADD-012 | Appends to existing .gitignore | Original content preserved |
| UNI-ADD-013 | Metadata file fields | blake3_checksum, size, add_time, message, saved_by |

### Output Format

| Test ID | Description | Key Assertions |
|---------|-------------|----------------|
| UNI-ADD-014 | Single df columns | relative_path, outcome, size, blake3_checksum, absolute_path, input, error, error_message |
| UNI-ADD-015 | Split df columns | Success df + Failure df with appropriate columns |

### Error Cases (Batch)

| Test ID | Description | Expected Error |
|---------|-------------|----------------|
| UNI-ADD-005 | File doesn't exist | Error thrown |
| UNI-ADD-006 | Not in git repo | Error thrown |
| UNI-ADD-007 | Not initialized | "configuration file not found" |
| UNI-ADD-016 | Storage dir deleted after init | "storage directory not found" |
| UNI-ADD-017 | Invalid permissions in config | "linux file permissions invalid" |
| UNI-ADD-018 | Invalid group in config | "linux primary group not found" |

### Error Cases (File-level)

| Test ID | Description | Error Type |
|---------|-------------|------------|
| UNI-ADD-021 | Path is directory | "path is a directory" |
| UNI-ADD-022 | Can't hash file (permissions) | "file hash not found" |
| UNI-ADD-023 | File outside git repo | "file not in git repository" |
| UNI-ADD-026 | Can't save metadata | "metadata file not saved" |
| UNI-ADD-027 | Can't update gitignore | "gitignore entry not saved" |
| UNI-ADD-028 | Can't set group | "linux primary group not set" |
| UNI-ADD-030 | Can't copy to storage | "file not copied" |

### Cleanup Behavior

| Test ID | Description | Key Assertions |
|---------|-------------|----------------|
| UNI-ADD-031 | Error → file not in storage | No partial state |
| UNI-ADD-032 | Copy error → cleanup metadata | .dvs file removed |

### Manual Review Tests

| Test ID | Description |
|---------|-------------|
| MAN-ADD-001 | Linux permissions can't be set |
| MAN-ADD-002 | File owner can't be found |
| MAN-ADD-003 | File size can't be found |
| MAN-ADD-004 | Absolute path can't be found |
| MAN-ADD-005 | Relative path can't be found |

---

## dvs_get Tests (test-dvs_get.R)

### Successful Retrieval

| Test ID | Description | Key Assertions |
|---------|-------------|----------------|
| UNI-GET-007 | Get multiple files explicit | Both retrieved |
| UNI-GET-008 | Get multiple files via glob | Only tracked files retrieved |
| UNI-GET-009 | Get by metadata file name | File restored, correct path |

### No-op Behavior

| Test ID | Description | Key Assertions |
|---------|-------------|----------------|
| UNI-GET-005 | Non-added file in glob | Zero rows returned |

### Output Format

| Test ID | Description | Key Assertions |
|---------|-------------|----------------|
| UNI-GET-011 | Split df output | Success + Failure dfs |
| UNI-GET-012 | Single df output | All columns present |

### Error Cases (Batch)

| Test ID | Description | Expected Error |
|---------|-------------|----------------|
| UNI-GET-001 | Outside git repo | "git repository not found" |
| UNI-GET-002 | File not added | "metadata file not found" |
| UNI-GET-003 | File doesn't exist | "metadata file not found" |
| UNI-GET-004 | Random input | "metadata file not found" |
| UNI-GET-006 | Not initialized | "configuration file not found" |
| UNI-GET-014 | Storage dir deleted | "storage directory not found" |

### Error Cases (File-level)

| Test ID | Description | Error Type |
|---------|-------------|------------|
| UNI-GET-018 | File outside git repo | "file not in git repository" |
| UNI-GET-020 | Invalid metadata JSON | "metadata file not loaded" |
| UNI-GET-021 | File missing from storage | "file not copied" |

### Manual Review Tests

| Test ID | Description |
|---------|-------------|
| MAN-GET-001 | User retrieves file versioned by another user |
| MAN-GET-002 | Absolute path can't be found |
| MAN-GET-003 | Relative path can't be found |
| MAN-GET-004 | File size can't be found |
| MAN-GET-005 | File contents can't be hashed |

---

## dvs_status Tests (test-dvs_status.R)

### Status Values

| Test ID | Description | Expected Status |
|---------|-------------|-----------------|
| UNI-STA-001 | No files added | 0 rows |
| UNI-STA-002 | File is current | "current" |
| UNI-STA-008 | Metadata unreadable | "error" |
| UNI-STA-009 | Mix of added/not added | "current" + "error" |

### Input Handling

| Test ID | Description | Key Assertions |
|---------|-------------|----------------|
| UNI-STA-003 | Single file input | 1 row returned |
| UNI-STA-004 | Glob pattern (*) | Matches tracked files |
| UNI-STA-005 | Glob pattern (*.txt) | Extension filtering |
| UNI-STA-010 | Multiple files via glob | Only tracked files |
| UNI-STA-013 | Input by metadata filename | Correct status |
| UNI-STA-014 | Glob excludes non-metadata files | Only .dvs files matched |

### Output Format

| Test ID | Description | Key Assertions |
|---------|-------------|----------------|
| UNI-STA-011 | Single df columns | All status fields present |
| UNI-STA-012 | Split df output | Success + Failure dfs |

### No-op Verification

| Test ID | Description | Key Assertions |
|---------|-------------|----------------|
| UNI-STA-016 | Status doesn't modify files | All timestamps unchanged |

### Error Cases (Batch)

| Test ID | Description | Expected Error |
|---------|-------------|----------------|
| UNI-STA-006 | Not initialized | "could not load configuration file" |
| UNI-STA-007 | Not in git repo | "could not find git repo root" |
| UNI-STA-017 | Not initialized (via function) | "configuration file not found" |

### Error Cases (File-level)

| Test ID | Description | Error Type |
|---------|-------------|------------|
| UNI-STA-018 | Path is directory | "path is a directory" |
| UNI-STA-019 | File not added | "file not added" |
| UNI-STA-020 | Can't hash file | "file hash not found" |

---

## Integration Tests

### test-integrated1.R - Update Workflow

| Test ID | Description | Workflow |
|---------|-------------|----------|
| INT-ADD-001 | Add, modify, re-add file | add→status(current)→modify→status(unsynced)→add→status(current) |

### test-integrated2.R - Get Workflow

| Test ID | Description | Workflow |
|---------|-------------|----------|
| INT-GET-001 | Add, get (present), delete, get (copied) | add→get(present)→delete→status(absent)→get(copied) |

---

## Test Helper Functions (helpers.R)

| Function | Purpose |
|----------|---------|
| `create_project(proj_name)` | Creates fake git repo (just .git dir) |
| `create_project_and_initialize_dvs(proj_name, env)` | Project + dvs_init |
| `create_project_and_initialize_real_repo(proj_name, env)` | Real `git init` + dvs_init |
| `create_project_no_dvs_init(proj_name, env)` | Real git repo, no dvs_init |
| `near_system_time(time_string)` | Check timestamp is recent |
| `is_near_time(iso_time_string, threshold)` | Check timestamp within threshold |
| `group_exists_unix(group)` | Check if Unix group exists |

---

## Test Data Patterns

### Standard PK Data
```r
pk_data <- data.frame(
  USUBJID = c(1, 1, 1),
  NTFD = c(0.5, 1, 2),
  DV = c(379.444, 560.613, 0)
)
```

### Directory Structure in Tests
```
tempdir()/
├── projects/
│   └── {proj_name}/
│       ├── .git/
│       ├── dvs.yaml
│       └── data/derived/
│           ├── pk_data.csv
│           ├── pk_data.csv.dvs
│           └── .gitignore
└── data/dvs/
    └── {proj_name}/  # storage directory
```

---

## Running Tests

```r
# Run all tests
devtools::test()

# Run specific test file
testthat::test_file("tests/testthat/test-dvs_add.R")

# Run single test by name
testthat::test_that("can add a single file [UNI-ADD-001]", {...})
```
