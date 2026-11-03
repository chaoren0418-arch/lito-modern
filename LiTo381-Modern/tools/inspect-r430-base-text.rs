#[path="../src/modern_lito/resource_cipher.rs"] mod resource_cipher;
use std::{fs,path::PathBuf};
fn main(){
 let root=PathBuf::from(std::env::args().nth(1).unwrap());
 let mut idx=fs::read(root.join("Text.idx")).unwrap();
 let count=u32::from_le_bytes(idx[..4].try_into().unwrap()) as usize;
 assert_eq!(idx.len(),4+count*28);assert!(count<=100_000);
 resource_cipher::decode_payload_in_place(&mut idx[4..]);
 let pak=fs::read(root.join("Text.pak")).unwrap();let mut found=0;
 for row in idx[4..].chunks_exact(28){
  let end=row[4..24].iter().position(|b|*b==0).unwrap_or(20);
  let name=std::str::from_utf8(&row[4..4+end]).unwrap();
  if name.eq_ignore_ascii_case("list.spr"){
   let offset=u32::from_le_bytes(row[..4].try_into().unwrap()) as usize;
   let size=u32::from_le_bytes(row[24..].try_into().unwrap()) as usize;
   assert!(size<=8*1024*1024 && offset+size<=pak.len());
   let mut b=pak[offset..offset+size].to_vec();resource_cipher::decode_payload_in_place(&mut b);
   assert!(!root.join("decoded-list.spr").exists());fs::write(root.join("decoded-list.spr"),b).unwrap();
   println!("base list.spr count={count} offset={offset} size={size}");found+=1;
  }
 }assert_eq!(found,1);
}
