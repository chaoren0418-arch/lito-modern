#[path="../src/modern_lito/resource_cipher.rs"] mod resource_cipher;
fn main() -> Result<(),Box<dyn std::error::Error>> {
 let root=std::path::PathBuf::from(std::env::args_os().nth(1).ok_or("root")?);
 let input=root.join("target/r473-campaign-resource-snapshot/lito-lttext");
 let mut idx=std::fs::read(input.join("LTtext.idx"))?;let pak=std::fs::read(input.join("LTtext.pak"))?;
 resource_cipher::decode_payload_in_place(&mut idx[4..]);
 let mut found=false;
 for row in idx[4..].chunks_exact(28){let end=row[4..24].iter().position(|b|*b==0).unwrap_or(20);
  if row[4..4+end].eq_ignore_ascii_case(b"LTstring-k.tbl") {assert!(!found);found=true;let off=u32::from_le_bytes(row[..4].try_into()?) as usize;let len=u32::from_le_bytes(row[24..].try_into()?) as usize;let mut bytes=pak[off..off+len].to_vec();resource_cipher::decode_payload_in_place(&mut bytes);std::fs::write(root.join("target/r478-mode-menu/source/LTstring-k.tbl"),bytes)?;println!("LTstring-k.tbl offset={off} encrypted_bytes={len}");}
 }assert!(found);Ok(())
}
