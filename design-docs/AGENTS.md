# DVS Design Documentation - Agent Guide

This document explains the structure and purpose of the design documentation in this directory, intended to help AI agents and developers quickly navigate and understand the DVS codebase.

---

## Purpose

These indices were created to enable efficient lookup of:
- Which files are most important for specific tasks
- How components relate to each other
- What functionality exists and where it's implemented
- Error types and their causes
- Test coverage and behavior specifications

---

## Index Structure

```
design-docs/
├── AGENTS.md              # This file - navigation guide
├── INDEX-ARCHITECTURE.md  # High-level system design
├── INDEX-FILES.md         # File-by-file reference with importance ratings
├── INDEX-FUNCTIONS.md     # Function reference with parameters/returns
├── INDEX-ERRORS.md        # Error catalog with causes and resolutions
├── INDEX-TESTS.md         # Test case catalog organized by ID
└── PRODUCT-SPEC.md        # Reverse-engineered product specification
```

---

## Quick Reference: Which Index to Use

| Task | Primary Index | Secondary Index |
|------|---------------|-----------------|
| Understanding overall design | INDEX-ARCHITECTURE.md | INDEX-FILES.md |
| Finding where feature is implemented | INDEX-FUNCTIONS.md | INDEX-FILES.md |
| Debugging an error | INDEX-ERRORS.md | INDEX-TESTS.md |
| Adding a new feature | INDEX-FILES.md | INDEX-ARCHITECTURE.md |
| Understanding expected behavior | INDEX-TESTS.md | PRODUCT-SPEC.md |
| Reviewing user-facing API | PRODUCT-SPEC.md | INDEX-FUNCTIONS.md |

---

## INDEX-ARCHITECTURE.md

**Purpose**: Provides system-level understanding

**Contents**:
- ASCII diagrams of component relationships
- Data flow diagrams for each operation
- Storage structure documentation
- Dependency list
- Cross-reference table mapping features to files

**When to use**:
- First encounter with codebase
- Planning architectural changes
- Understanding how components connect
- Identifying which files to modify together

---

## INDEX-FILES.md

**Purpose**: Comprehensive file catalog with importance ratings

**Contents**:
- Every file in the repository
- Importance rating (CRITICAL/HIGH/MEDIUM/LOW)
- Line counts
- Key functions per file
- Purpose descriptions
- R function parameter summaries
- Rust struct definitions

**Importance Ratings Explained**:
- **CRITICAL**: Core functionality, must understand for any changes
- **HIGH**: Important for feature work and bug fixes
- **MEDIUM**: Supporting functionality, understand when relevant
- **LOW**: Configuration, tests, or generated files

**When to use**:
- Deciding which files to read
- Understanding file relationships
- Finding where functionality lives
- Prioritizing code review

---

## INDEX-FUNCTIONS.md

**Purpose**: API reference for all functions

**Contents**:
- All R functions with parameters and returns
- All Rust FFI functions
- All Rust library functions
- All helper functions
- Logic flow documentation
- Struct definitions
- Function relationships

**When to use**:
- Understanding function signatures
- Finding which functions call which
- Tracing data flow through the system
- Adding new functionality

---

## INDEX-ERRORS.md

**Purpose**: Error troubleshooting guide

**Contents**:
- Error hierarchy explanation
- Complete error type catalog
- Cause and resolution for each error
- Error-to-function mapping
- Troubleshooting guide

**When to use**:
- Debugging user-reported errors
- Understanding error handling patterns
- Adding new error types
- Writing error messages

---

## INDEX-TESTS.md

**Purpose**: Test case catalog and behavior specification

**Contents**:
- Test ID convention (UNI/MAN/INT)
- Tests organized by function
- Key assertions for each test
- Expected error conditions
- Integration test workflows
- Helper function reference
- Test data patterns

**When to use**:
- Understanding expected behavior
- Finding test coverage for functionality
- Adding new tests
- Verifying behavior changes

---

## PRODUCT-SPEC.md

**Purpose**: Product-level specification reverse-engineered from code

**Contents**:
- Product overview and purpose
- Feature specifications
- User stories and journeys
- API reference
- Configuration reference
- File format specifications
- Constraints and limitations

**When to use**:
- Understanding product from user perspective
- Writing documentation
- Planning new features
- Reviewing completeness of implementation

---

## Navigation Patterns

### "I need to fix a bug in X"

1. Check **INDEX-ERRORS.md** if it's an error-related bug
2. Look up the feature in **INDEX-FUNCTIONS.md** to find implementation
3. Check **INDEX-TESTS.md** for related test cases
4. Read files identified in **INDEX-FILES.md** by importance

### "I need to add a new feature"

1. Review **PRODUCT-SPEC.md** to understand product context
2. Check **INDEX-ARCHITECTURE.md** for where feature fits
3. Look at **INDEX-FUNCTIONS.md** for similar patterns
4. Review **INDEX-TESTS.md** for testing patterns
5. Use **INDEX-FILES.md** to identify files to modify

### "I need to understand how this works"

1. Start with **INDEX-ARCHITECTURE.md** for overview
2. Trace through **INDEX-FUNCTIONS.md** for call flow
3. Read relevant source files ordered by **INDEX-FILES.md** importance

### "I need to write tests"

1. Check **INDEX-TESTS.md** for existing patterns
2. Review **PRODUCT-SPEC.md** for expected behaviors
3. Look at **INDEX-ERRORS.md** for error cases to cover

---

## File Importance Quick Reference

### Must Read First (CRITICAL)
- `R/init.R`, `R/add.R`, `R/get.R`, `R/status.R`
- `src/rust/src/lib.rs`
- `src/rust/src/library/init.rs`, `add.rs`, `get.rs`, `status.rs`
- `src/rust/src/helpers/error.rs`

### Read for Context (HIGH)
- `DESCRIPTION`
- `src/rust/src/helpers/*.rs` (most files)
- `R/extendr-wrappers.R`
- Test files in `tests/testthat/`

### Read When Relevant (MEDIUM/LOW)
- Build configuration files
- Generated documentation
- CI/CD workflows

---

## Updating These Indices

When making significant changes to DVS:

1. Update relevant index files to reflect changes
2. Maintain the same format and structure
3. Update cross-references if file responsibilities change
4. Add new test IDs to INDEX-TESTS.md
5. Update PRODUCT-SPEC.md if user-facing behavior changes

---

## Key DVS Concepts for Agents

1. **Content-Addressable Storage**: Files stored by blake3 hash
2. **Metadata Files**: `.dvs` JSON files track version info
3. **Storage Directory**: External directory (outside git) holding file contents
4. **Git Integration**: Automatic .gitignore management
5. **Statuses**: `current` | `absent` | `unsynced` | `error`
6. **Outcomes**: `copied` | `present` | `error`
