//! Local profile1 actor92/93 consumer. Immutable identity and real resource
//! worker binding only; formal KNOWN/live transport/LOADED remain unchanged.
use super::{application_protocol as app,protocol,battle_world as world,battle_follow,
    native_catalog_draft as catalog,native_canonical_draft as native,native_actor_inspection as body,
    native_resources as r,native_action_inspection as action};
use sha2::{Digest,Sha256};
use std::{io::Read,path::Path,sync::{Arc,atomic::{AtomicBool,Ordering}}};
type Result<T>=std::result::Result<T,app::ApplicationError>;
pub const ACTOR_BIT:u32=1024;
pub const IDENTITY_BYTES:usize=952;
pub const REQUEST_BYTES:usize=348;
fn invalid()->app::ApplicationError{app::error(app::ErrorKind::MalformedFrame,"invalid local Native actor dependency")}
fn u16_at(b:&[u8],a:usize)->u16{u16::from_be_bytes(b[a..a+2].try_into().unwrap())}
fn u32_at(b:&[u8],a:usize)->u32{u32::from_be_bytes(b[a..a+4].try_into().unwrap())}
fn u64_at(b:&[u8],a:usize)->u64{u64::from_be_bytes(b[a..a+8].try_into().unwrap())}
fn header(b:&[u8],kind:u16,min:usize,max:usize,id:Option<u32>)->Result<usize>{
    if b.len()<16{return Err(invalid());}let n=u32_at(b,4) as usize;
    if &b[..4]!=b"LTM1"||u16_at(b,12)!=kind||u16_at(b,14)!=0||u32_at(b,8)==0||id.is_some_and(|v|v!=u32_at(b,8))||!(min..=max).contains(&n){return Err(invalid());}Ok(n)
}
fn pin(b:&[u8],a:usize,min:u64,max:u64)->Result<r::BytePin>{let p=r::BytePin{bytes:u64_at(b,a),sha256:b[a+8..a+40].try_into().unwrap()};if !(min..=max).contains(&p.bytes)||p.sha256==[0;32]{return Err(invalid());}Ok(p)}
fn range(b:&[u8],a:usize,pak:u64,max:u64)->Result<(u64,r::BytePin)>{let off=u64_at(b,a);let p=pin(b,a+8,1,max)?;if off.checked_add(p.bytes).is_none_or(|e|e>pak){return Err(invalid());}Ok((off,p))}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ActorIdentity {wire:[u8;IDENTITY_BYTES]}
impl ActorIdentity {
    pub fn decode(b:&[u8])->Result<Self>{
        if b.len()!=952||b[..24]!=[0,1,0,4,0,1,0,0,0,0,0,0,0,0,12,153,0,0,0,3,0,0,0,1]{return Err(invalid());}
        for at in [24,56,88]{if b[at..at+32]==[0;32]{return Err(invalid());}}
        catalog::CatalogIdentity::decode(&b[120..288])?;
        pin(b,288,4,4*1024*1024)?;let text=pin(b,328,1,256*1024*1024)?;
        range(b,368,text.bytes,8*1024*1024)?;pin(b,416,1,8*1024*1024)?;
        pin(b,456,4,4*1024*1024)?;let sprite=pin(b,496,1,2*1024*1024*1024)?;
        for (i,shard) in [4,5,12,13].iter().enumerate(){let at=536+i*52;
            if u16_at(b,at)!=*shard||u16_at(b,at+2)!=0||!(1..=2*1024*1024*1024).contains(&u64_at(b,at+4)){return Err(invalid());}pin(b,at+12,4,4*1024*1024)?;
        }
        for (i,(image,shard)) in [(0,12),(1,13),(8,4),(9,5)].iter().enumerate(){let at=744+i*52;if u16_at(b,at)!=*image||b[at+2]!=*shard||b[at+3]!=0{return Err(invalid());}range(b,at+4,sprite.bytes,1024*1024)?;}
        Ok(Self{wire:b.try_into().unwrap()})
    }
    pub fn bytes(&self)->&[u8;952]{&self.wire}
    pub fn digest(&self)->[u8;32]{let mut h=Sha256::new();h.update(b"LITO-NATIVE-ACTOR-DEPENDENCY-IDENTITY\0");h.update(self.wire);h.finalize().into()}
    pub fn catalog(&self)->catalog::CatalogIdentity{catalog::CatalogIdentity::decode(&self.wire[120..288]).unwrap()}
    pub fn body_pins(&self)->body::NativeBaseBodyPins{let b=&self.wire;body::NativeBaseBodyPins{text_index:pin(b,288,4,u64::MAX).unwrap(),text_pak:pin(b,328,1,u64::MAX).unwrap(),decoded_catalog:pin(b,416,1,u64::MAX).unwrap(),sprite_index:pin(b,456,4,u64::MAX).unwrap(),sprite_pak:pin(b,496,1,u64::MAX).unwrap(),
        shard_misses:(0..4).map(|i|{let a=536+i*52;(u16_at(b,a) as u8,pin(b,a+12,4,u64::MAX).unwrap(),u64_at(b,a+4))}).collect()}}
    pub fn ranges(&self)->body::NativeBodyRanges{let b=&self.wire;let (text_offset,text_raw)=range(b,368,u64::MAX,u64::MAX).unwrap();body::NativeBodyRanges{text_offset,text_raw,
        members:(0..4).map(|i|{let a=744+i*52;let (off,p)=range(b,a+4,u64::MAX,u64::MAX).unwrap();(u32::from(u16_at(b,a)),off,p)}).collect()}}
    fn verify_provenance(&self,root:&Path,cancel:&AtomicBool)->Result<()> {
        let b=&self.wire;
        if hash_hex("ce6ee68ada8fd37e6c096b1a27e46cb6888f1179f85defb5007ac60287360292")?!=b[24..56]
            || hash_hex("62f7f690034189cfd19c0cf87b3b75df8c1bd77f96426c72faa731c075eb2e83")?!=b[88..120]{return Err(invalid());}
        let bytes=r::read_actor_provenance(root,b[56..88].try_into().unwrap(),cancel).map_err(|_|invalid())?;
        let v:serde_json::Value=serde_json::from_slice(&bytes).map_err(|_|invalid())?;
        let pins=v["observed_original_and_Modern_pins"].as_array().ok_or_else(invalid)?;if pins.len()!=8{return Err(invalid());}
        let p=self.body_pins();let mut expected=vec![("Text.idx".to_string(),p.text_index),("Text.pak".to_string(),p.text_pak),("Sprite.idx".to_string(),p.sprite_index),("Sprite.pak".to_string(),p.sprite_pak)];
        expected.extend(p.shard_misses.iter().map(|(n,p,_)|(format!("Sprite{n:02}.idx"),*p)));
        for (name,p) in expected{let selected:Vec<_>=pins.iter().filter(|row|row["name"].as_str()==Some(&name)).collect();if selected.len()!=1||selected[0]["bytes"].as_u64()!=Some(p.bytes)||json_hash(&selected[0]["sha256"])?!=p.sha256{return Err(invalid());}}
        let ranges=self.ranges();let text=&v["base_text_member"];
        if text["offset"].as_u64()!=Some(ranges.text_offset)||text["bytes"].as_u64()!=Some(ranges.text_raw.bytes)||json_hash(&text["packed_sha256"])?!=ranges.text_raw.sha256||json_hash(&text["decoded_sha256"])?!=p.decoded_catalog.sha256{return Err(invalid());}
        let routes=v["routes"].as_array().ok_or_else(invalid)?;if routes.len()!=4{return Err(invalid());}
        for (route,(image,off,p)) in routes.iter().zip(&ranges.members){let name=format!("3225-{image}.spr");let shard=name.bytes().map(u32::from).sum::<u32>()%16;
            if route["image"].as_u64()!=Some(u64::from(*image))||route["member"].as_str()!=Some(&name)||route["shard"].as_u64()!=Some(u64::from(shard))||route["offset"].as_u64()!=Some(*off)||route["bytes"].as_u64()!=Some(p.bytes)||json_hash(&route["sha256"])?!=p.sha256{return Err(invalid());}}
        if v["stream_buffer_bytes"]!=65536||v["Sprite_pak_memory_allocation"]!=false||v["selected_member_limit"]!=1048576{return Err(invalid());}
        Ok(())
    }
}
fn hash_hex(s:&str)->Result<[u8;32]>{if s.len()!=64||!s.is_ascii(){return Err(invalid());}let mut out=[0;32];for (i,p) in s.as_bytes().chunks_exact(2).enumerate(){out[i]=u8::from_str_radix(std::str::from_utf8(p).map_err(|_|invalid())?,16).map_err(|_|invalid())?;}Ok(out)}
fn json_hash(v:&serde_json::Value)->Result<[u8;32]>{hash_hex(v.as_str().ok_or_else(invalid)?)}
#[derive(Clone,PartialEq,Eq)]
pub struct Request {pub expected_identity:[u8;32],pub actor_proof:[u8;32],pub catalog:catalog::Request}
impl Request {
    fn validate(&self)->Result<()>{let q=&self.catalog.native.canonical;if self.expected_identity==[0;32] && !(q.action==world::Action::Read&&q.match_id==0)||(q.action==world::Action::Loaded)!=(self.actor_proof!=[0;32]){return Err(invalid());}Ok(())}
    pub fn check_authority(&self,identity:&ActorIdentity,source:&native::SourceAttachment)->Result<()> {
        self.validate()?;self.catalog.check_authority(&identity.catalog(),source)?;
        if self.expected_identity!=[0;32]&&self.expected_identity!=identity.digest(){return Err(invalid());}
        if self.catalog.native.canonical.action==world::Action::Loaded{let mut h=Sha256::new();h.update(b"LITO-NATIVE-ACTOR-DEPENDENCY-LOAD-PROOF\0");h.update(identity.digest());h.update(self.catalog.catalog_load_proof);if self.actor_proof!=<[u8;32]>::from(h.finalize()){return Err(invalid());}}Ok(())
    }
}
#[derive(Clone,Copy,Default)]
pub struct ActorCodec {inner:Option<catalog::CatalogCodec>}
pub struct Reply {request:Request,identity:Option<ActorIdentity>,catalog:catalog::Reply}
impl Reply {pub fn request(&self)->&Request{&self.request}pub fn identity(&self)->Option<&ActorIdentity>{self.identity.as_ref()}pub fn catalog(&self)->&catalog::Reply{&self.catalog}}
impl ActorCodec {
    pub fn local_experiment(base:protocol::Ack,extensions:u32)->Result<Self>{if extensions&1792!=1792{return Err(invalid());}Ok(Self{inner:Some(catalog::CatalogCodec::local_experiment(base,extensions&!ACTOR_BIT)?)})}
    fn inner(self)->Result<catalog::CatalogCodec>{self.inner.ok_or_else(invalid)}
    pub fn maximum_result(self)->Result<usize>{Ok(1044+self.inner()?.maximum_result()?)}
    pub fn decode_request(self,b:&[u8])->Result<Request>{self.inner()?;header(b,92,348,348,None)?;if b.len()!=348||b[16..20]!=[0,1,0,4]||u32_at(b,8)!=u32_at(b,92){return Err(invalid());}let q=Request{expected_identity:b[20..52].try_into().unwrap(),actor_proof:b[52..84].try_into().unwrap(),catalog:self.inner()?.decode_request(&b[84..])?};q.validate()?;Ok(q)}
    pub fn encode_request(self,id:u32,q:&Request)->Result<Vec<u8>>{q.validate()?;let nested=self.inner()?.encode_request(id,&q.catalog)?;let mut b=app::header(id,92,348)?;b.extend([0,1,0,4]);b.extend(q.expected_identity);b.extend(q.actor_proof);b.extend(nested);Ok(b)}
    pub fn read_reply(self,r:&mut impl Read,expected:&[u8])->Result<Reply>{let q=self.decode_request(expected)?;let mut h=[0;16];r.read_exact(&mut h).map_err(|_|invalid())?;let n=header(&h,93,428,self.maximum_result()?,Some(u32_at(expected,8)))?;let mut b=Vec::new();b.try_reserve_exact(n).map_err(|_|invalid())?;b.extend(h);b.resize(n,0);r.read_exact(&mut b[16..]).map_err(|_|invalid())?;
        if b[16..84]!=expected[16..84]||b[88..92]!=[0;4]{return Err(invalid());}let size=u32_at(&b,84) as usize;if !matches!(size,0|952)||n<92+size+336{return Err(invalid());}
        let identity=if size==0{None}else{Some(ActorIdentity::decode(&b[92..1044])?)};let catalog=self.inner()?.decode_reply(&b[92+size..],&expected[84..])?;
        if identity.is_some()!=catalog.catalog().is_some(){return Err(invalid());}if let Some(i)=&identity{if Some(i.catalog())!=catalog.catalog(){return Err(invalid());}q.check_authority(i,catalog.native().source.as_ref().ok_or_else(invalid)?)?;}else if q.expected_identity!=[0;32]{return Err(invalid());}Ok(Reply{request:q,identity,catalog})}
    pub fn decode_reply(self,b:&[u8],q:&[u8])->Result<Reply>{let mut r=b;let reply=self.read_reply(&mut r,q)?;if !r.is_empty(){return Err(invalid());}Ok(reply)}
}
pub struct ActorOwner {inner:catalog::CatalogOwner,current:Option<(native::WorkTicket,ActorIdentity)>}
pub struct ActorRendererJob {inner:catalog::CatalogRendererJob,identity:ActorIdentity}
#[derive(Clone)]
pub struct VerifiedActorBody {snapshot:body::NativeActorSnapshot,identity:ActorIdentity,cancel:Arc<AtomicBool>}
pub struct ActorPartialCompletion {inner:catalog::CatalogPartialCompletion,body:VerifiedActorBody}
fn profile_data(d:&super::battle_payload::UnvalidatedData)->Result<()> {
    let p=&d.personal.player;
    let m=d.room.members.0.iter().find(|m|m.identity.source==p.source&&m.identity.id==p.object).ok_or_else(invalid)?;
    let a=d.actors.0.iter().find(|a|a.entity==d.personal.entity).ok_or_else(invalid)?;
    if m.class!=0||m.sex!=0||!matches!(a.heading,0|1){return Err(invalid());}Ok(())
}
impl ActorOwner {
    pub fn local_experiment(c:ActorCodec)->Result<Self>{Ok(Self{inner:catalog::CatalogOwner::local_experiment(c.inner()?)?,current:None})}
    pub fn stage(&mut self,s:native::Scope,r:Reply)->Result<native::WorkTicket>{let i=r.identity.ok_or_else(invalid)?;if let Some(d)=r.catalog.native().canonical.data.as_ref(){profile_data(d)?;}if self.current.as_ref().is_some_and(|(_,old)|old!=&i){return Err(invalid());}let t=self.inner.stage(s,r.catalog)?;self.current=Some((t,i));Ok(t)}
    pub fn begin_renderer(&self,t:native::WorkTicket,o:&battle_follow::Observation)->Result<ActorRendererJob>{let (_,i)=self.current.as_ref().filter(|(ticket,_)|*ticket==t).ok_or_else(invalid)?;Ok(ActorRendererJob{inner:self.inner.begin_renderer(t,o)?,identity:i.clone()})}
    pub fn complete_partial(&mut self,t:native::WorkTicket,c:ActorPartialCompletion)->Result<()>{let (_,i)=self.current.as_ref().filter(|(ticket,_)|*ticket==t).ok_or_else(invalid)?;if &c.body.identity!=i||c.body.cancel.load(Ordering::Acquire){return Err(invalid());}self.inner.complete_partial(t,c.inner)}
    pub fn retire(&mut self){self.inner.retire();self.current=None;}
    pub fn partial_available(&self)->bool{self.inner.partial_available()}
    pub fn loaded_admission_available(&self)->bool{false}
}
impl VerifiedActorBody {
    pub(crate) fn scene_frame(&self,job:&native::RendererJob,a:body::BodyAction,frame:usize)->Result<(&super::sprite::SpriteFrame,super::world_depth::NativeDrawCell)> {
        self.snapshot.scene_frame(job,a,frame).map_err(|_|invalid())
    }
    pub fn render(&self,a:body::BodyAction,ms:u64)->Result<body::NativeActorSlice>{self.snapshot.render(a,ms).map_err(|_|invalid())}
    pub fn duration_ms(&self,a:body::BodyAction)->Result<Vec<u32>> {
        if self.cancel.load(Ordering::Acquire){return Err(invalid());}
        self.snapshot.duration_ms(a).map_err(|_|invalid())
    }
    pub fn render_frame_index(&self,a:body::BodyAction,frame:usize)->Result<body::NativeActorSlice> {
        let ms=self.duration_ms(a)?;if frame>=ms.len(){return Err(invalid());}
        self.render(a,ms[..frame].iter().map(|n|u64::from(*n)).sum())
    }
}
impl ActorRendererJob {
    pub fn native_job(&self)->&native::RendererJob{self.inner.native_job()}
    pub fn identity(&self)->&ActorIdentity{&self.identity}
    pub fn cancellation(&self)->&AtomicBool{self.inner.cancellation()}
    pub fn source_plan(&self)->&r::NativeAssetPlan{self.inner.source_plan()}
    pub fn verify_catalog(&self,root:&Path)->Result<catalog::VerifiedCatalog>{self.inner.verify_catalog(root)}
    pub fn load_body(&self,root:&Path,provenance_root:&Path)->Result<VerifiedActorBody>{self.identity.verify_provenance(provenance_root,self.cancellation())?;let snapshot=body::NativeActorSnapshot::load_authoritative(self.inner.native_job(),root,self.identity.body_pins(),&self.identity.ranges()).map_err(|_|invalid())?;Ok(VerifiedActorBody{snapshot,identity:self.identity.clone(),cancel:self.inner.native_job().cancellation_token()})}
    pub fn finish_partial(self,verified:catalog::VerifiedCatalog,body:VerifiedActorBody,assets:r::NativeStaticAssets,effects:Vec<action::NativeActionSlice>,slice:body::NativeActorSlice)->Result<ActorPartialCompletion>{
        if body.identity!=self.identity||!Arc::ptr_eq(&body.cancel,&self.inner.native_job().cancellation_token())||!slice.matches_pins(&self.identity.body_pins())||!slice.matches_job(self.inner.native_job()){return Err(invalid());}
        Ok(ActorPartialCompletion{inner:self.inner.finish_partial_actor(verified,assets,effects,slice)?,body})
    }
}
#[cfg(test)]
#[path="native_actor_dependency_draft_tests.rs"]
mod tests;
#[cfg(test)]
#[path="native_wire_capture_tests.rs"]
mod capture_tests;
