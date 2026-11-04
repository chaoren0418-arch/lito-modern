//! Pinned492090 cssd/cssds and480560 chdch subset. Fixtures are NOT currentOS authority.
use anyhow::{ensure,Result};
use crate::modern_lito::native_opcodes::{C_LOGIN,C_VERSION};
use zeroize::Zeroizing;
#[derive(Clone,Debug,PartialEq,Eq)]pub enum FixtureVariant {Normal,Special{supplemental:Vec<u8>}}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct FixtureEnvironment {pub(crate) label:String,pub(crate) code_page:u32,pub(crate) from_cd_registry_dword:u32,pub(crate) revision_word_source:u32,pub(crate) variant:FixtureVariant}
impl FixtureEnvironment {
 pub fn synthetic(label:&str,code_page:u32,from_cd_registry_dword:u32,revision_word_source:u32,variant:FixtureVariant)->Result<Self>{
  ensure!(!label.is_empty()&&label.len()<=80&&label.is_ascii(),"explicitsyntheticprovenancetag required");
  if let FixtureVariant::Special{supplemental}= &variant{ensure!(!supplemental.contains(&0)&&supplemental.len()<=148,"fixture supplemental exceeds boundedcandidate orcontainsNUL");}
  Ok(Self{label:label.into(),code_page,from_cd_registry_dword,revision_word_source,variant})
 }
 pub fn provenance(&self)->&str{&self.label}
 pub fn compatibility_body(&self)->[u8;10]{let mut out=[0u8;10];out[0]=C_VERSION;out[1..3].copy_from_slice(&0x8033u16.to_le_bytes());out[3..7].copy_from_slice(&self.code_page.to_le_bytes());out[7]=if matches!(self.variant,FixtureVariant::Normal){self.from_cd_registry_dword as u8}else{2};out[8..10].copy_from_slice(&(self.revision_word_source as u16).to_le_bytes());out}
}
/// Exactnormalformatterorder: c1,saccount,s password,d sin_addr rawnetworkoctets.
/// Supplemental s existsONLYin64BBF8nonzero cssdsbranch.155 is Moderncandidatebound.
pub fn serialize_fixture_login(account:&[u8],password:&[u8],local_ipv4:[u8;4],environment:&FixtureEnvironment)->Result<Zeroizing<Vec<u8>>>{
 ensure!((2..=27).contains(&account.len())&&(4..=13).contains(&password.len())&&!account.contains(&0)&&!password.contains(&0),"ownedlogininputoutside sourcecapacities");
 let mut body=Zeroizing::new(Vec::new());body.push(C_LOGIN);body.extend_from_slice(account);body.push(0);body.extend_from_slice(password);body.push(0);body.extend_from_slice(&local_ipv4);
 if let FixtureVariant::Special{supplemental}= &environment.variant{body.extend_from_slice(supplemental);body.push(0);}
 ensure!(body.len()<=155,"authbodyexceedsModerncandidate155bound");Ok(body)
}
/// Source492090/490390 decodeLOW byte ofeach12byteobject;noactualNativeobject read.
pub fn decode_fixture_scalar(index_xor:u32,pool:&[u32;16],key:u32)->Result<u8>{let index=(index_xor^0xB104_FA87)as usize;ensure!(index<16,"fixture scalarindex outside64bytebacking");Ok((pool[index]as u8)^(key as u8))}
///491FE0 addsfirst16bytesmod256 andwrites'0'at+16;492010 hashesONLYfirst16.
pub fn mix_fixture_environment(serial:[u8;16],adapter:[u8;16])->[u8;17]{let mut out=[0u8;17];for i in 0..16{out[i]=serial[i].wrapping_add(adapter[i]);}out[16]=b'0';out}
///49AE10 trimsleadingASCIIspaces andNUL-terminatesat+16;fixturesonly.
pub fn normalize_fixture_serial(raw:&[u8])->[u8;16]{let mut out=[0u8;16];let at=raw.iter().position(|b|*b!=b' ').unwrap_or(raw.len());for(i,b)in raw[at..].iter().take(16).enumerate(){if *b==0{break;}out[i]=*b;}out}
///491E80 hexformat6bytes;491ED0 chooseslastmatchingtype6entry,zeroifnone.
pub fn format_fixture_adapter(entries:&[(u32,[u8;6])])->[u8;16]{let mut out=[0u8;16];let hex=b"0123456789ABCDEF";for(kind,bytes)in entries{if *kind==6{for(i,b)in bytes.iter().enumerate(){out[i*2]=hex[(b>>4)as usize];out[i*2+1]=hex[(b&15)as usize];}}}out}
