test_that("a file can be getted", {
  # initialize
  dvs <- create_project_and_initialize_dvs("update", parent.frame())

  withr::with_dir(dvs$proj_dir, {
    # create file
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    fs::file_create(file)

    # add file
    added_file1 <- dvs_add(file)
    # check that data frame added it
    expect_equal(added_file1$outcome, "copied")

    # get timestamp from metadata path
    metapath <- paste0(file, ".dvs")
    metafile1 <- jsonlite::fromJSON(txt = metapath, simplifyDataFrame = TRUE)
    timestamp1 <- metafile1$time_stamp

    # get modification time of the file
    metapath_time1 <- file.info(metapath)$mtime
    # get modification time of the file itself
    filepath_time1 <- file.info(file)$mtime

    # trivially get a file that is already present
    getted_file1 <- dvs_get(file)

    # check that get no-op'ed

    # check that the file is already present
    expect_equal(getted_file1$outcome, "present")

    # get modification time of the file
    metapath_time2 <- file.info(metapath)$mtime
    # get modification time of the file itself
    filepath_time2 <- file.info(file)$mtime

    # check that the metadata file wasn't updated
    expect_equal(metapath_time1, metapath_time2)
    # check that file wasn't re-copied to project dir
    expect_equal(filepath_time1, filepath_time2)

    # check that file is still current
    status <- dvs_status(file)
    expect_equal(status$status, "current")

    # delete the file
    fs::file_delete(file)

    # check that file is absent
    status <- dvs_status(file)
    expect_equal(status$status, "absent")

    # get file
    getted_file2 <- dvs_get(file)

    # check that the file is copied
    expect_equal(added_file1$outcome, "copied")
  })
})
