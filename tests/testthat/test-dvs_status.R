test_that("dvs status base case works", {
  # initialize
  dvs <- create_project_and_initialize_dvs("base-case", parent.frame())

  # run status without adding anything
  withr::with_dir(dvs$proj_dir, {
    status <- dvs_status()
    expect_equal(nrow(status), 0)
  })
})

test_that("dvs status no args works", {
  # initialize
  dvs <- create_project_and_initialize_dvs("current", parent.frame())

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

test_that("dvs status input arg works", {
  # initialize
  dvs <- create_project_and_initialize_dvs("current", parent.frame())

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

test_that("dvs status input arg works", {
  # initialize
  dvs <- create_project_and_initialize_dvs("current", parent.frame())

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

test_that("dvs status file glob 1", {
  # initialize
  dvs <- create_project_and_initialize_dvs("current", parent.frame())

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

test_that("dvs status file glob 2", {
  # initialize
  dvs <- create_project_and_initialize_dvs("current", parent.frame())

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

