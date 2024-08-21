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
  # initialize
  dvs <- create_project_and_initialize_dvs("UNI-GET-009", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    # add file
    print(paste0("proj_dir: ", dvs$proj_dir))
    file1 <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    fs::file_create(file1)
    dvs_add(file1)

    # get while present
    get <- dvs_get(paste0(file1, ".dvs"))

    expect_equal(nrow(get), 1)
    expect_equal(get$outcome[1], "present")
    expect_equal(get$absolute_path[1], file1)

    fs::file_delete(file1)
    # get while deleted
    get2 <- dvs_get(paste0(file1, ".dvs"))

    expect_equal(nrow(get), 1)
    expect_equal(get2$outcome[1], "copied")
    expect_equal(get2$absolute_path[1], file1)
  })
})

test_that("A user can retrieve files that they didn't originally version [UNI-GET-010]", {
  #HELP
  #NOTEST I need help with this, probably need sudo
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
  dvs <- create_project_and_initialize_real_repo("UNI-GET-014", parent.frame())

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

  # dvs_add
  withr::with_dir(dvs$proj_dir, {
    add <- dvs_add(pk_data_path)
  })

  # delete stor dir for error
  fs::dir_delete(dvs$stor_dir)

  withr::with_dir(dvs$proj_dir, {
    error <- glue::glue("storage directory not found: storage_dir: {dvs$stor_dir} in dvs.yaml, No such file or directory")
    print(error)
    expect_error(dvs_get(pk_data_path), error)
  })
})

test_that("A file error occurs in the data frame output if an inputted file's absolute path cannot be found. [UNI-GET-015]", {
  #NOTEST
  #HELP
})

test_that("A file error occurs in the data frame output if an inputted file's relative path cannot be found [UNI-GET-016]", {
  #NOTEST
  #HELP
})

test_that("A file error occurs in the data frame output if an inputted file's contents cannot be hashed [UNI-GET-017]", {
  #TODO
  # dvs <- create_project_and_initialize_real_repo("UNI-GET-017", parent.frame())
  #
  # # create data file for testing
  # pk_data <- data.frame()
  # pk_data_path <- file.path(dvs$proj_dir, "pk_data.csv")
  # write.csv(pk_data, pk_data_path)
  #
  # # dvs_add
  # withr::with_dir(dvs$proj_dir, {
  #   add <- dvs_add(pk_data_path)
  # })
  #
  # fs::file_delete(pk_data_path)
  #
  # Sys.chmod("~/.cache/dvs/data/derived/pk_data.csv", mode = "000")
  # withr::defer(Sys.chmod("~/.cache/dvs/data/derived/pk_data.csv", mode = "777"))
  #
  # withr::with_dir(dvs$proj_dir, {
  #   get <- dvs_get(pk_data_path)
  #   expect_equal(get$outcome, "error")
  #   expect_equal(get$error, "file hash not found")
  #   expect_equal(get$error_message, "Permission denied (os error 13)")
  # })
})

test_that("A file error occurs in the data frame output if an inputted file does not exist in the git repository [UNI-GET-018]", {
  dvs1 <- create_project_and_initialize_real_repo("repo1", parent.frame())
  dvs2 <- create_project_and_initialize_real_repo("repo2", parent.frame())

  # create data file for testing
  pk_data <- data.frame()
  pk_data_path <- file.path(dvs1$proj_dir, "pk_data.csv")
  write.csv(pk_data, pk_data_path)

  withr::with_dir(dvs1$proj_dir, {
    add <- dvs_add(pk_data_path)
  })

  # dvs_add
  withr::with_dir(dvs2$proj_dir, {
    out <- dvs_get(pk_data_path)
    expect_equal(out$outcome, "error")
    expect_equal(out$error, "file not in git repository")
    expect_equal(out$error_message, "prefix not found")
  })
})

test_that("A file error occurs in the data frame output if an inputted file's size cannot be found [UNI-GET-019]", {
  #HELP
  #NO TEST
})

test_that("A file error occurs in the data frame output if an inputted file's corresponding metadata file cannot be loaded and parsed [UNI-GET-020]", {
  dvs <- create_project_and_initialize_real_repo("UNI-GET-020", parent.frame())

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

  # dvs_add
  withr::with_dir(dvs$proj_dir, {
    add <- dvs_add(pk_data_path)
  })

  metadata_file <- paste0(pk_data_path, ".dvs")
  expect_true(file.exists(metadata_file))
  writeLines("wwwww", metadata_file)
  fs::file_delete(pk_data_path)

  withr::with_dir(dvs$proj_dir, {
    out <- dvs_get(pk_data_path)
    expect_equal(out$outcome, "error")
    expect_equal(out$error, "metadata file not loaded")
    expect_equal(out$error_message, "expected value at line 1 column 1")
  })
})

test_that("A file error occurs in the data frame output if an inputted file cannot be recopied to the git repository [UNI-GET-021]", {
  #TODO
  # delete the file in stor dir

  dvs <- create_project_and_initialize_real_repo("UNI-GET-020", parent.frame())

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

  # dvs_add
  withr::with_dir(dvs$proj_dir, {
    add <- dvs_add(pk_data_path)
  })


  fs::file_delete(pk_data_path)
  stor_dir_files <- list.files(dvs$stor_dir, full.names = TRUE)
  unlink(stor_dir_files, recursive = TRUE)

  withr::with_dir(dvs$proj_dir, {
    out <- dvs_get(pk_data_path)
    expect_equal(out$outcome, "error")
    expect_equal(out$error, "file not copied")
  })
})



