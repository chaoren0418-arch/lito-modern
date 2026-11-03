from r491_common import *
def edit(name,pairs):
 p=P/'src/modern_lito'/name;s=p.read_text(encoding='utf-8')
 for a,b in pairs:assert a in s,(name,a[:70]);s=s.replace(a,b)
 p.write_text(s,encoding='utf-8')
edit('native_city_runtime.rs',[
 ('#[path="native_login_form.rs"]pub mod login_form;','#[path="native_login_form.rs"]pub mod login_form;\n#[path="native_login_wire.rs"]pub mod login_wire;\n#[path="native_login_transport.rs"]pub mod login_transport;'),
 ('pub struct Runtime {login_form:','pub struct Runtime {login_transport:Option<login_transport::Transport>,login_form:'),
 ('Ok(Self{login_form:','Ok(Self{login_transport:None,login_form:'),
 ('self.login_form.retire()?;self.awaiting_source_roster=false;','self.login_transport=None;self.login_form.retire()?;self.awaiting_source_roster=false;'),
 ('self.invalidate();self.login_form.retire()?;if self.source_scope.bound()','self.invalidate();self.login_transport=None;self.login_form.retire()?;if self.source_scope.bound()'),
 ('if matches!(progress,source_send::Progress::Retired{..}){self.disconnect_source_connection_candidate(epoch)?;}Ok(progress)','if let Some(t)=&mut self.login_transport{t.write_report(ticket.serial(),&progress);}\n  if matches!(progress,source_send::Progress::Retired{..}){self.disconnect_source_connection_candidate(epoch)?;}else{self.login_form.write_report(ticket.serial(),&progress)?;}Ok(progress)')])
edit('native_source_send.rs',[(' fn prepare_body(&self,',' ///Crateprivatefixture producers serializeknownsourceformats;notpublicrawbodyAPI.\n pub(crate) fn prepare_fixture_body(&self,body:Vec<u8>)->Result<Prepared>{ensure!((1..=155).contains(&body.len()),"fixturebodyoutsidecandidatebound");self.prepare_body(body)}\n fn prepare_body(&self,')])
edit('native_login_form.rs',[
 ('use zeroize::{Zeroize,Zeroizing};','use zeroize::{Zeroize,Zeroizing};\nuse super::{login_wire,source_send};'),
 ('Hidden,Editing,AwaitingAuthorities','Hidden,Editing,AwaitingAuthorities,Sending,AwaitingReply'),
 ('pub staged_password_len:usize}','pub staged_password_len:usize,pub write_serial:Option<u64>}'),
 ('staged:Option<Staged>}','staged:Option<Staged>,write_serial:Option<u64>}'),
 ('staged:None}}','staged:None,write_serial:None}}'),
 ('self.staged.take();}','self.staged.take();self.write_serial=None;}'),
 ('staged_password_len:p}}','staged_password_len:p,write_serial:self.write_serial}}'),
 (' pub(crate) fn stage(&mut self,',''' pub(crate) fn prepare_fixture_body(&self,c:&Control,local:[u8;4],environment:&login_wire::FixtureEnvironment)->Result<Zeroizing<Vec<u8>>>{self.admit(c)?;ensure!(self.stage==Stage::AwaitingAuthorities,"logininputnotstagedorwritealreadyreserved");let s=self.staged.as_ref().context("stagedlogininputabsent")?;login_wire::serialize_fixture_login(&s.account[..s.account_len],&s.password[..s.password_len],local,environment)}
 pub(crate) fn sending(&mut self,c:&Control,serial:u64)->Result<()>{self.admit(c)?;ensure!(self.stage==Stage::AwaitingAuthorities,"logininputalreadyreserved");let next=self.next()?;self.revision=next;self.stage=Stage::Sending;self.write_serial=Some(serial);Ok(())}
 pub(crate) fn write_report(&mut self,serial:u64,p:&source_send::Progress)->Result<()>{if self.write_serial==Some(serial)&&matches!(p,source_send::Progress::Complete){let next=self.next()?;self.clear();self.revision=next;self.stage=Stage::AwaitingReply;}Ok(())}
 pub(crate) fn stage(&mut self,''')])
print('R501 typedfixturecssd/chdch +actualownedloopback readerwriter +Sending/AwaitingReply wired')
