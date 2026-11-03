//! Source58 version/refusal and structurally complete205/209 admission subset.
//! Dynamic834960/834A60 table authority remains absent: no actor is installed.
use anyhow::{ensure,Context,Result};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Version {value:i32}
impl Version {pub fn value(&self)->i32{self.value}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Change {VersionEstablished{version:Version},LoginRefused,Ignored209Phase{status:u8}}
#[derive(Clone,Debug,Default,PartialEq,Eq)]pub struct Snapshot {pub version:Option<Version>}
#[derive(Clone,Default)]pub(crate) struct Session {version:Option<Version>}
struct Reader<'a>{body:&'a[u8],at:usize}
impl Reader<'_>{
 fn take(&mut self,n:usize)->Result<&[u8]>{let end=self.at.checked_add(n).context("actor offset overflow")?;let b=self.body.get(self.at..end).context("short source actor/version body")?;self.at=end;Ok(b)}
 fn byte(&mut self)->Result<u8>{Ok(self.take(1)?[0])}
 fn dword(&mut self)->Result<i32>{Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))}
 fn string(&mut self,cap:usize)->Result<&[u8]>{let n=self.body[self.at..].iter().take(cap).position(|b|*b==0).context("source actor name lacks bounded NUL")?;self.take(n+1)}
 fn end(&self)->Result<()>{ensure!(self.at==self.body.len(),"trailing source actor/version bytes");Ok(())}
}
fn version(body:&[u8])->Result<Change>{
 let mut r=Reader{body,at:1};let discriminator=r.byte()?;
 if discriminator==1{r.end()?;return Ok(Change::LoginRefused);}
 //4A3423 c;ordinary4A34A1 cdddddcc;4A34BA c;>=9E38 adds d.
 r.byte()?;let value=r.dword()?;for _ in 0..4{r.dword()?;}r.take(2)?;r.byte()?;
 if value>=0x9e38{r.dword()?;}r.end()?;Ok(Change::VersionEstablished{version:Version{value}})
}
fn state209(body:&[u8])->Result<u8>{let mut r=Reader{body,at:1};r.string(28)?;let status=r.byte()?;r.end()?;Ok(status)}
/// Structural validation independent of a Runtime revision.205 exact layout
/// is deliberately deferred to its privately owned source58 authority.
pub(crate) fn validate(body:&[u8])->Result<()>{
 ensure!(body.len()<=155,"source actor/version body exceeds bound");
 match body.first(){Some(58)=>{version(body)?;},Some(209)=>{state209(body)?;},Some(205)=>ensure!(body.len()>=5,"short source205 identifier"),_=>anyhow::bail!("not a source actor/version body")};Ok(())
}
fn actor205(body:&[u8],v:Version)->Result<()>{
 let fmt:&[u8]=if v.value>=0x9d7c{b"dscccdddcscsdccdc"}else if v.value>=0x7726{b"dscccdddcscsdccd"}else if v.value>=0x75a4{b"dscccdddcscsdccc"}else if v.value>=0x52e0{b"dscccdddcscsdcc"}else{b"dscccdddcscsdc"};
 let mut r=Reader{body,at:1};let mut strings=0;
 for c in fmt{match c{b'd'=>{r.take(4)?;},b'c'=>{r.byte()?;},b's'=>{r.string(if strings==0{28}else{17})?;strings+=1;},_=>unreachable!()}}
 if v.value>=0x9d7c{r.take(12)?;if v.value>=0x9f0a{r.take(5)?;r.dword()?;}}r.end()
}
impl Session {
 pub(crate) fn snapshot(&self)->Snapshot{Snapshot{version:self.version}}
 pub(crate) fn prepare(&self,body:&[u8],selected:bool,phase:u32)->Result<(Self,Change)>{
  validate(body)?;let mut next=self.clone();
  let change=match body[0]{
   58=>{let c=version(body)?;if let Change::VersionEstablished{version}=c{next.version=Some(version);}c},
   205=>{let v=self.version.context("source205 missing current-epoch source58 version")?;actor205(body,v)?;ensure!(selected,"source205 has no selected roster identity");anyhow::bail!("source205 dynamic comparison table authority unqualified;actor not admitted");},
   209=>{let status=state209(body)?;ensure!(!matches!(phase,2|3|6),"source209 active-phase dynamic comparison authority unqualified");Change::Ignored209Phase{status}},
   _=>unreachable!()
  };Ok((next,change))
 }
}
