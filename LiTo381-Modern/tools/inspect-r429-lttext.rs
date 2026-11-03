//! Read only own copied LTtext, use the pure L1 cipher, no game executable.
#[path="../src/modern_lito/resource_cipher.rs"] mod resource_cipher;
use std::{fs,io::Write,path::PathBuf};
fn main(){
 let root=PathBuf::from(std::env::args().nth(1).expect("own snapshot root"));
 let mut index=fs::read(root.join("LTtext.idx")).unwrap();
 assert_eq!(index.len(),9888);let count=u32::from_le_bytes(index[..4].try_into().unwrap()) as usize;
 assert_eq!(count,353);resource_cipher::decode_payload_in_place(&mut index[4..]);
 let pak=fs::read(root.join("LTtext.pak")).unwrap();assert_eq!(pak.len(),292553);
 let mut found=0;
 for row in index[4..].chunks_exact(28){
  let end=row[4..24].iter().position(|b|*b==0).unwrap_or(20);
  let name=std::str::from_utf8(&row[4..4+end]).unwrap();
  let offset=u32::from_le_bytes(row[..4].try_into().unwrap()) as usize;
  let size=u32::from_le_bytes(row[24..].try_into().unwrap()) as usize;
  assert!(offset+size<=pak.len());
  if name.eq_ignore_ascii_case("LTlist.spr"){
   found+=1;let mut b=pak[offset..offset+size].to_vec();resource_cipher::decode_payload_in_place(&mut b);
   assert_eq!(b,fs::read(root.join("LTlist-reference.spr")).unwrap());
   let mut out=fs::OpenOptions::new().create_new(true).write(true).open(root.join("decoded-LTlist.spr")).unwrap();out.write_all(&b).unwrap();
   println!("LTlist.spr index_count={count} offset={offset} bytes={size} decoded_equals_reference=true");
  }
 }
 assert_eq!(found,1);
}
