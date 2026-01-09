# DVS Error Index

This document catalogs all error types, their causes, and how to resolve them.

---

## Error Hierarchy

```
                        ┌─────────────────┐
                        │   User sees     │
                        │   R error or    │
                        │   df column     │
                        └────────┬────────┘
                                 │
        ┌────────────────────────┼────────────────────────┐
        │                        │                        │
        ▼                        ▼                        ▼
┌───────────────┐      ┌───────────────┐      ┌───────────────┐
│  InitError    │      │  BatchError   │      │  FileError    │
│  (R throws)   │      │  (R throws)   │      │  (in df row)  │
└───────────────┘      └───────────────┘      └───────────────┘
```

---

## Initialization Errors (InitError)

These errors are **thrown** as R errors with class `dvs_init_error`.

| Error Type | Message | Cause | Resolution |
|------------|---------|-------|------------|
| `GitRepoNotFound` | "git repository not found" | Not inside a git repo | Run `git init` or navigate to git repo |
| `ProjAlreadyInited` | "project already initialized" | dvs.yaml exists with different settings | Edit dvs.yaml manually or use same parameters |
| `GroupNotFound` | "linux primary group not found" | Specified group doesn't exist | Check group name with `groups` command |
| `PermissionsInvalid` | "linux file permissions invalid" | Invalid octal format | Use valid octal (e.g., 664, 777) |
| `StorageDirNotCreated` | "storage directory not created" | Permission denied on parent | Check parent directory permissions |
| `StorageDirNotADir` | "storage directory input is not a directory" | Path points to file | Use a directory path |
| `StorageDirAbsPathNotFound` | "storage directory absolute path not found" | Cannot resolve path | Check path validity |
| `StorageDirPermsNotSet` | "storage directory permissions not set" | Permission denied | Check filesystem permissions |
| `ConfigNotCreated` | "configuration file not created (dvs.yaml)" | Permission denied on project root | Check project directory permissions |
| `DirEmptyNotChecked` | "could not check if storage directory is empty" | Permission denied on storage dir | Check storage directory permissions |

---

## Batch Errors (BatchError)

These errors are **thrown** as R errors (various classes).

| Error Type | Message | Cause | Resolution |
|------------|---------|-------|------------|
| `GitRepoNotFound` | "git repository not found" | Not inside a git repo | Navigate to git repo |
| `ConfigNotFound` | "configuration file not found (dvs.yaml)" | DVS not initialized | Run `dvs_init()` first |
| `GroupNotFound` | "linux primary group not found" | Group in dvs.yaml doesn't exist | Update dvs.yaml or join group |
| `StorageDirNotFound` | "storage directory not found" | Storage dir in dvs.yaml missing | Create directory or update dvs.yaml |
| `PermissionsInvalid` | "linux file permissions invalid" | Bad permissions in dvs.yaml | Update permissions field in dvs.yaml |
| `AnyFilesDNE` | "at least one inputted file not found" | Explicit file doesn't exist | Check file paths (dvs_add only) |
| `AnyMetaFilesDNE` | "metadata file not found for at least one file" | File not yet added | Use `dvs_add()` first (dvs_get only) |

---

## File Errors (FileError)

These errors appear in **data frame output** rows, not as thrown errors.

| Error Type | Message | Cause | Resolution |
|------------|---------|-------|------------|
| `RelativePathNotFound` | "relative path not found" | Cannot compute relative path | Check working directory |
| `AbsolutePathNotFound` | "absolute path not found" | Cannot resolve path | Check path exists |
| `FileNotInGitRepo` | "file not in git repository" | File outside git repo root | Move file into repo |
| `PathIsDirectory` | "path is a directory" | Passed directory to add/get | Use glob pattern or file path |
| `HashNotFound` | "file hash not found" | Cannot read file | Check file permissions |
| `SizeNotFound` | "file size not found" | Cannot stat file | Check file permissions |
| `OwnerNotFound` | "file owner not found" | Cannot get ownership info | Check file exists |
| `GroupNotSet` | "linux primary group not set" | User not in specified group | Join group or update dvs.yaml |
| `PermissionsNotSet` | "linux file permissions not set" | Cannot chmod file | Check filesystem permissions |
| `MetadataNotSaved` | "metadata file not saved" | Cannot write .dvs file | Check directory permissions |
| `MetadataNotLoaded` | "metadata file not loaded" | Invalid .dvs JSON | Check .dvs file contents |
| `GitIgnoreNotAdded` | "gitignore entry not saved" | Cannot write .gitignore | Check directory permissions |
| `FileNotCopied` | "file not copied" | Copy to/from storage failed | Check storage directory permissions |
| `FileNotAdded` | "file not added" | No .dvs file exists | Run `dvs_add()` first |

---

## Error by Function

### dvs_init()

| Scenario | Error Type | Error Class |
|----------|------------|-------------|
| Not in git repo | InitError::GitRepoNotFound | dvs_init_error |
| Already initialized differently | InitError::ProjAlreadyInited | dvs_init_error |
| Invalid group name | InitError::GroupNotFound | dvs_init_error |
| Invalid permissions | InitError::PermissionsInvalid | dvs_init_error |
| Can't create storage dir | InitError::StorageDirNotCreated | dvs_init_error |
| Storage path is file | InitError::StorageDirNotADir | dvs_init_error |
| Can't write config | InitError::ConfigNotCreated | dvs_init_error |

### dvs_add()

| Scenario | Error Type | Error Class/Location |
|----------|------------|---------------------|
| Not in git repo | BatchError::GitRepoNotFound | dvs_get_error |
| Not initialized | BatchError::ConfigNotFound | dvs_get_error |
| Invalid group in config | BatchError::GroupNotFound | dvs_get_error |
| Storage dir missing | BatchError::StorageDirNotFound | dvs_get_error |
| Invalid perms in config | BatchError::PermissionsInvalid | dvs_get_error |
| File doesn't exist | BatchError::AnyFilesDNE | dvs_get_error |
| Path is directory | FileError::PathIsDirectory | df row |
| Can't hash file | FileError::HashNotFound | df row |
| File outside repo | FileError::FileNotInGitRepo | df row |
| Can't write metadata | FileError::MetadataNotSaved | df row |
| Can't update gitignore | FileError::GitIgnoreNotAdded | df row |
| Can't copy to storage | FileError::FileNotCopied | df row |
| Can't set group | FileError::GroupNotSet | df row |
| Can't set permissions | FileError::PermissionsNotSet | df row |

### dvs_get()

| Scenario | Error Type | Error Class/Location |
|----------|------------|---------------------|
| Not in git repo | BatchError::GitRepoNotFound | dvs_get_error |
| Not initialized | BatchError::ConfigNotFound | dvs_get_error |
| Storage dir missing | BatchError::StorageDirNotFound | dvs_get_error |
| Metadata file missing | BatchError::AnyMetaFilesDNE | dvs_get_error |
| Invalid metadata JSON | FileError::MetadataNotLoaded | df row |
| File outside repo | FileError::FileNotInGitRepo | df row |
| Can't copy from storage | FileError::FileNotCopied | df row |

### dvs_status()

| Scenario | Error Type | Error Class/Location |
|----------|------------|---------------------|
| Not in git repo | BatchError::GitRepoNotFound | dvs_get_error |
| Not initialized | BatchError::ConfigNotFound | dvs_get_error |
| Path is directory | FileError::PathIsDirectory | df row |
| File not added | FileError::FileNotAdded | df row |
| Invalid metadata | FileError::MetadataNotLoaded | df row |
| Can't hash file | FileError::HashNotFound | df row |

---

## Error Message Patterns

### Pattern: `{error_type}: {context_message}`

Examples:
```
"linux primary group not found: could not find group fake_group."
"storage directory not found: storage_dir: /path/to/storage in dvs.yaml, No such file or directory"
"configuration file not found (dvs.yaml): could not load configuration file..."
```

### Pattern: File errors include file path context

Examples:
```
error: "path is a directory"
error_message: NA
relative_path: "data/derived"
absolute_path: "/project/data/derived"
input: "data/derived"
```

---

## Troubleshooting Guide

### "git repository not found"
1. Verify you're in a git repository: `git status`
2. If not, run `git init`

### "configuration file not found (dvs.yaml)"
1. Run `dvs_init("/path/to/storage")` first
2. Check if dvs.yaml exists in project root

### "storage directory not found"
1. Check path in dvs.yaml
2. Create directory: `mkdir -p /path/to/storage`
3. Ensure path is accessible

### "metadata file not found for at least one file"
1. File not yet tracked: run `dvs_add("filename")`
2. Check if .dvs file exists alongside data file

### "project already initialized"
1. View current config: `cat dvs.yaml`
2. Either use same parameters or edit dvs.yaml directly

### "linux primary group not set"
1. Check if user is in group: `groups`
2. Add user to group: `sudo usermod -aG groupname username`
3. Re-login for group change to take effect

### "file not copied"
1. Check storage directory permissions
2. Check disk space
3. Verify file exists in storage (for dvs_get)
