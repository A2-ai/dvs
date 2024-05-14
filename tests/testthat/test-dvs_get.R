test_that("get doesn't work outside a git repo", {
  temp_dir <- fs::dir_create(tempdir())
  withr::defer(fs::dir_delete(temp_dir), parent.frame())
  withr::with_dir(temp_dir, {
    expect_error(dvs_get("file"), "user function panicked")
  })
})

test_that("get errors for a file that hasn't been added", {
  # initialize
  dvs <- create_project_and_initialize_dvs("file-error", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    # create file
    print(paste0("proj_dir: ", dvs$proj_dir))
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    print(file)
    fs::file_create(file)

    expect_error(dvs_get(file), "user function panicked")
  })
})

test_that("get errors for a file that doesn't exist", {
  # initialize
  dvs <- create_project_and_initialize_dvs("file-dne", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    expect_error(dvs_get("dne.txt"), "user function panicked")

  })
})

test_that("get errors for a bad input", {
  # initialize
  dvs <- create_project_and_initialize_dvs("random", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    expect_error(dvs_get("random"), "user function panicked")

  })
})

test_that("get doesn't error for a non-added file in a glob", {
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

test_that("get errors when dvs not inited", {
  # create git repo
  proj_dir <- create_project("status-init")
  # run get without initializing
  withr::with_dir(proj_dir, {
    # should be in git repo
    expect_true(file.exists(file.path(proj_dir, ".git")))
    # panic because not initialized
    expect_error(dvs_get("file"), "user function panicked")
  })
})

test_that("get can input multiple files - explicit", {
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

test_that("get can input multiple files - implicit via file glob", {
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
