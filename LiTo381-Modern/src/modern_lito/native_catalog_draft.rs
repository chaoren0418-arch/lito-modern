//! Fixed Native catalog90/91 local consumer. Default disabled, crate private.
//! Does not extend negotiated KNOWN, live dispatch, transport or LOADED.
use super::{application_protocol as app,battle_world as world,native_canonical_draft as native,
    native_resources as resources,native_action_inspection as action,protocol,battle_follow};
use sha2::{Digest,Sha256};
use std::{io::Read,path::Path,sync::{Arc,atomic::{AtomicBool,Ordering}}};
type Result<T>=std::result::Result<T,app::ApplicationError>;
pub const CATALOG_BIT:u32=512;
pub const REQUEST_BYTES:usize=264;
pub const IDENTITY_BYTES:usize=168;
fn invalid()->app::ApplicationError{app::error(app::ErrorKind::MalformedFrame,"invalid local Native catalog draft")}
fn u16_at(b:&[u8],a:usize)->u16{u16::from_be_bytes(b[a..a+2].try_into().unwrap())}
fn u32_at(b:&[u8],a:usize)->u32{u32::from_be_bytes(b[a..a+4].try_into().unwrap())}
fn u64_at(b:&[u8],a:usize)->u64{u64::from_be_bytes(b[a..a+8].try_into().unwrap())}
fn header(b:&[u8],kind:u16,min:usize,max:usize,id:Option<u32>)->Result<usize>{
    if b.len()<16{return Err(invalid());}let n=u32_at(b,4) as usize;
    if &b[..4]!=b"LTM1" || u16_at(b,12)!=kind || u16_at(b,14)!=0 || u32_at(b,8)==0
        || id.is_some_and(|v|v!=u32_at(b,8)) || !(min..=max).contains(&n){return Err(invalid());}Ok(n)
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct CatalogIdentity {pub index:resources::BytePin,pub pak:resources::BytePin,
    pub offset:u64,pub raw_member:resources::BytePin,pub decoded_sha256:[u8;32]}
impl CatalogIdentity {
    pub fn decode(b:&[u8])->Result<Self>{
        if b.len()!=IDENTITY_BYTES || b[..4]!=[0,1,0,3] || b[84..88]!=[0,1,0,0]{return Err(invalid());}
        let s=Self{index:resources::BytePin{bytes:u64_at(b,4),sha256:b[12..44].try_into().unwrap()},
            pak:resources::BytePin{bytes:u64_at(b,44),sha256:b[52..84].try_into().unwrap()},offset:u64_at(b,88),
            raw_member:resources::BytePin{bytes:u64_at(b,96),sha256:b[104..136].try_into().unwrap()},decoded_sha256:b[136..168].try_into().unwrap()};
        s.validate()?;Ok(s)
    }
    fn validate(&self)->Result<()> {
        if !(32..=8*1024*1024).contains(&self.index.bytes) || !(1..=256*1024*1024).contains(&self.pak.bytes)
            || !(1..=1024*1024).contains(&self.raw_member.bytes)
            || self.offset.checked_add(self.raw_member.bytes).is_none_or(|v|v>self.pak.bytes)
            || [self.index.sha256,self.pak.sha256,self.raw_member.sha256,self.decoded_sha256].contains(&[0;32]) {return Err(invalid());}Ok(())
    }
    pub fn encode(&self)->Result<Vec<u8>>{
        self.validate()?;let mut b=vec![0,1,0,3];b.extend(self.index.bytes.to_be_bytes());b.extend(self.index.sha256);
        b.extend(self.pak.bytes.to_be_bytes());b.extend(self.pak.sha256);b.extend([0,1,0,0]);b.extend(self.offset.to_be_bytes());
        b.extend(self.raw_member.bytes.to_be_bytes());b.extend(self.raw_member.sha256);b.extend(self.decoded_sha256);Ok(b)
    }
    pub fn digest(&self)->Result<[u8;32]>{let mut b=b"LITO-NATIVE-CATALOG-IDENTITY\0".to_vec();b.extend(self.encode()?);Ok(Sha256::digest(b).into())}
    pub fn action_pins(&self)->action::CatalogPins {action::CatalogPins{index:self.index,pak:self.pak,
        decoded_catalog:resources::BytePin{bytes:self.raw_member.bytes,sha256:self.decoded_sha256}}}
}
#[derive(Clone,PartialEq,Eq)]
pub struct Request {pub expected_catalog:[u8;32],pub catalog_load_proof:[u8;32],pub native:native::Request}
impl Request {
    fn validate(&self)->Result<()> {
        let q=&self.native.canonical;let first=q.action==world::Action::Read && q.match_id==0;
        if !first && self.expected_catalog==[0;32] || (q.action==world::Action::Loaded)!=(self.catalog_load_proof!=[0;32]){return Err(invalid());}Ok(())
    }
    /// Authority check is distinct from structural parsing. No proof minting
    /// or render admission: only compare diagnostic load claims to fixed pins.
    pub fn check_authority(&self,catalog:&CatalogIdentity,source:&native::SourceAttachment)->Result<()> {
        self.validate()?;source.check_claim(&self.native,u32::MAX)?;let digest=catalog.digest()?;
        if self.expected_catalog!=[0;32] && self.expected_catalog!=digest{return Err(invalid());}
        if self.native.canonical.action==world::Action::Loaded {
            let v1:[u8;32]=Sha256::digest(native::load_proof_preimage(source,&self.native.canonical)?).into();
            let mut pre=b"LITO-NATIVE-CATALOG-LOAD-PROOF\0".to_vec();pre.extend(digest);pre.extend(v1);
            if self.native.load_proof!=v1 || self.catalog_load_proof!=<[u8;32]>::from(Sha256::digest(pre)){return Err(invalid());}
        }Ok(())
    }
}
#[derive(Clone,Copy,Default)]
pub struct CatalogCodec {inner:Option<native::DraftCodec>}
pub struct Reply {request:Request,catalog:Option<CatalogIdentity>,native:native::Reply}
impl Reply {pub fn request(&self)->&Request{&self.request}pub fn catalog(&self)->Option<CatalogIdentity>{self.catalog}
    pub fn native(&self)->&native::Reply{&self.native}}
impl CatalogCodec {
    pub fn local_experiment(base:protocol::Ack,extensions:u32)->Result<Self>{
        if extensions&(CATALOG_BIT|native::NATIVE_BIT)!=(CATALOG_BIT|native::NATIVE_BIT){return Err(invalid());}
        Ok(Self{inner:Some(native::DraftCodec::local_experiment(base,extensions&!CATALOG_BIT)?)})
    }
    fn inner(self)->Result<native::DraftCodec>{self.inner.ok_or_else(invalid)}
    pub fn maximum_result(self)->Result<usize>{Ok(260+self.inner()?.maximum_result()?)}
    pub fn decode_request(self,b:&[u8])->Result<Request>{
        self.inner()?;header(b,90,REQUEST_BYTES,REQUEST_BYTES,None)?;
        if b.len()!=REQUEST_BYTES || b[16..20]!=[0,1,0,2] || u32_at(b,8)!=u32_at(b,92){return Err(invalid());}
        let q=Request{expected_catalog:b[20..52].try_into().unwrap(),catalog_load_proof:b[52..84].try_into().unwrap(),native:self.inner()?.decode_request(&b[84..])?};q.validate()?;Ok(q)
    }
    pub fn encode_request(self,id:u32,q:&Request)->Result<Vec<u8>> {
        q.validate()?;let inner=self.inner()?.encode_request(id,&q.native)?;
        let mut b=app::header(id,90,REQUEST_BYTES)?;b.extend([0,1,0,2]);b.extend(q.expected_catalog);b.extend(q.catalog_load_proof);b.extend(inner);Ok(b)
    }
    pub fn read_reply(self,r:&mut impl Read,expected:&[u8])->Result<Reply> {
        let q=self.decode_request(expected)?;let mut h=[0;16];r.read_exact(&mut h).map_err(|_|invalid())?;
        let n=header(&h,91,336,self.maximum_result()?,Some(u32_at(expected,8)))?;
        let mut b=Vec::new();b.try_reserve_exact(n).map_err(|_|invalid())?;b.extend(h);b.resize(n,0);
        r.read_exact(&mut b[16..]).map_err(|_|invalid())?;
        if b[16..84]!=expected[16..84] || b[88..92]!=[0;4]{return Err(invalid());}
        let c=u32_at(&b,84) as usize;
        if !matches!(c,0|IDENTITY_BYTES) || n<92+c+native::MIN_RESULT_BYTES{return Err(invalid());}
        let catalog=if c==0{None}else{Some(CatalogIdentity::decode(&b[92..92+c])?)};
        let nested=&b[92+c..];
        let native=self.inner()?.decode_reply(nested,&expected[84..])?;
        if catalog.is_some()!=native.source.is_some(){return Err(invalid());}
        if let (Some(c),Some(s))=(catalog,native.source.as_ref()){q.check_authority(&c,s)?;}
        else if q.expected_catalog!=[0;32]{return Err(invalid());}
        Ok(Reply{request:q,catalog,native})
    }
    pub fn decode_reply(self,b:&[u8],q:&[u8])->Result<Reply>{let mut r=b;let reply=self.read_reply(&mut r,q)?;if !r.is_empty(){return Err(invalid());}Ok(reply)}
}
/// Wrapper cannot be constructed from an old88 Reply. Private reply/worker
/// fields bind catalog authority to the inner opaque owner and full DATA.
pub struct CatalogOwner {inner:native::DraftOwner,current:Option<(native::WorkTicket,CatalogIdentity)>}
pub struct CatalogRendererJob {inner:native::RendererJob,catalog:CatalogIdentity}
pub struct VerifiedCatalog {catalog:CatalogIdentity,cancel:Arc<AtomicBool>}
pub struct CatalogPartialCompletion {inner:native::PartialRendererCompletion,verified:VerifiedCatalog}
impl CatalogOwner {
    pub fn local_experiment(codec:CatalogCodec)->Result<Self>{Ok(Self{inner:native::DraftOwner::local_experiment(codec.inner()?)?,current:None})}
    pub fn retire(&mut self){self.inner.retire();self.current=None;}
    pub fn stage(&mut self,scope:native::Scope,reply:Reply)->Result<native::WorkTicket>{
        let catalog=reply.catalog.ok_or_else(invalid)?;
        if self.current.is_some_and(|(_,c)|c!=catalog){return Err(invalid());}
        let t=self.inner.stage(scope,reply.native)?;self.current=Some((t,catalog));Ok(t)
    }
    pub fn begin_renderer(&self,t:native::WorkTicket,o:&battle_follow::Observation)->Result<CatalogRendererJob>{
        let (_,catalog)=self.current.filter(|(ticket,_)|*ticket==t).ok_or_else(invalid)?;
        Ok(CatalogRendererJob{inner:self.inner.begin_renderer(t,o)?,catalog})
    }
    pub fn complete_partial(&mut self,t:native::WorkTicket,c:CatalogPartialCompletion)->Result<()> {
        let (_,catalog)=self.current.filter(|(ticket,_)|*ticket==t).ok_or_else(invalid)?;
        if c.verified.catalog!=catalog || c.verified.cancel.load(Ordering::Acquire){return Err(invalid());}
        self.inner.complete_partial_renderer(t,c.inner)
    }
    pub fn partial_available(&self)->bool{self.inner.partial_renderer_available()}
    pub fn loaded_admission_available(&self)->bool{false}
}
impl CatalogRendererJob {
    pub fn native_job(&self)->&native::RendererJob{&self.inner}
    pub fn source_plan(&self)->&resources::NativeAssetPlan{self.inner.source_plan()}
    pub fn cancellation(&self)->&AtomicBool{self.inner.cancellation()}
    pub fn catalog(&self)->CatalogIdentity{self.catalog}
    pub fn verify_catalog(&self,root:&Path)->Result<VerifiedCatalog>{
        let c=self.catalog;resources::verify_native_catalog_member(root,c.index,c.pak,c.offset,c.raw_member,c.decoded_sha256,self.cancellation()).map_err(|_|invalid())?;
        Ok(VerifiedCatalog{catalog:c,cancel:self.inner.cancellation_token()})
    }
    pub fn finish_partial(self,verified:VerifiedCatalog,assets:resources::NativeStaticAssets,slices:Vec<action::NativeActionSlice>)->Result<CatalogPartialCompletion>{
        if verified.catalog!=self.catalog || !Arc::ptr_eq(&verified.cancel,&self.inner.cancellation_token())
            || slices.iter().any(|s|s.catalog_pins()!=self.catalog.action_pins()){return Err(invalid());}
        Ok(CatalogPartialCompletion{inner:self.inner.finish_partial(assets,slices)?,verified})
    }
    pub fn finish_partial_actor(self,verified:VerifiedCatalog,assets:resources::NativeStaticAssets,slices:Vec<action::NativeActionSlice>,actor:super::native_actor_inspection::NativeActorSlice)->Result<CatalogPartialCompletion>{
        if verified.catalog!=self.catalog || !Arc::ptr_eq(&verified.cancel,&self.inner.cancellation_token()) || slices.iter().any(|s|s.catalog_pins()!=self.catalog.action_pins()){return Err(invalid());}
        Ok(CatalogPartialCompletion{inner:self.inner.finish_partial_actor(assets,slices,actor)?,verified})
    }
}
#[cfg(test)]
#[path="native_catalog_draft_tests.rs"]
mod tests;
