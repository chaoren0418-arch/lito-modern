//! S_FB separates direct rooms from VS endpoints. Counts and wire words are u32.
use anyhow::{ensure,Context,Result};
use std::net::{Ipv4Addr,SocketAddr};
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]
pub struct DirectRoom {pub name:String,pub endpoint:SocketAddr,pub id:u32,pub users:u32}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]
pub struct VsServer {pub name:String,pub endpoint:SocketAddr,pub users:u32,pub capacity:u32}
#[derive(Clone,Debug,PartialEq,Eq,Default,serde::Serialize)]
pub struct Directory {pub rooms:Vec<DirectRoom>,pub servers:Vec<VsServer>}
/// S_F3 belongs to the currently advertised VS directory/page. An omitted slot
/// is unavailable; the packet itself does not authorize an endpoint handoff.
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]
pub struct Availability {pub slots:Vec<bool>}
impl Availability {
    pub fn available(&self,index:u8)->bool {self.slots.get(usize::from(index)).copied().unwrap_or(false)}
}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]
pub struct VsEndpoint {pub index:u8,pub endpoint:SocketAddr}
pub fn parse_availability(b:&[u8])->Result<Availability>{
    ensure!(b.len()>=2&&b[0]==0xf3,"VS availability envelope");
    let count=usize::from(b[1]);
    ensure!(count<=10&&b.len()==2+count*4,"VS availability count/length");
    let mut slots=Vec::with_capacity(count);
    for bytes in b[2..].chunks_exact(4){
        let word=u32::from_le_bytes(bytes.try_into()?);
        ensure!(word==0||word==u32::MAX,"unknown VS availability word");
        slots.push(word==0);
    }
    Ok(Availability{slots})
}
pub fn parse_endpoint(b:&[u8])->Result<VsEndpoint>{
    ensure!(b.len()==8&&b[0]==0xf4,"VS endpoint envelope");
    ensure!(b[1]<10,"VS endpoint index");
    let ip=Ipv4Addr::new(b[2],b[3],b[4],b[5]);
    let port=u16::from_le_bytes([b[6],b[7]]);ensure!(port!=0,"zero VS endpoint port");
    Ok(VsEndpoint{index:b[1],endpoint:(ip,port).into()})
}
struct Reader<'a>{bytes:&'a[u8],at:usize}
impl Reader<'_>{
    fn take(&mut self,n:usize)->Result<&[u8]>{let end=self.at.checked_add(n).context("service offset")?;let b=self.bytes.get(self.at..end).context("truncated service field")?;self.at=end;Ok(b)}
    fn d(&mut self)->Result<u32>{Ok(u32::from_le_bytes(self.take(4)?.try_into()?))}
    fn endpoint(&mut self)->Result<(String,SocketAddr)>{
        let len=self.bytes[self.at..].iter().take(257).position(|x|*x==0).context("service name lacks bounded NUL")?;
        let name=super::native_login::decode_text(self.take(len)?)?;self.take(1)?;
        let ip=Ipv4Addr::from(self.d()?.to_le_bytes());let port=u16::from_le_bytes(self.take(2)?.try_into()?);ensure!(port!=0,"zero service port");Ok((name,(ip,port).into()))
    }
}
pub fn parse(b:&[u8])->Result<Directory>{
    ensure!(b.first()==Some(&0xfb),"service opcode");let mut r=Reader{bytes:b,at:1};let rooms=r.d()?;let servers=r.d()?;
    ensure!(rooms<=127&&servers<=10,"service directory counts");
    let mut d=Directory::default();
    for _ in 0..rooms {let(name,endpoint)=r.endpoint()?;let id=r.d()?;let users=r.d()?;ensure!(id!=0,"zero service room");d.rooms.push(DirectRoom{name,endpoint,id,users});}
    for _ in 0..servers {let(name,endpoint)=r.endpoint()?;let users=r.d()?;let capacity=r.d()?;ensure!(capacity!=0,"zero service capacity");d.servers.push(VsServer{name,endpoint,users,capacity});}
    ensure!(r.at==b.len(),"trailing service directory bytes");Ok(d)
}
