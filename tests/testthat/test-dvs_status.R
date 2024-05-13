test_that("status works when no files have been added", {
  # initialize
  dvs <- create_project_and_initialize_dvs("base-case", parent.frame())

  # run status without adding anything
  withr::with_dir(dvs$proj_dir, {
    status <- dvs_status()
    expect_equal(nrow(status), 0)
  })
})

test_that("status works with no input", {
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

test_that("status works with a single file input", {
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

test_that("status works with * glob", {
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

test_that("status works with *.txt glob", {
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

test_that("status errors when dvs not inited", {
  expect_error(dvs_status(), "user function panicked")
})

test_that("status errors when not in a git repo", {
  proj_name <- "no-git-repo"
  proj_dir <- fs::dir_create(file.path(tempdir(), proj_name))
  withr::defer(fs::dir_delete(proj_dir), envir = parent.frame())
  withr::with_dir(proj_dir, {
    expect_error(dvs_status(), "user function panicked")
  })
})

test_that("status errors for a file that doesn't exist", {
  # initialize
  dvs <- create_project_and_initialize_dvs("file-error", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    # add file
    print(paste0("proj_dir: ", dvs$proj_dir))
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    print(file)
    fs::file_create(file)
    dvs_add(file)

    # status
    status <- dvs_status("dne.txt")

    # one row
    expect_equal(nrow(status), 1)
    # status is error
    expect_equal(status$status, "error")
    # error is "metadata file not found"
    expect_equal(status$error, "metadata file not found")
    # error message exists
    expect_equal(sum(!is.na(status$error_message)), 1)
  })
})




