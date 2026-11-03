//! Pure bounded LTtext extraction from the existing Modern-owned snapshot.
#[path="../src/modern_lito/resource_cipher.rs"] mod resource_cipher;
use std::{fs,path::PathBuf,io::Write};
fn main(){
 let root=PathBuf::from(std::env::args().nth(1).expect("snapshot root"));
 let out=PathBuf::from(std::env::args().nth(2).expect("new output directory"));
 let mut idx=fs::read(root.join("LTtext.idx")).unwrap();let pak=fs::read(root.join("LTtext.pak")).unwrap();
 assert!(idx.len()>=4&&idx.len()<=8*1024*1024&&pak.len()<=64*1024*1024);
 let count=u32::from_le_bytes(idx[..4].try_into().unwrap()) as usize;assert_eq!(idx.len(),4+count*28);
 resource_cipher::decode_payload_in_place(&mut idx[4..]);let mut found=0;
 for(i,row)in idx[4..].chunks_exact(28).enumerate(){let name=&row[4..24];let end=name.iter().position(|b|*b==0).unwrap_or(20);if &name[..end]!=b"LTmapstr-k.tbl"{continue;}
  assert_eq!(found,0);let offset=u32::from_le_bytes(row[..4].try_into().unwrap()) as usize;let len=u32::from_le_bytes(row[24..].try_into().unwrap()) as usize;assert!(len>0&&len<8192&&offset+len<=pak.len());
  let mut bytes=pak[offset..offset+len].to_vec();fs::OpenOptions::new().write(true).create_new(true).open(out.join("LTmapstr-k.tbl.raw")).unwrap().write_all(&bytes).unwrap();resource_cipher::decode_payload_in_place(&mut bytes);
  fs::OpenOptions::new().write(true).create_new(true).open(out.join("LTmapstr-k.tbl.decoded")).unwrap().write_all(&bytes).unwrap();println!("record={i} offset={offset} bytes={len} complete_index={count}");found+=1;
 }assert_eq!(found,1);
}
