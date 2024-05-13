test_that("add fn updates a file", {
  # initialize
  dvs <- create_project_and_initialize_dvs("update", parent.frame())
  withr::with_dir(dvs$proj_dir, {
    # create file
    file <- tempfile(tmpdir = dvs$proj_dir, fileext = ".txt")
    fs::file_create(file)
    # get metadata path
    metapath <- paste0(file, ".dvs")

    # add file
    added_file1 <- dvs_add(file)
    # get timestamp from metadata path
    metafile1 <- jsonlite::fromJSON(txt = metapath, simplifyDataFrame = TRUE)
    init_add_time <- metafile1$time_stamp

    # check that file was added close to system time
    expect_true(near_system_time(init_add_time))

    # check that data frame added it
    expect_equal(added_file1$outcome, "copied")

    # check that status says the file is updated
    status <- dvs_status(file)
    expect_equal(status$status, "current")

    # update file contents
    file_content <- readLines(file)
    new_content <- c(file_content, "This is the new line to append.")
    writeLines(new_content, file)

    status <- dvs_status(file)
    expect_equal(status$status, "unsynced")

    # add file again to update it
    added_file2 <- dvs_add(file)

    # get the timestamp again
    metafile2 <- jsonlite::fromJSON(txt = metapath, simplifyDataFrame = TRUE)
    update_time <- metafile2$time_stamp

    # check that file was added close to system time
    expect_true(near_system_time(update_time))

    # check that the initial add time was before the updated time
    print(difftime(init_add_time, update_time))
    expect_true(init_add_time < update_time)

    # check that data frame added it
    expect_equal(added_file2$outcome, "copied")
    # check that hashes DON'T match
    expect_true(added_file1$blake3_checksum != added_file2$blake3_checksum)
    # check that timestamps in metadata don't match

    status <- dvs_status(file)
    expect_equal(status$status, "current")
  })
})

test_that("helper works when true", {
  time <- as.character(char_to_zulu(Sys.time()))
  expect_true(near_system_time(time))
})

test_that("helper works when false", {
  time <- as.character(char_to_zulu(Sys.time()))
  Sys.sleep(5)
  expect_false(near_system_time(time))
})


