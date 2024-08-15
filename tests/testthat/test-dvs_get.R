test_that("get doesn't work outside a git repo [UNI-GET-001]", {
  temp_dir <- fs::dir_create(tempdir())
  withr::defer(fs::dir_delete(temp_dir), parent.frame())
  withr::with_dir(temp_dir, {
    expect_error(dvs_get("*"), "git repository not found")
  })
})

test_that("get errors for a file that hasn't been added [UNI-GET-002]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("file-error", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    # create file
    print(paste0("proj_dir: ", dvs$proj_dir))
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    print(file)
    fs::file_create(file)

    expect_error(dvs_get(file), "metadata file not found for at least one file:")
  })
})

test_that("get errors for a file that doesn't exist [UNI-GET-003]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("file-dne", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    expect_error(dvs_get("dne.txt"), "metadata file not found for at least one file:")

  })
})

test_that("get errors for a bad input [UNI-GET-004]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("random", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    expect_error(dvs_get("random"), "metadata file not found for at least one file:")

  })
})

test_that("get doesn't error for a non-added file in a glob [UNI-GET-005]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("file-error", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    # create file in proj_dir
    print(paste0("proj_dir: ", dvs$proj_dir))
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    print(file)
    fs::file_create(file)

    # get all files in proj_dir (should not get file because it hasn't been added)
    get <- dvs_get(file.path(dvs$proj_dir, "*"))
    expect_equal(nrow(get), 0)
  })
})

test_that("get errors when dvs not inited [UNI-GET-006]", {
  # create git repo
  proj_dir <- create_project("status-init")
  # run get without initializing
  withr::with_dir(proj_dir, {
    # should be in git repo
    expect_true(file.exists(file.path(proj_dir, ".git")))
    # panic because not initialized
    expect_error(dvs_get("*"), "could not load configuration file")
  })
})

test_that("get can input multiple files - explicit [UNI-GET-007]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("multiple-files", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    # add file
    print(paste0("proj_dir: ", dvs$proj_dir))
    file1 <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    file2 <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    file3 <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    fs::file_create(c(file1, file2, file3))
    dvs_add(c(file1, file2))

    # get
    get <- dvs_get(c(file1, file2))

    expect_equal(nrow(get), 2)
    expect_equal(sum(get$outcome == "present"), 2)
  })
})

test_that("get can input multiple files - implicit via file glob [UNI-GET-008]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("multiple-files-glob", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    # add file
    print(paste0("proj_dir: ", dvs$proj_dir))
    file1 <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    file2 <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    file3 <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    fs::file_create(c(file1, file2, file3))
    dvs_add(c(file1, file2))

    # get with glob - shouldn't get file3 because it hasn't been previously added
    get <- dvs_get(file.path(dvs$proj_dir, "*"))

    expect_equal(nrow(get), 2)
    expect_equal(sum(get$outcome == "present"), 2)
  })
})

test_that("files can be getted explicitly (by metadata file name) [UNI-GET-009]", {
  #TODO
})

test_that("A user can retrieve files that they didn't originally version [UNI-GET-010]", {
  #TODO
})

test_that("A list of two data frame outputs can be returned, in which case, get outputs a list of two data frames:
 1) a success data frame including the success state of each versioned file and metadata for each versioned file including the 
 relative path, absolute path, file size, file hash, and 
 2) a failure data frame, including input, relative path, absolute path, error types and error messages if relevant [UNI-GET-011]", {
  #TODO
})

test_that("A single data frame output can be returned, in which case, get outputs a data frame including the success state of each 
versioned file and metadata for each versioned file including the relative path, absolute path, file size, file hash, and input, 
error type, and error message in the case of error [UNI-GET-012]", {
  #TODO
})

test_that("If get returns an error, no operations are performed on any inputs - i.e. no files should be recopied from the storage directory [UNI-GET-013]", {
  #TODO
})

test_that("get returns an error if the initialized storage directory no longer exists [UNI-GET-014]", {
  #TODO
})

test_that("A file error occurs in the data frame output if an inputted file's absolute path cannot be found. [UNI-GET-015]", {
  #TODO
})

test_that("A file error occurs in the data frame output if an inputted file's relative path cannot be found [UNI-GET-016]", {
  #TODO
})

test_that("A file error occurs in the data frame output if an inputted file's contents cannot be hashed [UNI-GET-017]", {
  #TODO
})

test_that("A file error occurs in the data frame output if an inputted file does not exist in the git repository [UNI-GET-018]", {
  #TODO
})

test_that("A file error occurs in the data frame output if an inputted file's size cannot be found [UNI-GET-019]", {
  #TODO
})

test_that("A file error occurs in the data frame output if an inputted file's corresponding metadata file cannot be loaded and parsed [UNI-GET-020]", {
  #TODO
})

test_that("A file error occurs in the data frame output if an inputted file cannot be recopied to the git repository [UNI-GET-021]", {
  #TODO
})



