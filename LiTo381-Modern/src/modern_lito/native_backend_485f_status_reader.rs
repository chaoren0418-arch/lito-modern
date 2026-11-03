// Status-only proposal. Default std File::open includes FILE_SHARE_DELETE on
// Windows. No share-mode repair, path lock, retry or provider operation needed.
use anyhow::{ensure,Result};
use serde_json::Value;
use std::{fs,io::Read,path::Path,time::{Duration,SystemTime}};
pub struct Snapshot {pub json:Value,pub modified:SystemTime,pub length:u64}
fn read_opened(mut file:fs::File,bound:u64,max_age:Duration)->Result<Snapshot>{
    ensure!(bound<usize::MAX as u64 && bound<u64::MAX,"status bound overflow");
    let before=file.metadata()?;let modified=before.modified()?;
    ensure!(before.is_file() && before.len()<=bound,"status file exceeds bound or is not regular");
    let mut bytes=Vec::new();(&mut file).take(bound+1).read_to_end(&mut bytes)?;
    let after=file.metadata()?;
    ensure!(bytes.len()as u64<=bound && bytes.len()as u64==before.len() && after.len()==before.len() && after.modified()?==modified,"status opened generation changed during read");
    ensure!(modified.elapsed()?<=max_age,"status opened generation is stale");
    drop(file);
    // Parser requires one complete JSON document. No raw-body diagnostic/log.
    let json=serde_json::from_slice(&bytes)?;
    Ok(Snapshot{json,modified,length:before.len()})
}
pub fn read_json(path:&Path,bound:u64,max_age:Duration)->Result<Snapshot>{
    ensure!(path.is_absolute(),"absolute status path required");
    read_opened(fs::File::open(path)?,bound,max_age)
}

