from r491_common import *
p=P/'src/modern_lito/native_java_room_settings_request.rs';s=p.read_text('utf-8')
s=s.replace('//! Ordinary compact C_DD ASCII candidate writer. No Native codec/UI parity,\n//! extended/mode/map fields, optimistic state or server edit ACK is inferred.','//! Version-pinned compact AD368 / compact+extended485F ASCII C_DD writers.\n//! No Native codec/UI parity, optimistic state, mode/map or server ACK.')
s=s.replace('#[derive(Clone)]pub struct Adapter;\nimpl Adapter {pub fn parse(raw:&[u8])->Result<Self>{ensure!(raw.len()<=1024,"edit writer adapter bound");let v:serde_json::Value=serde_json::from_slice(raw)?;ensure!(v["kind"]=="modern_synthetic_AD368_compact_settings_request_v1"&&v["producer_sha256"]==super::login_bootstrap::PRODUCER_SHA&&v["edit_sha256"]==EDIT_SHA&&v["native_sha256"]==NATIVE_SHA,"pinned compact settings writer required");Ok(Self)}}', '''pub const SETTINGS_485F_SHA:&str="485F599DA9BA1A13504CB48DDCA3D53C6CAD91D7BAFDF738D104E65A05E1DFD5";
pub const SETTINGS_485F_NETWORK_SHA:&str="8C1776AA503098E09B42EFE8CA45E07E651DD4570A07CB6A8994006421CCEF41";
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Producer {AD368,Settings485F}
impl Producer {pub fn sha256(self)->&'static str{match self{Self::AD368=>super::login_bootstrap::PRODUCER_SHA,Self::Settings485F=>SETTINGS_485F_SHA}}}
#[derive(Clone)]pub struct Adapter {producer:Producer}
impl Adapter {pub fn producer(&self)->Producer{self.producer}pub fn parse(raw:&[u8])->Result<Self>{ensure!(raw.len()<=1024,"edit writer adapter bound");let v:serde_json::Value=serde_json::from_slice(raw)?;ensure!(v["native_sha256"]==NATIVE_SHA,"pinned C_DD static boundary required");let producer=if v["kind"]=="modern_synthetic_AD368_compact_settings_request_v1"&&v["producer_sha256"]==super::login_bootstrap::PRODUCER_SHA&&v["edit_sha256"]==EDIT_SHA{Producer::AD368}else{ensure!(v["kind"]=="modern_synthetic_485F_settings_request_v1"&&v["producer_sha256"]==SETTINGS_485F_SHA&&v["network_sha256"]==SETTINGS_485F_NETWORK_SHA,"pinned485F extended settings writer required");Producer::Settings485F};Ok(Self{producer})}}''')
needle='#[derive(Clone,Debug,PartialEq,Eq)]pub struct ContextKey'
assert needle in s
s=s.replace(needle,'''/// Edit-tail values do not grant Create/Join/source/membership authority.
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Extension {pub values:[u8;4],pub selections:Vec<u8>}
pub fn encode_extended(input:&Inputs,tail:&Extension)->Result<Vec<u8>>{let mut b=encode(input)?;ensure!(tail.selections.len()<=8&&!tail.selections.contains(&0),"extended K0..8/nonzero selection bytes required");ensure!((tail.values==[0;4]&&tail.selections.is_empty())||(tail.values[0]<=7&&tail.values[1]<=2&&(1..=127).contains(&tail.values[2])&&tail.values[3]<=31),"extended options outside finite producer domain");b.extend(tail.values);b.push(tail.selections.len()as u8);b.extend(&tail.selections);Ok(b)}
''' +needle)
s=s.replace('pub input:Inputs,pub context:ContextKey','pub input:Inputs,pub extension:Option<Extension>,pub producer:Producer,pub context:ContextKey')
s=s.replace('context:ContextKey,input:Inputs}','context:ContextKey,input:Inputs,extension:Option<Extension>,producer:Producer}')
s=s.replace('self.java_settings_request.adapter.is_some()&&self.java_settings.adapter.is_some()&&self.page', 'self.java_settings_request.adapter.as_ref().zip(self.java_settings.adapter.as_ref()).is_some_and(|(writer,receiver)|writer.producer.sha256()==receiver.producer_sha256())&&self.page')
start=s.index(' pub fn java_room_settings_request_control(');end=s.index(' pub fn handle_java_room_settings_request_control',start)
s=s[:start]+''' pub fn java_room_settings_request_control(&self,input:Inputs)->Option<Control>{self.settings_request_control(input,None)}
 pub fn java_room_extended_settings_request_control(&self,input:Inputs,tail:Extension)->Option<Control>{self.settings_request_control(input,Some(tail))}
 fn settings_request_control(&self,input:Inputs,extension:Option<Extension>)->Option<Control>{let producer=self.java_settings_request.adapter.as_ref()?.producer;if extension.is_some()&&producer!=Producer::Settings485F{return None;}if self.source_send.snapshot().pending.is_some()||self.java_settings_request.requests.len()>=MAX_REQUESTS||(if let Some(t)=&extension{encode_extended(&input,t)}else{encode(&input)}).is_err(){return None;}Some(Control{epoch:self.source_scope.current_epoch().ok()?,revision:self.java_settings_request.revision,context:self.current_settings_request_context().ok()?,input,extension,producer})}
''' +s[end:]
s=s.replace('ensure!(c.revision==self.java_settings_request.revision&&c.context==self.current_settings_request_context()?','ensure!(self.java_settings_request.adapter.as_ref().is_some_and(|a|a.producer==c.producer)&&(c.extension.is_none()||c.producer==Producer::Settings485F)&&c.revision==self.java_settings_request.revision&&c.context==self.current_settings_request_context()?')
s=s.replace('let body=encode(&c.input)?;let prepared=self.source_send.prepare_ordinary_room_settings(&c.input)?;', 'let(body,prepared)=if let Some(tail)=&c.extension{(encode_extended(&c.input,tail)?,self.source_send.prepare_extended_ordinary_room_settings(&c.input,tail)?)}else{(encode(&c.input)?,self.source_send.prepare_ordinary_room_settings(&c.input)?)};')
s=s.replace('input:c.input.clone(),context:c.context.clone()', 'input:c.input.clone(),extension:c.extension.clone(),producer:c.producer,context:c.context.clone()')
s=s.replace('write_serial:serial,input:i,context:', 'write_serial:serial,input:i,extension:None,producer:Producer::AD368,context:')
assert 'Some(Self)' not in s and 'let(body,prepared)' in s and 'receiver.producer_sha256()' in s
p.write_text(s,'utf-8')
p=P/'src/modern_lito/native_java_room_settings.rs';s=p.read_text('utf-8');s=s.replace('#[derive(Clone)]pub struct Adapter;','#[derive(Clone)]pub struct Adapter {producer:&\'static str}')
start=s.index('impl Adapter {');end=s.index('/// Mode is a structural observation;',start)
s=s[:start]+'''impl Adapter {
 pub fn producer_sha256(&self)->&'static str{self.producer}
 pub fn parse(bytes:&[u8])->Result<Self>{ensure!(bytes.len()<=1024,"settings adapter exceeds bound");let v:serde_json::Value=serde_json::from_slice(bytes)?;ensure!(v["layout_sha256"]==LAYOUT_SHA&&v["native_sha256"]==NATIVE_SHA,"pinned independent settings layout required");let producer=if v["kind"]=="modern_synthetic_AD368_room_settings_v1"&&v["producer_sha256"]==super::login_bootstrap::PRODUCER_SHA&&v["edit_sha256"]==EDIT_SHA{super::login_bootstrap::PRODUCER_SHA}else{ensure!(v["kind"]=="modern_synthetic_485F_room_settings_v1"&&v["producer_sha256"]==super::java_room_settings_request::SETTINGS_485F_SHA&&v["network_sha256"]==super::java_room_settings_request::SETTINGS_485F_NETWORK_SHA,"explicit pinned485F independent0/1 adapter required");super::java_room_settings_request::SETTINGS_485F_SHA};Ok(Self{producer})}
}
''' +s[end:];p.write_text(s,'utf-8')
p=P/'src/modern_lito/native_source_send.rs';s=p.read_text('utf-8');needle='    pub(crate) fn prepare_ordinary_room_settings(';at=s.index(needle)
s=s[:at]+'''    pub(crate) fn prepare_extended_ordinary_room_settings(&self,input:&super::java_room_settings_request::Inputs,tail:&super::java_room_settings_request::Extension)->Result<Prepared> {
        self.prepare_body(super::java_room_settings_request::encode_extended(input,tail)?)
    }
''' +s[at:];p.write_text(s,'utf-8')
print('R524 integration:explicit485F writer+0/1 adapters, tail extends same opaque/context/once/Sender pipeline; AD368 compact remains separate')
