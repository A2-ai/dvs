use chrono::prelude::*;
use csv::{WriterBuilder, Writer};
use serde::{Deserialize, Serialize};
use std::fs::{OpenOptions, Permissions};
use std::os::unix::fs::chown;
use std::path::Path;
use users::{get_current_uid, get_current_username};

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Init,
    Add,
    Remove,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HashType {
    Blake3,
    NA,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub action: Action,
    pub path: String,
    pub hash: String,
    pub hash_type: HashType,
    pub uid: u32,
    pub username: String,
}

pub fn create_audit_log(
    log_path: impl AsRef<Path>,
    guid: Option<u32>,
    permissions: &Permissions,
) -> Result<(), Box<dyn std::error::Error>> {
    // Try to create with specific permissions first
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(log_path.as_ref())
    {
        Ok(file) => {
            // New file created - set permissions and ownership
            file.set_permissions(permissions.to_owned())?;
            chown(log_path.as_ref(), None, guid)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            // Write header for new file
            let mut writer = Writer::from_writer(file);
            writer.write_record(&[
                "timestamp",
                "action",
                "path",
                "hash",
                "hash_type",
                "uid",
                "username",
            ])?;
            writer.flush()?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // File exists - that's fine
        }
        Err(e) => return Err(Box::new(e)),
    }
    Ok(())
}

fn get_user_info() -> Option<(u32, String)> {
    let uid = get_current_uid();
    let username = get_current_username()?.into_string().ok()?;

    Some((uid, username))
}

pub fn write_entry(
    log_path: impl AsRef<Path>,
    file_path: &str,
    hash: &str,
    hash_type: HashType,
    action: Action,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = log_path.as_ref();
    let (uid, username) = get_user_info().ok_or("Could not get user info")?;
    let entry = LogEntry {
        timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        action: action,
        path: file_path.to_string(),
        hash: hash.to_string(),
        hash_type: hash_type,
        uid: uid,
        username: username,
    };
    // Now append the entry
    let file = OpenOptions::new().append(true).open(path)?;

    let mut writer = WriterBuilder::new().has_headers(false).from_writer(file);
    writer.serialize(entry)?;
    writer.flush()?;
    Ok(())
}
