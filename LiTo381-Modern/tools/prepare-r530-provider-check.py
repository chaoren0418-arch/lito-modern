from pathlib import Path
P=Path(__file__).resolve().parent.parent
parent=(P/'src/modern_lito/native_backend_485f_entry.rs').read_text('utf-8')
defs=parent[parent.index('struct ConsumerSpec'):parent.index('fn save_new')]
prefix='''use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};use sha2::{Digest,Sha256};
use std::{cell::Cell,fs,io::Read,path::{Path,PathBuf},time::{Duration,Instant}};
use lito_modern::modern_lito::native_city_runtime::{Runtime,login_events::Backend485fProfile,java_room_ready as ready_flow};
'''
old=(P/'tools/r528-status-io-provider.rs').read_text('utf-8');start=old.index(' fn sync()->Value');end=old.index(' #[test]',start);helpers=old[start:end]
vstart=old.index('let root=Path::new(SERVER_RUNS)',end);vend=old.index(';let raw=serde_json::to_vec(&v)',vstart);base=old[vstart:vend]+';'
test='''
 #[cfg(test)]mod r530_new_retirement_boundary {
 use super::*;
'''+helpers+''' #[test]fn new_ui_boundary_rejects_retired_c9af_and_untrusted_metadata_without_io(){
 '''+base+'''
 let raw=serde_json::to_vec(&v).unwrap();assert!(static_ready(&raw,&digest(&raw)).is_ok());
 let retired=Path::new(SERVER_RUNS).join("nw-c9afae2f2cd14da08208599b1bf066fb");v["run_identity"]=json!(retired.file_name().unwrap().to_str().unwrap());v["run_root"]=json!(retired);
 let raw=serde_json::to_vec(&v).unwrap();assert!(static_ready(&raw,&digest(&raw)).err().unwrap().to_string().contains("retired"));
 assert!(static_ready(&raw,&"0".repeat(64)).err().unwrap().to_string().contains("digest"));
 }
}
'''
s=prefix+defs+'mod provider {include!("../src/modern_lito/native_backend_485f_provider.rs");'+test+'}\nmod status_reader{include!("../src/modern_lito/native_backend_485f_status_reader.rs");}\n'
with (P/'tools/r530-provider-boundary-check.rs').open('x',encoding='utf-8',newline='\n')as f:f.write(s)
print('New c9af static boundary check prepared; no retired READY read')
