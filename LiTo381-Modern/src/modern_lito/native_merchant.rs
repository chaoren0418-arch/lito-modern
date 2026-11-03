//! Native rental merchant packets. Order IDs belong to the received list, never the item catalogue.
use anyhow::{ensure,Context,Result};
use serde::Serialize;
use std::collections::BTreeSet;
#[derive(Clone,Debug,PartialEq,Eq,Serialize)]
pub struct Dialog{pub npc:u32,pub html:String,pub parameter:String,pub arguments:Vec<String>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize)]
pub struct NumericInput{pub npc:u32,pub auxiliary:u32,pub current:u32,pub minimum:u32,pub maximum:u32,pub mode:u8,pub prompt:String,pub html:String,pub callback:String,pub arguments:Vec<String>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize)]
pub struct BuyRow{pub id:u32,pub icon:u16,pub price:u32,pub name:String,pub detail:Vec<u8>}
#[derive(Clone,Debug,PartialEq,Eq,Serialize)]
pub struct SellRow{pub id:u32,pub price:u32}
#[derive(Clone,Debug,PartialEq,Eq,Serialize)]
pub enum Listing{Buy{npc:u32,rows:Vec<BuyRow>},Sell{npc:u32,rows:Vec<SellRow>}}
impl Listing{pub fn npc(&self)->u32{match self{Self::Buy{npc,..}|Self::Sell{npc,..}=>*npc}}}
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Packet{Dialog(Dialog),List(Listing)}
/// These argument-free formal messages reject an inventory transaction.
pub fn inventory_refusal(message:Option<&(u16,Vec<String>)>)->Option<&'static str>{
    let (code,args)=message?;if !args.is_empty(){return None;}
    match code{263=>Some("背包空間不足，請減少數量或整理背包。"),82=>Some("負重已達上限，請減少數量或減輕背包重量。"),_=>None}
}
struct Reader<'a>{b:&'a[u8],at:usize}
impl Reader<'_>{
    fn take(&mut self,n:usize)->Result<&[u8]>{let end=self.at.checked_add(n).context("merchant offset")?;let b=self.b.get(self.at..end).context("truncated merchant field")?;self.at=end;Ok(b)}
    fn c(&mut self)->Result<u8>{Ok(self.take(1)?[0])}
    fn h(&mut self)->Result<u16>{Ok(u16::from_le_bytes(self.take(2)?.try_into()?))}
    fn d(&mut self)->Result<u32>{Ok(u32::from_le_bytes(self.take(4)?.try_into()?))}
    fn text(&mut self)->Result<String>{let len=self.b[self.at..].iter().take(257).position(|v|*v==0).context("merchant text exceeds 256-byte budget or lacks NUL")?;
        let s=if len==0{String::new()}else{super::native_login::decode_text(self.take(len)?)?};self.take(1)?;Ok(s)}
}
pub fn parse(b:&[u8])->Result<Packet>{
    ensure!(b.len()<=65533,"merchant frame budget");let mut r=Reader{b,at:0};let op=r.c()?;ensure!([0x2a,0x2b,0x2c].contains(&op),"merchant opcode");let npc=r.d()?;ensure!(npc!=0,"zero merchant identity");
    let packet=if op==0x2a{let html=r.text()?;ensure!(!html.is_empty(),"empty dialog resource");let parameter=r.text()?;let n=r.h()? as usize;ensure!(n<=256,"dialog argument budget");let mut arguments=Vec::with_capacity(n);for _ in 0..n{arguments.push(r.text()?);}Packet::Dialog(Dialog{npc,html,parameter,arguments})}
    else{let n=r.h()? as usize;ensure!(n<=4096,"merchant row budget");let mut ids=BTreeSet::new();
        if op==0x2b{let mut rows=Vec::with_capacity(n);for _ in 0..n{let id=r.d()?;ensure!(ids.insert(id),"duplicate buy order ID");let icon=r.h()?;let price=r.d()?;ensure!(price!=0,"zero buy price");let name=r.text()?;let size=r.c()? as usize;let detail=r.take(size)?.to_vec();rows.push(BuyRow{id,icon,price,name,detail});}Packet::List(Listing::Buy{npc,rows})}
        else{let mut rows=Vec::with_capacity(n);for _ in 0..n{let id=r.d()?;ensure!(id!=0&&ids.insert(id),"invalid sell inventory ID");let price=r.d()?;ensure!(price!=0,"zero sell price");rows.push(SellRow{id,price});}Packet::List(Listing::Sell{npc,rows})}
    };ensure!(r.at==b.len(),"trailing merchant bytes");Ok(packet)
}
pub fn talk(npc:u32)->Result<Vec<u8>>{ensure!(npc!=0,"zero dialog target");let mut b=vec![0x29];b.extend(npc.to_le_bytes());Ok(b)}
pub fn parse_input(b:&[u8])->Result<NumericInput>{
    ensure!(b.len()<=65533,"NPC input frame budget");let mut r=Reader{b,at:0};ensure!(r.c()?==0x5b,"NPC input opcode");
    let npc=r.d()?;let auxiliary=r.d()?;let current=r.d()?;let minimum=r.d()?;let maximum=r.d()?;let mode=r.c()?;
    ensure!(npc!=0&&minimum<=current&&current<=maximum&&maximum<=i32::MAX as u32,"NPC input range");
    let prompt=r.text()?;let html=r.text()?;let callback=r.text()?;ensure!(!html.is_empty()&&html.len()<=31,"NPC input resource");
    let n=r.h()? as usize;ensure!(n<=256,"NPC input argument budget");let mut arguments=Vec::with_capacity(n);for _ in 0..n{arguments.push(r.text()?);}
    ensure!(r.at==b.len(),"trailing NPC input bytes");Ok(NumericInput{npc,auxiliary,current,minimum,maximum,mode,prompt,html,callback,arguments})
}
impl NumericInput{
    pub fn reply(&self,value:u32)->Result<Vec<u8>>{
        ensure!(self.npc!=0&&value>=self.minimum&&value<=self.maximum&&self.mode==0,"NPC quantity outside supported request");
        let mut b=vec![0x51];b.extend(self.npc.to_le_bytes());b.extend(value.to_le_bytes());b.push(self.mode);
        if !self.callback.is_empty(){let callback=super::native_login::encode_text(&self.callback,255)?;b.extend_from_slice(&callback);}
        b.push(0);Ok(b)
    }
}
pub fn action(npc:u32,action:&str)->Result<Vec<u8>>{ensure!(npc!=0&&!action.is_empty()&&action.trim()==action,"invalid rental action");let s=super::native_login::encode_text(action,255)?;let mut b=vec![0x27];b.extend(npc.to_le_bytes());b.extend_from_slice(&s);b.extend([0,0]);Ok(b)}
pub fn transaction(npc:u32,sell:bool,lines:&[(u32,u32)])->Result<Vec<u8>>{
    ensure!(npc!=0&&lines.len()<=255,"merchant transaction envelope");let mut b=vec![0x28];b.extend(npc.to_le_bytes());b.extend([u8::from(sell),lines.len() as u8,0]);let mut ids=BTreeSet::new();
    for &(id,q) in lines{ensure!((!sell||id!=0)&&ids.insert(id)&&q>0&&q<=i32::MAX as u32,"invalid merchant quantity or duplicate row");b.extend(id.to_le_bytes());b.extend(q.to_le_bytes());}Ok(b)
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize)]
pub struct Owner{pub connection:u64,pub generation:u64,pub incarnation:u64,pub control_interruptions:u64,pub inventory:super::native_inventory::Owner}
#[derive(Clone,Debug,Default,Serialize)]
pub struct Session{
    pub owner:Option<Owner>,pub npc:Option<u32>,pub dialog:Option<Dialog>,pub listing:Option<Listing>,pub input:Option<NumericInput>,
    pub revision:u64,pub waiting:bool,pub timed_out:bool,pub submitted:bool,
    #[serde(skip)] deadline:Option<std::time::Instant>,
    #[serde(skip)] expires:Option<std::time::Instant>,
}
impl Session{
    pub fn clear(&mut self){let revision=self.revision.wrapping_add(1);*self=Self{revision,..Self::default()};}
    pub fn begin(&mut self,owner:Owner,npc:u32,now:std::time::Instant)->Result<()>{
        ensure!(!self.waiting,"wait for NPC response");self.clear();self.owner=Some(owner);self.npc=Some(npc);self.waiting=true;self.deadline=Some(now+std::time::Duration::from_secs(8));Ok(())
    }
    pub fn accept(&mut self,p:Packet,now:std::time::Instant)->Result<()>{
        let npc=match &p{Packet::Dialog(d)=>d.npc,Packet::List(l)=>l.npc()};
        ensure!(self.owner.is_some()&&self.npc==Some(npc),"NPC response has no matching interaction");
        self.waiting=false;self.timed_out=false;self.deadline=None;self.input=None;self.revision=self.revision.wrapping_add(1);
        match p{Packet::Dialog(d)=>{self.dialog=Some(d);self.listing=None;self.expires=None;},Packet::List(l)=>{self.listing=Some(l);self.expires=Some(now+std::time::Duration::from_secs(30));}}
        Ok(())
    }
    pub fn accept_input(&mut self,input:NumericInput,now:std::time::Instant)->Result<()>{
        ensure!(self.owner.is_some()&&self.npc==Some(input.npc),"NPC quantity has no matching interaction");
        let dialog=Dialog{npc:input.npc,html:input.html.clone(),parameter:String::new(),arguments:input.arguments.clone()};
        self.accept(Packet::Dialog(dialog),now)?;self.input=Some(input);self.expires=Some(now+std::time::Duration::from_secs(30));Ok(())
    }
    pub fn input_reply(&self,revision:u64,value:u32,now:std::time::Instant)->Result<Vec<u8>>{
        ensure!(!self.waiting&&self.revision==revision&&self.expires.is_some_and(|at|now<at),"NPC quantity request changed or expired");
        self.input.as_ref().context("no current NPC quantity request")?.reply(value)
    }
    pub fn tick(&mut self,owner:Option<&Owner>,now:std::time::Instant){
        if self.owner.as_ref()!=owner {if self.owner.is_some(){self.clear();}return;}
        if self.deadline.is_some_and(|at|now>=at){self.waiting=false;self.timed_out=true;self.deadline=None;}
        if self.expires.is_some_and(|at|now>=at){self.listing=None;self.input=None;self.expires=None;self.revision=self.revision.wrapping_add(1);}
    }
    pub fn order(&self,revision:u64,index:usize,quantity:u32,now:std::time::Instant)->Result<Vec<u8>>{
        ensure!(!self.waiting&&self.revision==revision&&self.expires.is_some_and(|at|now<at),"merchant list changed or expired");
        let list=self.listing.as_ref().context("no active merchant list")?;
        let(sell,id)=match list{Listing::Buy{rows,..}=>(false,rows.get(index).context("buy row index")?.id),Listing::Sell{rows,..}=>(true,rows.get(index).context("sell row index")?.id)};
        transaction(list.npc(),sell,&[(id,quantity)])
    }
    pub fn submitted(&mut self){self.listing=None;self.input=None;self.expires=None;self.submitted=true;self.revision=self.revision.wrapping_add(1);}
}
