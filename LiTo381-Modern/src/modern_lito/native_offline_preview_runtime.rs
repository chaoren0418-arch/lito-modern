//! UI thread owns only presentation. One cancellable worker owns all resource IO/render.
use super::{native_offline_preview_resources::{self,Inputs},native_offline_map_flow::{OfflineMapFlow,MapChoice,Page,PreviewFrame},compositor::{Canvas,Viewport},scene::CameraRect};
use anyhow::{ensure,Context,Result};
use std::{sync::{Arc,mpsc::{self,Receiver,SyncSender},atomic::{AtomicBool,AtomicU64,Ordering}},thread::{self,JoinHandle}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum UiPage {Booting,Menu,Loading,Preview,Closing,Closed}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Action {Enter,Pan(i32,i32),Back,Close}
pub fn key_action(page:UiPage,key:u32,control:bool,alt:bool,repeat:bool)->Option<Action>{
 if control||alt||repeat||matches!(page,UiPage::Closing|UiPage::Closed){return None;}
 match key {0x1b=>Some(Action::Back),0x25 if page==UiPage::Preview=>Some(Action::Pan(-48,0)),0x27 if page==UiPage::Preview=>Some(Action::Pan(48,0)),0x26 if page==UiPage::Preview=>Some(Action::Pan(0,-24)),0x28 if page==UiPage::Preview=>Some(Action::Pan(0,24)),_=>None}
}
#[derive(Default)]pub struct Progress {pub initialized:AtomicBool,pub frames_produced:AtomicU64,pub retired:AtomicBool,pub load_active:AtomicBool}
enum Command {Enter{generation:u64,cancel:Arc<AtomicBool>},Pan{generation:u64,dx:i32,dy:i32},Back}
struct Pixels {bgra:Arc<[u8]>,camera:CameraRect}
enum Event {Ready(MapChoice,Option<Arc<super::native_offline_menu_skin::Skin>>),InitFailed(String),Frame{generation:u64,pixels:Pixels},Failed{generation:u64,error:String}}
fn pixels(frame:PreviewFrame)->Result<Pixels>{let mut c=Canvas::new(frame.camera.width,frame.camera.height)?;c.blit_rgba(frame.rgba()?,frame.camera.width,frame.camera.height,0,0)?;let mut bgra=Vec::new();c.copy_bgra(&mut bgra)?;Ok(Pixels{bgra:bgra.into(),camera:frame.camera})}
#[derive(Clone)]pub struct UiFrame {pixels:Arc<Pixels>,guard:Arc<AtomicU64>,revision:u64}
impl UiFrame {
 pub fn bgra(&self)->Result<&[u8]>{ensure!(self.guard.load(Ordering::Acquire)==self.revision,"UI frame retired");Ok(&self.pixels.bgra)}
 pub fn camera(&self)->CameraRect{self.pixels.camera}
 pub fn viewport(&self,width:i32,height:i32)->Option<Viewport>{Viewport::fit(width,height,self.pixels.camera.width,self.pixels.camera.height)}
}
pub struct Runtime {skin:Option<Arc<super::native_offline_menu_skin::Skin>>,page:UiPage,status:String,choice:Option<MapChoice>,generation:u64,revision:u64,frame:Option<UiFrame>,guard:Option<Arc<AtomicU64>>,cancel:Option<Arc<AtomicBool>>,pan_pending:bool,commands:Option<SyncSender<Command>>,events:Receiver<Event>,worker:Option<JoinHandle<()>>,shutdown:Arc<AtomicBool>,progress:Arc<Progress>,pub stale_results_rejected:u64}
struct Retirement(Arc<Progress>);impl Drop for Retirement {fn drop(&mut self){self.0.load_active.store(false,Ordering::Release);self.0.retired.store(true,Ordering::Release);}}
fn worker(inputs:Inputs,skin_inputs:Option<Inputs>,commands:Receiver<Command>,events:mpsc::Sender<Event>,shutdown:Arc<AtomicBool>,progress:Arc<Progress>){
 let _retirement=Retirement(progress.clone());
 let initialized=(||{let skin=skin_inputs.as_ref().map(super::native_offline_menu_skin::Skin::load).transpose()?.map(Arc::new);let(c,b)=native_offline_preview_resources::load(&inputs)?;let mut f=OfflineMapFlow::new(true,c,vec![b])?;f.select(1,564)?;Ok::<_,anyhow::Error>((f,skin))})();
 let (mut flow,skin)=match initialized{Ok(f)=>f,Err(e)=>{let _=events.send(Event::InitFailed(format!("{e:#}")));return;}};
 if shutdown.load(Ordering::Acquire){return;}
 let _=events.send(Event::Ready(flow.selected().unwrap().clone(),skin));progress.initialized.store(true,Ordering::Release);
 let mut active:Option<(u64,Arc<AtomicBool>)>=None;
 while let Ok(command)=commands.recv(){
  if shutdown.load(Ordering::Acquire){break;}
  if active.as_ref().is_some_and(|(_,c)|c.load(Ordering::Acquire)){if flow.page()!=Page::Menu{let _=flow.back();}active=None;}
  match command {
   Command::Back=>{if flow.page()!=Page::Menu{let _=flow.back();}active=None;},
   Command::Enter{generation,cancel}=>{
    if cancel.load(Ordering::Acquire){continue;}
    if flow.page()!=Page::Menu{let _=flow.back();}progress.load_active.store(true,Ordering::Release);
    let result=(||{let ready=flow.begin_entry()?.load(&cancel)?;let frame=flow.complete_entry(ready)?;pixels(frame)})();
    progress.load_active.store(false,Ordering::Release);
    if cancel.load(Ordering::Acquire)||shutdown.load(Ordering::Acquire){if flow.page()!=Page::Menu{let _=flow.back();}active=None;continue;}
    match result{Ok(pixels)=>{active=Some((generation,cancel));let _=events.send(Event::Frame{generation,pixels});progress.frames_produced.fetch_add(1,Ordering::Release);},Err(e)=>{if flow.page()!=Page::Menu{let _=flow.back();}let _=events.send(Event::Failed{generation,error:format!("{e:#}")});active=None;}}
   },
   Command::Pan{generation,dx,dy}=>{
    let Some((g,cancel))=&active else{continue;};if *g!=generation||cancel.load(Ordering::Acquire){continue;}
    let result=flow.pan(dx,dy,cancel).and_then(pixels);
    if cancel.load(Ordering::Acquire)||shutdown.load(Ordering::Acquire){if flow.page()!=Page::Menu{let _=flow.back();}active=None;continue;}
    match result{Ok(pixels)=>{let _=events.send(Event::Frame{generation,pixels});progress.frames_produced.fetch_add(1,Ordering::Release);},Err(e)=>{let _=flow.back();let _=events.send(Event::Failed{generation,error:format!("{e:#}")});active=None;}}
   }
  }
 }
 // flow/assets drop before the retirement guard publishes actual worker retirement.
}
impl Runtime {
 pub fn start(inputs:Inputs)->Result<Self>{Self::start_inner(inputs,None)}
 pub fn start_with_menu_skin(inputs:Inputs,skin_inputs:Inputs)->Result<Self>{Self::start_inner(inputs,Some(skin_inputs))}
 pub fn skin(&self)->Option<Arc<super::native_offline_menu_skin::Skin>>{self.skin.clone()}
 fn start_inner(inputs:Inputs,skin_inputs:Option<Inputs>)->Result<Self>{let(tx,rx)=mpsc::sync_channel(4);let(out_tx,out_rx)=mpsc::channel();let shutdown=Arc::new(AtomicBool::new(false));let progress=Arc::new(Progress::default());let(s,p)=(shutdown.clone(),progress.clone());let worker=thread::Builder::new().name("modern-offline-preview".into()).spawn(move||worker(inputs,skin_inputs,rx,out_tx,s,p))?;Ok(Self{skin:None,page:UiPage::Booting,status:"正在讀取場地清單…".into(),choice:None,generation:0,revision:0,frame:None,guard:None,cancel:None,pan_pending:false,commands:Some(tx),events:out_rx,worker:Some(worker),shutdown,progress,stale_results_rejected:0})}
 pub fn page(&self)->UiPage{self.page}pub fn status(&self)->&str{&self.status}pub fn choice(&self)->Option<&MapChoice>{self.choice.as_ref()}pub fn frame(&self)->Option<UiFrame>{self.frame.clone()}pub fn pan_pending(&self)->bool{self.pan_pending}pub fn progress(&self)->Arc<Progress>{self.progress.clone()}
 fn invalidate(&mut self){if let Some(c)=self.cancel.take(){c.store(true,Ordering::Release);}if let Some(g)=self.guard.take(){g.store(0,Ordering::Release);}self.frame=None;self.pan_pending=false;}
 fn next_generation(&mut self)->Result<u64>{self.generation=self.generation.checked_add(1).context("UI generation exhausted")?;Ok(self.generation)}
 pub fn handle(&mut self,action:Action)->Result<()>{
  match action {
   Action::Enter=>{ensure!(self.page==UiPage::Menu&&self.choice.is_some(),"場地尚未準備好");let generation=self.next_generation()?;let cancel=Arc::new(AtomicBool::new(false));self.commands.as_ref().context("worker retired")?.try_send(Command::Enter{generation,cancel:cancel.clone()}).map_err(|_|anyhow::anyhow!("背景工作尚未可接受新載入，請稍後重試"))?;self.cancel=Some(cancel);self.page=UiPage::Loading;self.status="正在載入 Battle Ground…可隨時返回".into();},
   Action::Pan(dx,dy)=>{ensure!(self.page==UiPage::Preview&&!self.pan_pending,"視角更新尚未完成");self.commands.as_ref().context("worker retired")?.try_send(Command::Pan{generation:self.generation,dx,dy}).map_err(|_|anyhow::anyhow!("視角工作佇列已滿"))?;self.pan_pending=true;},
   Action::Back=>{ensure!(matches!(self.page,UiPage::Loading|UiPage::Preview),"已在場地選單");self.next_generation()?;self.invalidate();self.page=UiPage::Menu;self.status="已返回場地選單".into();if let Some(tx)=&self.commands{let _=tx.try_send(Command::Back);}},
   Action::Close=>{if matches!(self.page,UiPage::Closing|UiPage::Closed){return Ok(());}self.next_generation()?;self.invalidate();self.shutdown.store(true,Ordering::Release);self.commands.take();self.skin=None;self.page=if self.worker.is_none(){UiPage::Closed}else{UiPage::Closing};self.status=if self.page==UiPage::Closed{"已結束"}else{"正在結束背景工作…"}.into();}
  }Ok(())
 }
 pub fn poll(&mut self)->bool {let mut changed=false;
  for _ in 0..16{let Ok(event)=self.events.try_recv() else{break;};changed=true;
   if matches!(self.page,UiPage::Closing|UiPage::Closed){self.stale_results_rejected+=1;continue;}
   match event {
    Event::Ready(choice,skin)=>{if self.page==UiPage::Booting{self.skin=skin;self.choice=Some(choice);self.page=UiPage::Menu;self.status="選擇場地後進入離線預覽".into();}},
    Event::InitFailed(error)=>{self.page=UiPage::Menu;self.status=format!("無法讀取場地：{}",error.chars().take(500).collect::<String>());},
    Event::Frame{generation,pixels}=>{if generation!=self.generation||!matches!(self.page,UiPage::Loading|UiPage::Preview)||self.cancel.as_ref().is_none_or(|c|c.load(Ordering::Acquire)){self.stale_results_rejected+=1;continue;}self.revision+=1;let guard=self.guard.get_or_insert_with(||Arc::new(AtomicU64::new(0))).clone();guard.store(self.revision,Ordering::Release);self.frame=Some(UiFrame{pixels:Arc::new(pixels),guard,revision:self.revision});self.page=UiPage::Preview;self.pan_pending=false;self.status="離線預覽 · 方向鍵移動視角，Esc 返回".into();},
    Event::Failed{generation,error}=>{if generation!=self.generation{self.stale_results_rejected+=1;continue;}self.invalidate();self.page=UiPage::Menu;self.status=format!("載入失敗，可重試或返回：{}",error.chars().take(500).collect::<String>());}
   }
  }
  if self.worker.as_ref().is_some_and(JoinHandle::is_finished){let result=self.worker.take().unwrap().join();changed=true;if self.page==UiPage::Closing{self.page=UiPage::Closed;self.status="已結束".into();}else if result.is_err(){self.invalidate();self.page=UiPage::Menu;self.status="背景工作已停止，請退出後重新開啟預覽".into();}}
  changed
 }
}
impl Drop for Runtime {fn drop(&mut self){self.invalidate();self.shutdown.store(true,Ordering::Release);self.commands.take();/* normal UI waits for Closed; emergency drop only requests cooperative cleanup */}}
#[cfg(test)]#[path="native_offline_preview_runtime_tests.rs"]mod tests;
