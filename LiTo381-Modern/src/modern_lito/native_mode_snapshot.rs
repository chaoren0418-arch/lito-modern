//! Complete bounded layouts for the two source-proven subtype-zero handlers.
//! Format/varargs pairing proves mode is `c` in BOTH layouts, not `h`.
//! Strings remain raw bytes; source encoding is unknown. Strict capacity,
//! packet-end and extension limits are Modern inspection policies.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Kind {State206,State213}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Context {pub phase:u32,pub dispatch_word:u32}
#[derive(Clone,Debug,PartialEq,Eq)]pub enum Value {Byte(u8),Word(u16),Dword(u32),Bytes(Vec<u8>)}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Field {pub destination:&'static str,pub offset:usize,pub value:Value}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Snapshot {kind:Kind,identifier:u32,mode_byte:u8,fields:Vec<Field>,raw:Vec<u8>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum SnapshotError {
 PacketLimit{actual:usize},Truncated{field:&'static str,offset:usize,needed:usize},
 Opcode{actual:u8},Subtype{actual:u8},MissingNul{field:&'static str,offset:usize,capacity:usize},
 StringLimit{field:&'static str,offset:usize,capacity:usize},ExtensionCount{actual:u8},
 Trailing{offset:usize,actual:usize},DispatchUnavailable{phase:u32,dispatch_word:u32}
}
impl std::fmt::Display for SnapshotError {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{write!(f,"native state inspection refused: {self:?}")}}
impl std::error::Error for SnapshotError {}
/// 27 minimum bytes +89/4/27 string bytes +8 extension bytes.
pub const MAX_PACKET_BYTES:usize=155;
struct Cursor<'a>{packet:&'a[u8],pos:usize,fields:Vec<Field>}
impl Cursor<'_>{
 fn take(&mut self,n:usize,field:&'static str)->Result<&[u8],SnapshotError>{let start=self.pos;let end=start.checked_add(n).ok_or(SnapshotError::Truncated{field,offset:start,needed:n})?;let bytes=self.packet.get(start..end).ok_or(SnapshotError::Truncated{field,offset:start,needed:n})?;self.pos=end;Ok(bytes)}
 fn byte(&mut self,name:&'static str)->Result<u8,SnapshotError>{let offset=self.pos;let v=self.take(1,name)?[0];self.fields.push(Field{destination:name,offset,value:Value::Byte(v)});Ok(v)}
 fn word(&mut self,name:&'static str)->Result<u16,SnapshotError>{let offset=self.pos;let b=self.take(2,name)?;let v=u16::from_le_bytes([b[0],b[1]]);self.fields.push(Field{destination:name,offset,value:Value::Word(v)});Ok(v)}
 fn dword(&mut self,name:&'static str)->Result<u32,SnapshotError>{let offset=self.pos;let b=self.take(4,name)?;let v=u32::from_le_bytes([b[0],b[1],b[2],b[3]]);self.fields.push(Field{destination:name,offset,value:Value::Dword(v)});Ok(v)}
 fn string(&mut self,name:&'static str,capacity:usize)->Result<(),SnapshotError>{let offset=self.pos;let remaining=&self.packet[offset..];let bound=remaining.len().min(capacity);let Some(n)=remaining[..bound].iter().position(|b|*b==0)else{return Err(if remaining.len()>=capacity{SnapshotError::StringLimit{field:name,offset,capacity}}else{SnapshotError::MissingNul{field:name,offset,capacity}});};let bytes=self.take(n+1,name)?[..n].to_vec();self.fields.push(Field{destination:name,offset,value:Value::Bytes(bytes)});Ok(())}
}
impl Snapshot {
 pub fn kind(&self)->Kind{self.kind}pub fn identifier(&self)->u32{self.identifier}pub fn mode_byte(&self)->u8{self.mode_byte}
 pub fn fields(&self)->&[Field]{&self.fields}pub fn raw(&self)->&[u8]{&self.raw}
 pub fn decode(packet:&[u8])->Result<Self,SnapshotError>{
  if packet.len()>MAX_PACKET_BYTES{return Err(SnapshotError::PacketLimit{actual:packet.len()});}
  let mut c=Cursor{packet,pos:0,fields:Vec::new()};let opcode=c.take(1,"opcode")?[0];
  let kind=match opcode{206=>Kind::State206,213=>Kind::State213,_=>return Err(SnapshotError::Opcode{actual:opcode})};
  let subtype=c.byte("subtype")?;if subtype!=0{return Err(SnapshotError::Subtype{actual:subtype});}
  let identifier=c.dword("dword_64ABF4")?;
  c.string("dword_64AB70",90)?;
  let mode_byte=match kind {
   Kind::State206=>{
    c.byte("byte_64ABE6")?;c.string("byte_64ABE8",5)?;
    c.word("word_64ABEE")?;c.word("dword_64ABF0_low16")?;
    let mode=c.byte("word_64ACCA_low8")?;c.word("var_4")?;
    c.byte("dword_64ACCC_low8")?;c.byte("byte_64ACD0")?;c.byte("byte_64ACD3")?;c.byte("byte_64ACD1")?;c.byte("byte_64ACD2")?;
    c.string("word_64ABCA",28)?;
    c.byte("byte_64ACD4")?;c.byte("byte_64ACD5")?;c.byte("byte_64ACD6")?;c.byte("byte_64ACD7")?;
    let count=c.byte("var_1_extension_count")?;
    // The source copies at most eight bytes using a signed count. Structures
    // with negative or >8 count are deliberately unsupported, not guessed.
    if count>8{return Err(SnapshotError::ExtensionCount{actual:count});}
    let offset=c.pos;let bytes=c.take(usize::from(count),"byte_64ACD8")?.to_vec();c.fields.push(Field{destination:"byte_64ACD8",offset,value:Value::Bytes(bytes)});mode
   },
   Kind::State213=>{
    let mode=c.byte("word_64ACCA_low8")?;c.word("var_4")?;c.byte("dword_64ACCC_low8")?;
    c.string("word_64ABCA",28)?;c.byte("byte_64ACD0")?;c.byte("byte_64ACD1")?;c.byte("byte_64ACD2")?;c.byte("byte_64ACD3")?;mode
   }
  };
  if c.pos!=packet.len(){return Err(SnapshotError::Trailing{offset:c.pos,actual:packet.len()});}
  Ok(Self{kind,identifier,mode_byte,fields:c.fields,raw:packet.to_vec()})
 }
}
