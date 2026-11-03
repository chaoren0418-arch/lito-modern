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

#[cfg(test)]mod r528_new_status_io {
    use super::*;
    use std::io::Write;
    fn root(name:&str)->std::path::PathBuf{
        let dir=std::path::PathBuf::from(std::env::var("R528_READER_CHECK_ROOT").unwrap());
        assert_eq!(dir.parent(),Some(Path::new("D:/lito-dev-runtime/20261003-supervision/modern")));
        assert!(dir.file_name().unwrap().to_str().unwrap().starts_with("r528-status-io-"));
        let sub=dir.join(name);fs::create_dir(&sub).unwrap();sub
    }
    fn write_at(path:&Path,body:&[u8],modified:SystemTime){
        let mut file=fs::File::create(path).unwrap();file.write_all(body).unwrap();drop(file);
        fs::OpenOptions::new().write(true).open(path).unwrap().set_modified(modified).unwrap();
    }
    #[test]fn default_open_allows_replace_while_held_and_reads_own_generation(){
        let dir=root("held-replacement");let path=dir.join("status.json");let tmp=dir.join("status.json.tmp");
        write_at(&path,br#"{"generation":1}"#,SystemTime::now()-Duration::from_secs(1));
        let held=fs::File::open(&path).unwrap();let old_mtime=held.metadata().unwrap().modified().unwrap();
        write_at(&tmp,br#"{"generation":22}"#,SystemTime::now());
        fs::rename(&tmp,&path).unwrap(); // succeeds with old reader handle still open
        let old=read_opened(held,65536,Duration::from_secs(5)).unwrap();
        let new=read_json(&path,65536,Duration::from_secs(5)).unwrap();
        assert_eq!(old.json["generation"],1);assert_eq!(old.modified,old_mtime);
        assert_eq!(new.json["generation"],22);assert_ne!(old.length,new.length);
        println!("held default File::open: replacement PASS, old/new JSON and handle metadata remain distinct");
    }
    #[test]fn path_mtime_and_later_open_can_mix_versions_but_handle_reader_refuses_stale(){
        let dir=root("mtime-generation");let path=dir.join("status.json");let tmp=dir.join("status.json.tmp");
        write_at(&path,br#"{"generation":"fresh-A"}"#,SystemTime::now());
        let path_mtime=fs::metadata(&path).unwrap().modified().unwrap();
        assert!(path_mtime.elapsed().unwrap()<=Duration::from_secs(5));
        write_at(&tmp,br#"{"generation":"stale-B"}"#,SystemTime::UNIX_EPOCH);
        fs::rename(&tmp,&path).unwrap();
        let later_bytes=fs::read(&path).unwrap();assert_eq!(serde_json::from_slice::<Value>(&later_bytes).unwrap()["generation"],"stale-B");
        let err=read_json(&path,65536,Duration::from_secs(5)).err().unwrap();assert!(err.to_string().contains("stale"));
        println!("path mtime A + JSON B mismatch reproduced on own files; same-handle freshness correctly refuses stale B");
    }
    #[test]fn incomplete_json_and_bound_violation_stop_without_retry(){
        let dir=root("first-failure");let partial=dir.join("partial.json");let oversized=dir.join("oversized.json");
        write_at(&partial,br#"{"generation":1"#,SystemTime::now());
        write_at(&oversized,br#"{"generation":22}"#,SystemTime::now());
        assert!(read_json(&partial,65536,Duration::from_secs(5)).is_err());
        assert!(read_json(&oversized,4,Duration::from_secs(5)).is_err());
        println!("incomplete JSON/bound failures preserved; no retry/no provider input");
    }
}
