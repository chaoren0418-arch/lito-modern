//! Bounded plain-text/action interpretation of the qualified Chinese rental HTML.
//! No browser, script execution, path interpolation, or inferred action buttons.
use anyhow::{bail, ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::{Path,PathBuf}, sync::{atomic::{AtomicBool,Ordering},mpsc,Arc},thread::JoinHandle};

pub const MAX_BYTES:usize=16*1024;
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Action {pub command:String,pub label:String,pub navigate:bool}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Document {pub text:String,pub actions:Vec<Action>,pub images:Vec<u16>}
/// Formal openRentalTownHypertext sends no arguments for the initial inn page.
/// Its #0 speaker label comes from the same received actor, never a guessed name.
pub fn resolved_arguments(resource:&str,args:&[String],received_npc_name:&str)->Vec<String>{
    if resource.eq_ignore_ascii_case("inn")&&args.is_empty(){vec![received_npc_name.into()]}else{args.to_vec()}
}
impl Document{
    pub fn text_with_args(&self,args:&[String])->Result<String>{
        ensure!(args.len()<=256&&args.iter().all(|a|a.len()<=768&&!a.chars().any(char::is_control)&&!a.contains(['\u{e000}','\u{e001}'])),"NPC argument budget/control");
        let mut out=String::new();let mut remaining=self.text.as_str();
        while let Some(i)=remaining.find('\u{e000}'){
            out.push_str(&remaining[..i]);remaining=&remaining[i+'\u{e000}'.len_utf8()..];
            let end=remaining.find('\u{e001}').context("NPC variable end")?;let index=remaining[..end].parse::<usize>()?;
            out.push_str(args.get(index).context("NPC variable was not supplied")?);remaining=&remaining[end+'\u{e001}'.len_utf8()..];
        }out.push_str(remaining);ensure!(out.len()<=MAX_BYTES*3,"NPC expanded text budget");Ok(out)
    }
}
pub const MEMBERS:&[&str]=&[
    "teleportme-c.html",
    "inn-c.html","inn2-c.html","inn20-c.html","inn4-c.html",
    "aria_citizen-c.html",
    "aria_end-c.html",
    "aria_end_l-c.html",
    "aria_inn-c.html",
    "aria_start-c.html",
    "click_aid-c.html",
    "click_start-c.html",
    "ezee_citizen-c.html",
    "ezee_end-c.html",
    "ezee_end_l-c.html",
    "ezee_start-c.html",
    "love_aid-c.html",
    "love_start-c.html",
    "maya_aid-c.html",
    "maya_citizen-c.html",
    "maya_end-c.html",
    "maya_end_l-c.html",
    "maya_start-c.html",
    "maya_trade-c.html",
    "paper_aid-c.html",
    "paper_start-c.html",
    "phan_citizen-c.html",
    "phan_end_k-c.html",
    "phan_end_l-c.html",
    "phan_history-c.html",
    "phan_item-c.html",
    "phan_start-c.html",
    "soul_citizen-c.html",
    "soul_end-c.html",
    "soul_end_k-c.html",
    "soul_end_l-c.html",
    "soul_start-c.html",
];
pub fn member(resource:&str)->Result<&'static str>{
    let key=resource.to_ascii_lowercase();
    MEMBERS.iter().copied().find(|name|name.strip_suffix("-c.html")==Some(key.as_str())).with_context(||format!("此 NPC 對話格式尚未支援：{resource}"))
}
fn qualified_request(command:&str)->bool{
    matches!(command,"LT_TELEPORT_B"|"LT_TELEPORT_C"|"buy"|"sell"|"room"|"return"|"enter"|"request LTC light jewel"|"request LTC emblem of authority"|"request LTC emblem of craftsman"|"request LTC emblem of delight"|"request LTC emblem of peace")
}
fn attributes(mut s:&str)->Result<BTreeMap<String,String>>{
    let mut values=BTreeMap::new();
    while !s.trim().is_empty(){
        s=s.trim_start();let end=s.find(|c:char|!c.is_ascii_alphanumeric()&&c!='_').unwrap_or(s.len());
        ensure!(end>0,"dialog attribute name");let key=s[..end].to_ascii_lowercase();s=s[end..].trim_start();
        s=s.strip_prefix('=').context("dialog attribute assignment")?.trim_start();
        let value;
        if s.starts_with('"')||s.starts_with('\''){
            let quote=s.as_bytes()[0] as char;s=&s[1..];let end=s.find(quote).context("dialog attribute quote")?;value=&s[..end];s=&s[end+1..];
        }else{let end=s.find(char::is_whitespace).unwrap_or(s.len());value=&s[..end];s=&s[end..];}
        ensure!(!value.is_empty()&&value.len()<=64&&values.len()<4&&values.insert(key,value.into()).is_none(),"dialog attribute budget/duplicate");
    }
    Ok(values)
}
pub fn parse(bytes:&[u8])->Result<Document>{
    parse_template(bytes,false)
}
fn parse_template(bytes:&[u8],allow_original_open_body:bool)->Result<Document>{
    use windows::Win32::Globalization::{MultiByteToWideChar,MB_ERR_INVALID_CHARS};
    ensure!(!bytes.is_empty()&&bytes.len()<=MAX_BYTES,"dialog byte budget");
    let mut wide=vec![0u16;bytes.len()];let n=unsafe{MultiByteToWideChar(950,MB_ERR_INVALID_CHARS,bytes,Some(&mut wide))};
    ensure!(n>0,"invalid dialog CP950");let source=String::from_utf16(&wide[..n as usize])?;
    ensure!(!source.contains(['\u{e000}','\u{e001}']),"reserved NPC template character");
    ensure!(!source.chars().any(|c|c.is_control()&&!matches!(c,'\r'|'\n'|'\t')),"dialog control character");
    let mut doc=Document{text:String::new(),actions:vec![],images:vec![]};let mut rest=source.as_str();
    let mut body=false;let mut closed=false;let mut tags=0;let mut anchor:Option<Action>=None;
    while !rest.is_empty(){
        if let Some(after)=rest.strip_prefix('<'){
            let end=after.find('>').context("unterminated dialog tag")?;let tag=after[..end].trim();rest=&after[end+1..];
            tags+=1;ensure!(tags<=256&&!tag.contains('<'),"dialog tag budget");
            let split=tag.find(char::is_whitespace).unwrap_or(tag.len());let name=tag[..split].to_ascii_lowercase();let attrs=attributes(&tag[split..])?;
            ensure!(!closed,"tag after dialog body");
            if name=="body"{ensure!(!body&&attrs.is_empty(),"duplicate dialog body");body=true;continue;}
            ensure!(body,"tag before dialog body");
            match name.as_str(){
                "/body"=>{ensure!(attrs.is_empty()&&anchor.is_none(),"unclosed dialog action");closed=true;},
                "p"=>{ensure!(attrs.is_empty()||(attrs.len()==1&&attrs.get("align").is_some_and(|v|v=="left")),"unsupported paragraph attributes");ensure!(anchor.is_none(),"paragraph in action");if !doc.text.ends_with('\n'){doc.text.push('\n');}},
                "/p"|"br"|"br/"=>{ensure!(attrs.is_empty()&&anchor.is_none(),"unsupported paragraph break");if !doc.text.ends_with('\n'){doc.text.push('\n');}},
                "font"=>ensure!(attrs.len()==1&&attrs.get("fg").is_some_and(|v|v.len()==6&&v.bytes().all(|b|b.is_ascii_hexdigit())),"unsupported font attributes"),
                "/font"=>ensure!(attrs.is_empty(),"closing font attributes"),
                "img"=>{
                    // Preserve the two original TeleportMe image references.
                    // This plain-text view does not claim original icon layout.
                    ensure!(anchor.is_none()&&attrs.len()==1&&doc.images.len()<16,"NPC image shape/budget");
                    let src=attrs.get("src").context("NPC image source")?;
                    let id=match src.as_str(){"#7311"=>7311,"#7312"=>7312,_=>bail!("unqualified NPC image")};
                    doc.images.push(id);
                },
                "var"=>{ensure!(anchor.is_none()&&attrs.len()==1,"NPC variable shape");let value=attrs.get("src").context("NPC variable source")?;let index=value.strip_prefix('#').context("NPC variable index")?.parse::<u8>()?;doc.text.push_str(&format!("\u{e000}{index}\u{e001}"));},
                "a"=>{ensure!(anchor.is_none()&&attrs.len()==1&&doc.actions.len()<16,"dialog action shape");let navigate=attrs.contains_key("link");let command=attrs.get(if navigate{"link"}else{"action"}).context("dialog action missing")?;if navigate{member(command)?;}else{ensure!(qualified_request(command),"unqualified dialog action");}anchor=Some(Action{command:command.clone(),label:String::new(),navigate});},
                "/a"=>{ensure!(attrs.is_empty(),"closing action attributes");let mut a=anchor.take().context("unmatched dialog action")?;a.label=a.label.trim().into();ensure!(!a.label.is_empty()&&!doc.actions.iter().any(|v|v.command==a.command&&v.navigate==a.navigate),"empty/duplicate dialog action");doc.actions.push(a);},
                _=>bail!("unsupported dialog tag {name}"),
            }
        }else{
            let end=rest.find('<').unwrap_or(rest.len());let text=&rest[..end];rest=&rest[end..];
            ensure!(!text.contains('&'),"unqualified dialog entity");
            if !body||closed{ensure!(text.trim().is_empty(),"text outside dialog body");continue;}
            let dest=if let Some(a)=anchor.as_mut(){&mut a.label}else{&mut doc.text};
            // Source indentation is layout whitespace; explicit p/br own line breaks.
            for ch in text.chars(){if ch.is_whitespace(){if !dest.is_empty()&&!dest.ends_with([' ','\n']){dest.push(' ');}}else{dest.push(ch);}}
            ensure!(dest.len()<=MAX_BYTES*3,"dialog output budget");
        }
    }
    ensure!(body&&(closed||allow_original_open_body)&&anchor.is_none(),"incomplete dialog body");doc.text=doc.text.trim().into();Ok(doc)
}

#[derive(Clone,Debug)]
pub struct Source {pub member:String,pub bytes:usize,pub sha256:String}
pub struct Catalog {documents:BTreeMap<&'static str,Document>,pub sources:Vec<Source>,pub images:Option<super::native_npc_images::Bank>,pub image_error:Option<String>}
impl Catalog{
    pub fn load(root:&Path,cancel:&AtomicBool)->Result<Self>{
        let lt=root.join("LiTo");let index=super::native_world_scene::read_pin(&lt.join("LTtext.idx"),1024*1024,cancel)?;
        let pak=super::native_world_scene::read_pin(&lt.join("LTtext.pak"),1024*1024,cancel)?;
        let mut catalog=Self{documents:BTreeMap::new(),sources:vec![],images:None,image_error:None};
        for &name in MEMBERS{
            ensure!(!cancel.load(Ordering::Acquire),"dialog loader cancelled");
            let bytes=super::native_resources::read_native_text_optional(&lt,index,pak,name,cancel)?.context("qualified NPC resource missing")?;
            // These exact original hotel pages end at EOF without a closing body.
            let document=parse_template(&bytes,matches!(name,"inn2-c.html"|"inn20-c.html"|"inn4-c.html"))?;catalog.sources.push(Source{member:name.into(),bytes:bytes.len(),sha256:format!("{:X}",Sha256::digest(&bytes))});catalog.documents.insert(name,document);
        }
        match super::native_npc_images::Bank::load(root,cancel){Ok(images)=>catalog.images=Some(images),Err(e)=>catalog.image_error=Some(format!("{e:#}"))}
        ensure!(!cancel.load(Ordering::Acquire),"dialog loader cancelled");
        Ok(catalog)
    }
    pub fn get(&self,resource:&str)->Result<&Document>{self.documents.get(member(resource)?).context("NPC document not loaded")}
}
/// Immutable qualified-document cache; a worker never retains a player or NPC identity.
pub struct Service{receiver:mpsc::Receiver<Result<Catalog>>,worker:Option<JoinHandle<()>>,cancel:Arc<AtomicBool>,catalog:Option<Catalog>,pub error:Option<String>}
impl Service{
    pub fn start(root:PathBuf)->Result<Self>{let(send,receiver)=mpsc::sync_channel(1);let cancel=Arc::new(AtomicBool::new(false));let token=cancel.clone();
        let worker=std::thread::Builder::new().name("native-npc-documents".into()).spawn(move||{let _=send.send(Catalog::load(&root,&token));})?;
        Ok(Self{receiver,worker:Some(worker),cancel,catalog:None,error:None})}
    pub fn poll(&mut self)->bool{if !self.worker.as_ref().is_some_and(|w|w.is_finished()){return false;}let result=self.receiver.try_recv().context("NPC loader ended without result").and_then(|v|v);let _=self.worker.take().unwrap().join();
        if !self.cancel.load(Ordering::Acquire){match result{Ok(v)=>self.catalog=Some(v),Err(e)=>self.error=Some(format!("{e:#}"))}}true}
    pub fn catalog(&self)->Option<&Catalog>{self.catalog.as_ref()}
    pub fn cancel(&mut self){self.cancel.store(true,Ordering::Release);self.catalog=None;self.error=None;}
    pub fn quiescent(&self)->bool{self.worker.is_none()}
}
impl Drop for Service{fn drop(&mut self){self.cancel();if let Some(w)=self.worker.take(){let _=w.join();}}}
