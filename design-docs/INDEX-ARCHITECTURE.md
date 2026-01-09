# DVS Architecture Index

This document provides an architectural overview of the DVS (Data Versioning System) codebase, mapping the relationships between components and their responsibilities.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        R Package Layer                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │
│  │ dvs_init │ │ dvs_add  │ │ dvs_get  │ │   dvs_status     │   │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────────┬─────────┘   │
│       │            │            │                 │             │
│       └────────────┴────────────┴─────────────────┘             │
│                           │                                     │
│                    extendr-wrappers.R                           │
│                    (FFI bindings)                               │
└───────────────────────────┬─────────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────────────┐
│                      Rust Core Layer                            │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    lib.rs (FFI Entry)                   │   │
│  │   - R data frame conversion (RFile, RStatusFile, etc.)  │   │
│  │   - Error handling/propagation to R                     │   │
│  │   - split_output handling                               │   │
│  └─────────────────────────────────────────────────────────┘   │
│                            │                                    │
│  ┌─────────────────────────┴────────────────────────────────┐  │
│  │                   library/ (Core Logic)                  │  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────────┐│  │
│  │  │ init.rs │ │ add.rs  │ │ get.rs  │ │    status.rs    ││  │
│  │  │         │ │         │ │         │ │                 ││  │
│  │  │ Config  │ │ Hash,   │ │ Restore │ │ Compare hashes  ││  │
│  │  │ setup   │ │ Copy,   │ │ files   │ │ Report status   ││  │
│  │  │         │ │ Metadata│ │         │ │                 ││  │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────────────┘│  │
│  └──────────────────────────────────────────────────────────┘  │
│                            │                                    │
│  ┌─────────────────────────┴────────────────────────────────┐  │
│  │                 helpers/ (Utilities)                     │  │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ │  │
│  │  │hash.rs │ │copy.rs │ │file.rs │ │config.rs│ │parse.rs│ │  │
│  │  │blake3  │ │FS ops  │ │metadata│ │ YAML   │ │ globs  │ │  │
│  │  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘ │  │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌──────────────────┐  │  │
│  │  │repo.rs │ │ignore.rs│ │cache.rs│ │  error.rs        │  │  │
│  │  │git ops │ │gitignore│ │XDG hash│ │  outcome.rs      │  │  │
│  │  └────────┘ └────────┘ └────────┘ └──────────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow

### Adding Files (dvs_add)
```
User Input (files/globs)
    │
    ▼
R: dvs_add() ─────────────────────────────▶ parse_files_from_globs_add_impl()
    │                                                    │
    ▼                                                    ▼
Rust: lib.rs::dvs_add_impl()              Expand globs, filter .gitignore/.dvs
    │
    ▼
library/add.rs::add()
    │
    ├──▶ config::read() - Load dvs.yaml
    ├──▶ For each file:
    │       ├──▶ hash::get_file_hash() - Blake3 hash (with caching)
    │       ├──▶ file::save() - Create .dvs metadata file
    │       ├──▶ ignore::add_gitignore_entry() - Update .gitignore
    │       └──▶ copy::copy_file_to_storage_directory()
    │
    ▼
Return Vec<Result<AddedFile, FileError>>
    │
    ▼
R: Convert to data.frame (single or split)
```

### Retrieving Files (dvs_get)
```
User Input (files/globs)
    │
    ▼
R: dvs_get() ─────────────────────────────▶ parse_files_from_globs_get_impl()
    │                                                    │
    ▼                                                    ▼
Rust: lib.rs::dvs_get_impl()              Expand globs, check metadata exists
    │
    ▼
library/get.rs::get()
    │
    ├──▶ config::read() - Load dvs.yaml
    ├──▶ file::check_meta_files_exist()
    ├──▶ For each file:
    │       ├──▶ file::load() - Read .dvs metadata
    │       ├──▶ hash::get_storage_path() - Compute storage location
    │       ├──▶ Compare local hash with metadata hash
    │       └──▶ copy::copy() - Restore from storage if needed
    │
    ▼
Return Vec<Result<RetrievedFile, FileError>>
```

## Storage Structure

```
project-root/
├── dvs.yaml                    # Configuration file
├── data/
│   ├── derived/
│   │   ├── pk_data.csv         # Original data file (gitignored)
│   │   ├── pk_data.csv.dvs     # Metadata file (git tracked)
│   │   └── .gitignore          # Auto-generated exclusions
│   └── ...
└── ...

storage-directory/              # External storage (e.g., /data/dvs/project)
├── ab/                         # First 2 chars of hash
│   └── cdef1234...             # Rest of hash (file contents)
├── cd/
│   └── ef5678...
└── ...

~/.cache/dvs/                   # XDG cache directory
└── project-name/
    └── path/to/file            # Cached hash + modification time
```

## Key Dependencies

### Rust Dependencies (Cargo.toml)
| Dependency | Version | Purpose |
|------------|---------|---------|
| extendr-api | 0.7.1 | R-Rust FFI bindings |
| blake3 | 1.5.1 | Fast cryptographic hashing |
| serde | 1.0 | JSON/YAML serialization |
| serde_yaml | 0.9 | YAML config parsing |
| serde_json | 1.0.79 | JSON metadata parsing |
| memmap2 | 0.9.4 | Memory-mapped file I/O |
| rayon | 1.7.0 | Parallel hash computation |
| file-owner | 0.1.2 | Linux file ownership |
| walkdir | 2.4.0 | Directory traversal |
| glob | 0.3.1 | Glob pattern matching |
| xdg | 2.5.2 | XDG base directories |
| chrono | 0.4.37 | Timestamp handling |

### R Dependencies (DESCRIPTION)
| Dependency | Purpose |
|------------|---------|
| rlang | Error handling |
| testthat | Testing framework |
| fs | File system operations (tests) |
| jsonlite | JSON parsing (tests) |
| withr | Test fixtures |
| yaml | YAML parsing (tests) |

## Cross-Reference: Files to Functionality

| Feature | R File | Rust Library | Rust Helpers |
|---------|--------|--------------|--------------|
| Initialize | R/init.R | library/init.rs | config.rs, repo.rs |
| Add Files | R/add.R | library/add.rs | hash.rs, copy.rs, file.rs, ignore.rs |
| Get Files | R/get.R | library/get.rs | hash.rs, copy.rs, file.rs |
| Check Status | R/status.R | library/status.rs | hash.rs, file.rs |
| File Info | - | library/info.rs | file.rs |
| Glob Parsing | - | - | parse.rs |
| Hash Caching | - | - | cache.rs |
| Error Types | - | - | error.rs, outcome.rs |
