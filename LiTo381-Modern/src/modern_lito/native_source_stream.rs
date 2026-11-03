//! Known ordinary Native receive candidate, owned by the existing Runtime.
//! No socket, LTM1 translation, guessed seed, or authentication side effect.
//! 4C62A0 clears8317B8; first plain opcode1/d ->4AE050 seeds both counters.
//! Only the receive half is used here. 480560 login effects remain OPEN.
use anyhow::{ensure,Result};
use super::source_connection::MAX_DECODED_BODY;

pub const MAX_FRAME_BYTES:usize=MAX_DECODED_BODY+2;
pub const MAX_BATCH_EVENTS:usize=8;
pub const SEED_HIGH_WORD:u32=crate::modern_lito::native_wire::KEY_HIGH;

/// Pure Native add/adc arithmetic. It cannot install a Runtime counter.
/// The full-u64 wrap probe is inspection; real initial highword is fixed.
pub fn advance_counter(counter:u64,body_len:usize)->Result<u64>{
    ensure!((1..=MAX_DECODED_BODY).contains(&body_len),"receive body length outside candidate bound");
    Ok(counter.wrapping_add(body_len as u64))
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Terminal { InvalidLength, SeedExpected, SeedLength, RepeatedSeed, UnsupportedBody, DispatchRefused, CleanEof, HeaderEof, BodyEof }
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Snapshot { pub counter:Option<u64>,pub header_bytes:usize,pub body_bytes:usize,pub body_expected:Option<usize> }
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct End { pub terminal:Terminal,pub before:Snapshot }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct CounterStep { pub before:u64,pub after:u64 }
pub(crate) enum Frame { Seed{counter:u64},Body{bytes:Vec<u8>,step:CounterStep} }
pub(crate) struct Receiver {
    counter:Option<u64>,header:[u8;2],header_len:usize,
    body:Box<[u8]>,body_len:usize,expected:Option<usize>,
}
impl Default for Receiver { fn default()->Self{Self{counter:None,header:[0;2],header_len:0,body:vec![0;MAX_DECODED_BODY].into_boxed_slice(),body_len:0,expected:None}} }
impl Receiver {
    pub(crate) fn snapshot(&self)->Snapshot{Snapshot{counter:self.counter,header_bytes:self.header_len,body_bytes:self.body_len,body_expected:self.expected}}
    pub(crate) fn end(&self)->End{End{terminal:if self.header_len==0{Terminal::CleanEof}else if self.expected.is_none(){Terminal::HeaderEof}else{Terminal::BodyEof},before:self.snapshot()}}
    /// Consumes at most one frame. Caller delivers it synchronously through
    /// the existing epoch decoder/Runtime, then resumes the unconsumed tail.
    pub(crate) fn next(&mut self,input:&[u8])->(usize,std::result::Result<Option<Frame>,Terminal>){
        let mut at=0;
        if self.expected.is_none(){
            let n=(2-self.header_len).min(input.len());self.header[self.header_len..self.header_len+n].copy_from_slice(&input[..n]);self.header_len+=n;at+=n;
            if self.header_len<2{return(at,Ok(None));}
            let total=usize::from(u16::from_le_bytes(self.header));
            if !(3..=MAX_FRAME_BYTES).contains(&total){return(at,Err(Terminal::InvalidLength));}
            self.expected=Some(total-2);
        }
        let expected=self.expected.unwrap();let n=(expected-self.body_len).min(input.len()-at);
        self.body[self.body_len..self.body_len+n].copy_from_slice(&input[at..at+n]);self.body_len+=n;at+=n;
        if self.body_len<expected{return(at,Ok(None));}
        let frame=if let Some(counter)=self.counter{
            let mut bytes=self.body[..expected].to_vec();
            crate::modern_lito::native_wire::decrypt_in_place(counter,&mut bytes);
            // Source4B37C0 add/adc precedes4B2970 dispatch, even when the
            // downstream candidate body decoder will subsequently refuse it.
            let after=advance_counter(counter,expected).expect("bounded received body");self.counter=Some(after);
            Frame::Body{bytes,step:CounterStep{before:counter,after}}
        }else{
            if self.body[0]!=1{return(at,Err(Terminal::SeedExpected));}
            if expected!=5{return(at,Err(Terminal::SeedLength));}
            let low=u32::from_le_bytes(self.body[1..5].try_into().unwrap());let counter=(u64::from(SEED_HIGH_WORD)<<32)|u64::from(low);
            self.counter=Some(counter);Frame::Seed{counter}
        };
        self.header_len=0;self.body_len=0;self.expected=None;
        (at,Ok(Some(frame)))
    }
}
