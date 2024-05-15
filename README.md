## Use 
`dvs` (data versioning system) is a file linker that allows teams to version files under Git without directly tracking them.

This R package allows teams to collaborate without uploading large or sensitive files to Git.

## How it works
Instead of uploading data files to Git, a user can employ `dvs`, which copies the files to a shared storage directory and generates metadata files. The user can upload these metadata files to Git to make the versioned files accessible to collaborators.\
`dvs` will generate a `.gitignore` in the immediate directory of each versioned file excluding the versioned file and including its corresponding metadata file.

When collaborators pull from Git, they can employ `dvs` to parse the metadata files to locate each corresponding data file copy in the storage directory
and copy them back to the project directory.

A `dvs.yaml` file is generated upon initialization in the project directory from which `dvs` parses the storage directory.

A `.dvs` metadata file is generated for each versioned file in its given directory.\
A versioned file's metadata file contains a hash of the versioned file's contents via the blake3 algorithm. \
This hash is used to both track the most current version of the file and create the path for a versioned file's copy in the storage directory.

## Tutorial
See a detailed tutorial [here](https://github.com/A2-ai/dvs_demo/tree/main).

## Example Workflow
### To add files to dvs:
**Step 1**: Initialize with `dvs_init` to set an accessible storage directory outside the git repo.
```
dvs_init("/data/dvs/storage_directory")
```
Output data frame:\
<img width="526" alt="Screenshot 2024-05-14 at 3 25 53 PM" src="https://github.com/a2-ai-tech-training/dvs/assets/145997132/4c974fc1-9b26-43e6-b5ac-fa5a51cd99c9">


**Step 2**: Add with `dvs_add` to copy files to the storage directory. 
```
dvs_add("data.csv")
```
Output data frame:\
<img width="802" alt="Screenshot 2024-05-14 at 3 26 38 PM" src="https://github.com/a2-ai-tech-training/dvs/assets/145997132/132255a1-5382-4131-84d8-a91b3424bdf0">

**Step 3**: Push to Git.

<br />

### To get files from dvs:
**Step 1**: Pull from Git.

**Step 2**: Generate a report with `dvs_status` to see versioned files.
```
dvs_status()
```
Output data frame:\
<img width="799" alt="Screenshot 2024-05-14 at 3 29 05 PM" src="https://github.com/a2-ai-tech-training/dvs/assets/145997132/c655002b-c756-4d6f-bfc2-eb5280557579">

**Step 3**: Get files with `dvs_get` to copy files back from the storage directory.
```
dvs_get("data.csv")
```
Output data frame:\
<img width="798" alt="Screenshot 2024-05-14 at 3 29 50 PM" src="https://github.com/a2-ai-tech-training/dvs/assets/145997132/66260d57-d100-4a9e-87ea-73bd59e4e316">


