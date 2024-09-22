test_that("status works when no files have been added [UNI-STA-001]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("base-case", parent.frame())

  # run status without adding anything
  withr::with_dir(dvs$proj_dir, {
    status <- dvs_status()
    expect_equal(nrow(status), 0)
  })
})

test_that("status works with no input [UNI-STA-002]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("no-args", parent.frame())

  # run status after adding
  withr::with_dir(dvs$proj_dir, {
    print(paste0("proj_dir: ", dvs$proj_dir))
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    print(file)
    fs::file_create(file)
    dvs_add(file)
    meta_path <- paste0(file, ".dvs")
    print(meta_path)
    expect_true(file.exists(paste0(file, ".dvs")))
    status <- dvs_status()
    expect_equal(nrow(status), 1)
    expect_equal(status$status, "current")
  })
})

test_that("status works with a single file input [UNI-STA-003]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("explicit-arg", parent.frame())

  # run status after adding
  withr::with_dir(dvs$proj_dir, {
    print(paste0("proj_dir: ", dvs$proj_dir))
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    print(file)
    fs::file_create(file)
    dvs_add(file)
    meta_path <- paste0(file, ".dvs")
    print(meta_path)
    expect_true(file.exists(paste0(file, ".dvs")))
    status <- dvs_status(file)
    expect_equal(nrow(status), 1)
    expect_equal(status$status, "current")
  })
})

test_that("status works with * glob [UNI-STA-004]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("glob-1", parent.frame())

  # run status after adding
  withr::with_dir(dvs$proj_dir, {
    print(paste0("proj_dir: ", dvs$proj_dir))
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    print(file)
    fs::file_create(file)
    dvs_add(file)
    meta_path <- paste0(file, ".dvs")
    print(meta_path)
    expect_true(file.exists(paste0(file, ".dvs")))
    status <- dvs_status("*")
    expect_equal(nrow(status), 1)
    expect_equal(status$status, "current")
  })
})

test_that("status works with *.txt glob, [UNI-STA-005]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("glob-2", parent.frame())

  # run status after adding
  withr::with_dir(dvs$proj_dir, {
    print(paste0("proj_dir: ", dvs$proj_dir))
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    print(file)
    fs::file_create(file)
    dvs_add(file)
    meta_path <- paste0(file, ".dvs")
    print(meta_path)
    expect_true(file.exists(paste0(file, ".dvs")))
    status <- dvs_status("*.txt")
    expect_equal(nrow(status), 1)
    expect_equal(status$status, "current")
  })
})

test_that("status errors when dvs not inited [UNI-STA-006]", {
  # create git repo
  proj_dir <- create_project("status-init")
  # run status without initializing
  withr::with_dir(proj_dir, {
    # should be in git repo
    expect_true(file.exists(file.path(proj_dir, ".git")))
    # panic because not inited
    expect_error(dvs_status(), "could not load configuration file")
  })
})

test_that("status errors when not in a git repo [UNI-STA-007]", {
  temp_dir <- fs::dir_create(tempdir())
  withr::defer(fs::dir_delete(temp_dir), parent.frame())
  withr::with_dir(temp_dir, {
    expect_error(dvs_status(), "could not find git repo root")
  })
})

test_that("status errors for a file whose metadata file cannot be loaded [UNI-STA-008]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("file-error", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    # add file
    print(paste0("proj_dir: ", dvs$proj_dir))
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    print(file)
    fs::file_create(file)
    dvs_add(file)

    Sys.chmod(paste0(file, ".dvs"), mode = "000")
    # status
    status <- dvs_status(file)

    # one row
    expect_equal(nrow(status), 1)
    # status is error
    expect_equal(status$status, "error")
    expect_equal(status$error, "metadata file not loaded")
    # error message exists
    expect_equal(status$error_message, "Permission denied (os error 13)")
  })
})

test_that("status can input multiple files - explicit [UNI-STA-009]", {
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

    # status
    status <- dvs_status(c(file1, file2, file3))
    print(status)
    expect_equal(nrow(status), 3)
    expect_equal(sum(status$status == "current"), 2)
    expect_equal(sum(status$status == "error"), 1)
  })
})

test_that("status can input multiple files - implicit via file glob [UNI-STA-010]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("multiple-files-glob", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    # add files
    file1 <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    file2 <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    file3 <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    fs::file_create(c(file1, file2, file3))
    dvs_add(c(file1, file2))

    # status with glob
    status <- dvs_status(file.path(dvs$proj_dir, "*"))

    expect_equal(nrow(status), 2)
    expect_equal(sum(status$status == "current"), 2)
  })
})

test_that("For single data frame output, the status function should output a data frame including the versioning status
and metadata for a given set of versioned files including the relative path, absolute path, file size, file hash,
time of the most recent file version, user who uploaded the most recent file version, message from the most recent
file versioner, and input, error type, and error message if relevant [UNI-STA-011]", {
  dvs <- create_project_and_initialize_real_repo("UNI-STA-011", parent.frame())

  # create data file for testing
  pk_data <- data.frame(
    USUBJID = c(1, 1, 1),
    NTFD = c(0.5, 1, 2),
    DV = c(379.444, 560.613, 0)
  )

  data_derived_dir <- file.path(dvs$proj_dir, "data/derived")
  fs::dir_create(data_derived_dir)

  pk_data_path <- file.path(data_derived_dir, "pk_data.csv")
  write.csv(pk_data, pk_data_path)

  # dvs_get
  withr::with_dir(dvs$proj_dir, {
    added_files <- dvs_add(pk_data_path)
    statused_files <- dvs_status(pk_data_path, split_output = FALSE)

    status_df_fields <- c("relative_path", "status", "size", "blake3_checksum", "add_time", "saved_by", "message", "absolute_path", "error", "error_message", "input")

    sapply(status_df_fields, function(field) {
      expect_true(field %in% names(statused_files))
    })
  })
})

test_that("For split data frame output, the status function should output a list of two data frames:
1) a success data frame including the versioning status and metadata for a given set of versioned
files including the relative path, absolute path, file size, file hash, time of the most recent
file version, user who uploaded the most recent file version, and message from the most recent file versioner, and
2) a failure data frame, including input, relative path, absolute path, error type, and error message if relevant [UNI-STA-012]", {
  dvs <- create_project_and_initialize_real_repo("UNI-STA-012", parent.frame())

  # create data file for testing
  pk_data <- data.frame(
    USUBJID = c(1, 1, 1),
    NTFD = c(0.5, 1, 2),
    DV = c(379.444, 560.613, 0)
  )

  data_derived_dir <- file.path(dvs$proj_dir, "data/derived")
  fs::dir_create(data_derived_dir)

  pk_data_path <- file.path(data_derived_dir, "pk_data.csv")
  write.csv(pk_data, pk_data_path)

  # dvs_status
  withr::with_dir(dvs$proj_dir, {
    added_files <- dvs_add(pk_data_path)
    statused_files <- dvs_status(c(pk_data_path, "dne.R"), split_output = TRUE)

    # check success fields
    success_fields <- c("relative_path", "status", "size", "blake3_checksum", "add_time", "saved_by", "message", "absolute_path")
    sapply(success_fields, function(field) {
      expect_true(field %in% names(statused_files$successes))
    })

    # check failure fields
    failure_fields <- c("relative_path", "absolute_path", "input", "error", "error_message")
    sapply(failure_fields, function(field) {
      expect_true(field %in% names(statused_files$failures))
    })
  })

})

test_that("The status function can input files explicitly (by metadata file name) [UNI-STA-013]", {
  dvs <- create_project_and_initialize_dvs("explicit-arg", parent.frame())

  # run status after adding
  withr::with_dir(dvs$proj_dir, {
    # create file
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    fs::file_create(file)

    # add file
    dvs_add(file)
    meta_path <- paste0(file, ".dvs")

    expect_true(file.exists(paste0(file, ".dvs")))

    status <- dvs_status(meta_path)

    expect_equal(nrow(status), 1)
    expect_equal(status$status, "current")
  })
})

test_that("Sets of implicitly inputted files to the status function should exclude non-metadata files [UNI-STA-014]", {
  dvs <- create_project_and_initialize_dvs("explicit-arg", parent.frame())

  # create file
  file1 <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
  fs::file_create(file1)

  file2 <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
  fs::file_create(file2)

  # run status after adding
  withr::with_dir(dvs$proj_dir, {
    # add file
    dvs_add(file1)
    meta_path <- paste0(file1, ".dvs")

    expect_true(file.exists(paste0(file1, ".dvs")))

    status <- dvs_status("*") # shouldn't include file2

    expect_equal(nrow(status), 1)
    expect_equal(status$status, "current")
  })
})

test_that("No file operations should be performed by the status function [UNI-STA-016]", {
  # initialize
  dvs <- create_project_and_initialize_dvs("no-op", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    # create file
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    fs::file_create(file)
    # get modification time of the file
    filepath_time1 <- file.info(file)$mtime

    # add file
    added_file1 <- dvs_add(file)

    # get metadata path generated from adding
    metapath <- paste0(file, ".dvs")
    # get modification time of the metadata file
    metapath_time1 <- file.info(metapath)$mtime

    # get gitignore path generated from adding
    git_path <- file.path(dvs$proj_dir, ".gitignore")
    # get modification time of the gitignore file
    gitpath_time1 <- file.info(git_path)$mtime

    # check that data frame added file
    expect_equal(added_file1$outcome, "copied")

    # status
    status <- dvs_status(file)

    # check for no-op
    # outcome in output should be present instead of copied
    expect_equal(status$status, "current")

    # get modification time of the file
    filepath_time2 <- file.info(file)$mtime
    # check that file didn't change
    expect_equal(filepath_time1, filepath_time2)

    # get modification time of the metadata file
    metapath_time2 <- file.info(metapath)$mtime
    # check that metadata file didn't change
    expect_equal(metapath_time1, metapath_time2)

    # get modification time of the gitignore file
    gitpath_time2 <- file.info(git_path)$mtime
    # check that gitignore file didn't change
    expect_equal(gitpath_time1, gitpath_time2)
  })
})

test_that("The status function should return an error if the repository hasn't been initialized
with the initialization function [UNI-STA-017]", {
  #TODO
  proj_dir <- create_project_no_dvs_init("UNI-STA-017", parent.frame())
  expect_true(dir.exists(proj_dir))
  expect_true(dir.exists(file.path(proj_dir, ".git")))

  data_derived_dir <- file.path(tempdir(), "projects/add_single_file/data/derived")
  fs::dir_create(data_derived_dir)

  #check data directory exists
  expect_true(dir.exists(data_derived_dir))

  # create data file for testing
  pk_data <- data.frame(
    USUBJID = c(1, 1, 1),
    NTFD = c(0.5, 1, 2),
    DV = c(379.444, 560.613, 0)
  )

  pk_data_path <- file.path(data_derived_dir, "pk_data.csv")
  write.csv(pk_data, pk_data_path)

  expect_true(file.exists(pk_data_path))

  # dvs_status
  withr::with_dir(proj_dir, {
    expect_error(dvs_status(pk_data_path), "configuration file not found")
  })
})

test_that("The status function, should not return an error, but rather indicate a file error in its data
frame output(s) if an inputted file is actually a directory path [UNI-STA-018]", {
  dvs <- create_project_and_initialize_real_repo("UNI-STA-018", parent.frame())

  # create data file for testing
  pk_data <- data.frame(
    USUBJID = c(1, 1, 1),
    NTFD = c(0.5, 1, 2),
    DV = c(379.444, 560.613, 0)
  )

  data_derived_dir <- file.path(dvs$proj_dir, "data/derived")
  fs::dir_create(data_derived_dir)

  pk_data_path <- file.path(data_derived_dir, "pk_data.csv")
  write.csv(pk_data, pk_data_path)

  # dvs_status
  withr::with_dir(dvs$proj_dir, {
    out <- dvs_status(data_derived_dir)
    expect_equal(out$status, "error")
    expect_equal(out$error, "path is a directory")
    expect_true(is.na(out$error_message))
  })
})

test_that("The status function, should not return an error, but rather indicate a file error in its data
frame output(s) if an inputted file hasn't been versioned with the versioning function [UNI-STA-019]", {
  dvs <- create_project_and_initialize_real_repo("UNI-STA-019", parent.frame())

  # create data file for testing
  pk_data <- data.frame(
    USUBJID = c(1, 1, 1),
    NTFD = c(0.5, 1, 2),
    DV = c(379.444, 560.613, 0)
  )

  data_derived_dir <- file.path(dvs$proj_dir, "data/derived")
  fs::dir_create(data_derived_dir)

  pk_data_path <- file.path(data_derived_dir, "pk_data.csv")
  write.csv(pk_data, pk_data_path)

  pk_data_2 <- data.frame()
  pk_data_path_2 <- file.path(data_derived_dir, "pk_data_2.csv")
  write.csv(pk_data_2, pk_data_path_2)

  # dvs_status
  withr::with_dir(dvs$proj_dir, {
    out <- dvs_status(pk_data_path_2)
    expect_equal(out$status, "error")
    expect_equal(out$error, "file not added")
    expect_equal(out$error_message, "metadata file not found - add the file to dvs to get status")
  })
})

test_that("The status function, should not return an error, but rather indicate a file error in its data
frame output(s) if an inputted file's contents cannot be hashed [UNI-STA-020]", {
  #TODO
  dvs <- create_project_and_initialize_real_repo("UNI-STA-020", parent.frame())

  # create data file for testing
  pk_data <- data.frame(
    USUBJID = c(1, 1, 1),
    NTFD = c(0.5, 1, 2),
    DV = c(379.444, 560.613, 0)
  )

  data_derived_dir <- file.path(dvs$proj_dir, "data/derived")
  fs::dir_create(data_derived_dir)

  pk_data_path <- file.path(data_derived_dir, "pk_data.csv")
  write.csv(pk_data, pk_data_path)

  # dvs_status
  withr::with_dir(dvs$proj_dir, {
    # add
    add <- dvs_add(pk_data_path)
    # delete cache
    fs::file_delete("~/.cache/dvs/UNI-STA-020/data/derived/pk_data.csv")
    Sys.chmod(pk_data_path, mode = "000")

    out <- dvs_status(pk_data_path)
    expect_equal(out$status, "error")
    expect_equal(out$error, "file hash not found")
    expect_equal(out$error_message, "Permission denied (os error 13)")
  })
})
