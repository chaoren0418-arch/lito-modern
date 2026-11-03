//! Closed ASCII input/lifetime subset of631E38/492270, with typed synthetic login wire.
//! Native12-byte obfuscated scalar objects/locale/provider are not emulated.
use anyhow::{ensure,Context,Result};
use std::sync::Arc;
use zeroize::{Zeroize,Zeroizing};
use super::{login_wire,source_send};
pub const ACCOUNT_MAX:usize=27;pub const PASSWORD_MIN:usize=4;pub const PASSWORD_MAX:usize=13;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Stage {Hidden,Editing,AwaitingAuthorities,Sending,AwaitingReply}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Snapshot {pub stage:Stage,pub revision:u64,pub account_len:usize,pub password_len:usize,pub staged_account_len:usize,pub staged_password_len:usize,pub write_serial:Option<u64>}
#[derive(Clone)]pub struct Control {owner:Arc<()>,revision:u64}
/// Source401680(EDX27) common ASCII branch, then4D3860 uppercasesFIRST byte.
/// Multibyte locale branches require64B6DC/tableauthority;no guessedencoding.
pub fn normalized_ascii_account(input:&[u8])->Result<Vec<u8>>{
 ensure!((2..=ACCOUNT_MAX).contains(&input.len()),"帳號長度須為 2–27 bytes");
 ensure!(input.iter().all(|b|b.is_ascii_alphanumeric()||*b==b' '),"帳號字元尚未支援；目前可用英數字與空格");
 let mut out=input.to_vec();out[0]=out[0].to_ascii_uppercase();Ok(out)
}
struct Staged {account:Zeroizing<[u8;28]>,password:Zeroizing<[u8;14]>,account_len:usize,password_len:usize}
pub(crate) struct Session {owner:Arc<()>,revision:u64,stage:Stage,account:Zeroizing<[u8;28]>,password:Zeroizing<[u8;14]>,account_len:usize,password_len:usize,staged:Option<Staged>,write_serial:Option<u64>}
impl Default for Session {fn default()->Self{Self{owner:Arc::new(()),revision:0,stage:Stage::Hidden,account:Zeroizing::new([0;28]),password:Zeroizing::new([0;14]),account_len:0,password_len:0,staged:None,write_serial:None}}}
impl Session {
 fn next(&self)->Result<u64>{self.revision.checked_add(1).context("login form revision exhausted")}
 fn clear(&mut self){self.account.zeroize();self.password.zeroize();self.account_len=0;self.password_len=0;self.staged.take();self.write_serial=None;}
 pub(crate) fn open(&mut self)->Result<()>{let next=self.next()?;self.clear();self.revision=next;self.stage=Stage::Editing;Ok(())}
 pub(crate) fn retire(&mut self)->Result<()>{let next=self.next()?;self.clear();self.revision=next;self.stage=Stage::Hidden;Ok(())}
 pub(crate) fn snapshot(&self)->Snapshot{let(a,p)=self.staged.as_ref().map(|s|{let _=(&s.account,&s.password);(s.account_len,s.password_len)}).unwrap_or((0,0));Snapshot{stage:self.stage,revision:self.revision,account_len:self.account_len,password_len:self.password_len,staged_account_len:a,staged_password_len:p,write_serial:self.write_serial}}
 pub(crate) fn control(&self)->Option<Control>{(self.stage!=Stage::Hidden).then(||Control{owner:self.owner.clone(),revision:self.revision})}
 pub(crate) fn admit(&self,c:&Control)->Result<()>{ensure!(Arc::ptr_eq(&c.owner,&self.owner)&&c.revision==self.revision&&self.stage!=Stage::Hidden,"retired or foreign login control");Ok(())}
 pub(crate) fn edit(&mut self,c:&Control,account:&[u8],password:&[u8])->Result<()>{
  self.admit(c)?;ensure!(self.stage==Stage::Editing,"login input already staged");
  ensure!(account.len()<=ACCOUNT_MAX&&password.len()<=PASSWORD_MAX,"登入欄位超過來源容量");
  ensure!(account.iter().chain(password).all(|b|(32..=126).contains(b)),"登入欄位尚未支援此字元編碼");
  if self.account[..self.account_len]==*account&&self.password[..self.password_len]==*password{return Ok(());}
  let next=self.next()?;self.account.zeroize();self.password.zeroize();self.account[..account.len()].copy_from_slice(account);self.password[..password.len()].copy_from_slice(password);self.account_len=account.len();self.password_len=password.len();self.revision=next;Ok(())
 }
 pub(crate) fn prepare_fixture_body(&self,c:&Control,local:[u8;4],environment:&login_wire::FixtureEnvironment)->Result<Zeroizing<Vec<u8>>>{self.admit(c)?;ensure!(self.stage==Stage::AwaitingAuthorities,"logininputnotstagedorwritealreadyreserved");let s=self.staged.as_ref().context("stagedlogininputabsent")?;login_wire::serialize_fixture_login(&s.account[..s.account_len],&s.password[..s.password_len],local,environment)}
 pub(crate) fn fixture_credentials(&self,c:&Control)->Result<super::login_bootstrap::Credentials>{self.admit(c)?;ensure!(self.stage==Stage::AwaitingAuthorities,"staged fixture credentials required");let s=self.staged.as_ref().context("staged fixture input absent")?;Ok(super::login_bootstrap::Credentials{account:Zeroizing::new(s.account[..s.account_len].to_vec()),password:Zeroizing::new(s.password[..s.password_len].to_vec())})}
 pub(crate) fn sending(&mut self,c:&Control,serial:u64)->Result<()>{self.admit(c)?;ensure!(self.stage==Stage::AwaitingAuthorities,"logininputalreadyreserved");let next=self.next()?;self.revision=next;self.stage=Stage::Sending;self.write_serial=Some(serial);Ok(())}
 pub(crate) fn write_report(&mut self,serial:u64,p:&source_send::Progress)->Result<()>{if self.write_serial==Some(serial)&&matches!(p,source_send::Progress::Complete){let next=self.next()?;self.clear();self.revision=next;self.stage=Stage::AwaitingReply;}Ok(())}
 pub(crate) fn stage(&mut self,c:&Control)->Result<()>{
  self.admit(c)?;ensure!(self.stage==Stage::Editing,"login input already staged");
  let canonical=normalized_ascii_account(&self.account[..self.account_len])?;ensure!((PASSWORD_MIN..=PASSWORD_MAX).contains(&self.password_len),"密碼長度須為 4–13 bytes");let next=self.next()?;
  let mut account=Zeroizing::new([0;28]);account[..canonical.len()].copy_from_slice(&canonical);let mut password=Zeroizing::new([0;14]);password[..self.password_len].copy_from_slice(&self.password[..self.password_len]);
  self.staged=Some(Staged{account,password,account_len:canonical.len(),password_len:self.password_len});self.password.zeroize();self.password_len=0;self.revision=next;self.stage=Stage::AwaitingAuthorities;Ok(())
 }
}
