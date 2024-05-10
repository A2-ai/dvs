#
#
#
# old_project <- getwd()
# # create project dir one level up
# proj_dir_parent <- tempdir()
# proj_dir <- file.path(proj_dir_parent, "proj_dir")
# dir.create(proj_dir)
# print(paste("proj_dir", proj_dir))
# on.exit(fs::dir_delete(proj_dir))
# #withr::defer(fs::dir_delete(proj_dir))
#
# # make git dir (fake being in git repo)
# git_dir <- file.path(proj_dir, ".git")
# print(paste("git_dir", git_dir))
# dir.create(git_dir)
# #withr::defer(fs::dir_delete(git_dir), parent.frame())
#
# # set up dir for storage_dir
# storage_dir_parent <- tempdir()
# storage_dir <- file.path(storage_dir_parent, "storage_dir")
# print(paste("storage_dir", storage_dir))
#
# # change wd to temp proj dir
# setwd(proj_dir)
# withr::defer(setwd(old_project), parent.frame())
# withr::defer(print("i'm getting cleaned up"), parent.frame())
# print(paste("current wd", getwd()))

