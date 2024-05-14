test_that("add no-ops", {
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

    # add again
    added_file2 <- dvs_add(file)

    # check for no-op
    # outcome in output should be present instead of copied
    expect_equal(added_file2$outcome, "present")

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
