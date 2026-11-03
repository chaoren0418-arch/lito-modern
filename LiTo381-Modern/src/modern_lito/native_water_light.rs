//! Pure Rust port of the original water color tables and raster sampling.
//! The caller owns source selection, GAME lifetime, phase and camera binding.
use super::{native_light_color::Format,native_light_mask::Mask};
use anyhow::{ensure,Context,Result};
use sha2::{Digest,Sha256};
use std::{io::Read,path::Path,sync::atomic::{AtomicBool,Ordering}};
pub const SHADE_WIDTH:usize=640;
pub const SHADE_HEIGHT:usize=420;
pub const SHADE_SHA256:&str="cb4f1aca4e5272c68830b106db1508292807233c199ca062ab53e3d2e709b52d";
const RESOURCE_LIMIT:usize=1<<20;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Params {pub phase:i8,pub scroll:[u16;2]}
impl Params {
    fn validate(self)->Result<()> {
        ensure!((-21..=21).contains(&self.phase) && self.scroll[0]<640 && self.scroll[1]<420,
            "water phase/scroll outside retained kernel domain");
        Ok(())
    }
}
pub struct Shade {values:Box<[u8]>}
impl Shade {
    pub fn from_clamped(values:&[u8])->Result<Self> {
        ensure!(values.len()==SHADE_WIDTH*SHADE_HEIGHT && values.iter().all(|v|*v<=9),"water shade plane extent/domain");
        Ok(Self{values:values.into()})
    }
    /// Original fopen("r")/%c reads Windows text mode. Preserve standalone CR,
    /// collapse CRLF, then apply the signed-byte upper clamp from the loader.
    pub fn parse_resource(bytes:&[u8])->Result<Self> {
        ensure!(bytes.len()<=RESOURCE_LIMIT && format!("{:x}",Sha256::digest(bytes))==SHADE_SHA256,
            "shade.dat differs from qualified original");
        let mut values=Vec::with_capacity(SHADE_WIDTH*SHADE_HEIGHT);let mut at=0;
        while at<bytes.len() {
            if bytes[at]==b'\r' && bytes.get(at+1)==Some(&b'\n'){at+=1;}
            let value=bytes[at] as i8;
            ensure!(value>=0 && bytes[at]!=26,"unsupported shade text-mode byte");
            values.push(value.min(9) as u8);at+=1;
        }
        Self::from_clamped(&values)
    }
    pub fn load(root:&Path,cancel:&AtomicBool)->Result<Self> {
        ensure!(!cancel.load(Ordering::Acquire),"water shade load cancelled");
        let file=std::fs::File::open(root.join("shade.dat"))?;
        let mut bytes=Vec::new();file.take((RESOURCE_LIMIT+1) as u64).read_to_end(&mut bytes)?;
        ensure!(!cancel.load(Ordering::Acquire),"water shade load cancelled");
        Self::parse_resource(&bytes)
    }
    pub fn values(&self)->&[u8] {&self.values}
    fn at(&self,params:Params,x:usize,y:usize)->u8 {
        let start_x=(1280-usize::from(params.scroll[0])-usize::from(params.phase.unsigned_abs()))%640;
        let start_y=(816-usize::from(params.scroll[1]))%408;
        let wx=(start_x+(x%640)/2*2)%640;
        // Native code takes remainder BEFORE incrementing. Later rows use
        // 1..408, unlike the first row's 0..407. Row408 is intentional here.
        let wy=if y==0 {start_y}else{(start_y+(y-1)%408)%408+1};
        self.values[(wx/2)*420+wy]
    }
    pub fn sample(&self,params:Params,x:usize,y:usize)->Result<u8> {
        params.validate()?;Ok(self.at(params,x,y))
    }
}

pub struct Channels {format:Format,values:Box<[u16]>}
impl Channels {
    pub fn new(format:Format)->Self {
        let mut values=vec![0;3*10240].into_boxed_slice();
        let shifts=if format==Format::Rgb555 {[0,5,10]}else{[0,6,11]};
        for (channel,shift) in shifts.into_iter().enumerate() {
            let input=channel*32;
            let response=if channel==1 {416}else{96};
            for shade in 0..10 {for color in 0..32 {for level in 0..32 {
                let adjusted=usize::from(TABLES[response+shade*32+usize::from(TABLES[input+color])]);
                values[channel*10240+shade*1024+color*32+level]=((adjusted*level/31) as u16)<<shift;
            }}}
        }
        Self{format,values}
    }
    pub fn native_words(&self)->&[u16] {&self.values}
    fn pixel(&self,color:u16,shade:u8,level:u8)->u16 {
        let (green,red)=if self.format==Format::Rgb555 {(5,10)}else{(6,11)};
        let at=usize::from(shade)*1024+usize::from(level);
        // Water has no remap31 identity bypass; its tint still changes colors.
        self.values[at+usize::from(color&31)*32]
            |self.values[10240+at+usize::from((color>>green)&31)*32]
            |self.values[20480+at+usize::from((color>>red)&31)*32]
    }
    pub fn apply_words(&self,pixels:&mut[u16],pitch:usize,origin:[usize;2],size:[usize;2],
        mask:&Mask,row:&[u8;32],shade:&Shade,params:Params)->Result<Receipt> {
        validate(size,mask,row,params)?;
        let right=origin[0].checked_add(size[0]).context("water viewport X overflow")?;
        ensure!(pitch>=right,"water surface pitch");
        let bottom=origin[1].checked_add(size[1]-1).context("water viewport Y overflow")?;
        let end=bottom.checked_mul(pitch).and_then(|v|v.checked_add(right)).context("water backing overflow")?;
        ensure!(end<=pixels.len(),"water backing extent");
        for y in 0..size[1] {for x in 0..size[0] {
            let level=row[usize::from(mask.pixels()[(y/2)*mask.width()+x/2]>>3)];
            let at=(origin[1]+y)*pitch+origin[0]+x;
            pixels[at]=self.pixel(pixels[at],shade.at(params,x,y),level);
        }}
        Ok(Receipt{transformed:size[0]*size[1]})
    }
    pub fn apply_rgba555(&self,pixels:&mut[u8],size:[usize;2],mask:&Mask,row:&[u8;32],
        shade:&Shade,params:Params)->Result<Receipt> {
        validate(size,mask,row,params)?;
        ensure!(self.format==Format::Rgb555,"RGBA water surface requires RGB555 tables");
        ensure!(pixels.len()==size[0]*size[1]*4,"water RGBA extent");
        let expand=|v:u16|{let v=(v&31) as u8;(v<<3)|(v>>2)};
        for y in 0..size[1] {for x in 0..size[0] {
            let level=row[usize::from(mask.pixels()[(y/2)*mask.width()+x/2]>>3)];
            let at=(y*size[0]+x)*4;
            let color=(u16::from(pixels[at]>>3)<<10)|(u16::from(pixels[at+1]>>3)<<5)|u16::from(pixels[at+2]>>3);
            let value=self.pixel(color,shade.at(params,x,y),level);
            pixels[at]=expand(value>>10);pixels[at+1]=expand(value>>5);pixels[at+2]=expand(value);
        }}
        Ok(Receipt{transformed:size[0]*size[1]})
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize)]
pub struct Receipt {pub transformed:usize}
fn validate(size:[usize;2],mask:&Mask,row:&[u8;32],params:Params)->Result<()> {
    params.validate()?;
    ensure!(size.iter().all(|v|(1..=4096).contains(v)),"water viewport budget");
    ensure!(mask.width()==size[0].div_ceil(2) && mask.height()==size[1].div_ceil(2),"water mask extent");
    ensure!(row.iter().all(|v|*v<=31),"water remap domain");Ok(())
}

// Pinned original static lookup data is appended by fixture preparation.

const TABLES: [u16; 736] = [
    0, 0, 1, 1, 3, 3, 4, 4, 7, 7, 8, 8, 10, 10, 11, 11,
    14, 14, 15, 15, 17, 17, 18, 18, 21, 21, 22, 22, 24, 24, 25, 25,
    0, 0, 1, 2, 3, 3, 4, 5, 6, 6, 7, 8, 9, 9, 10, 11,
    12, 12, 13, 14, 15, 15, 16, 17, 18, 18, 19, 20, 21, 21, 22, 23,
    0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7,
    0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7,
    1, 1, 3, 3, 4, 4, 6, 6, 8, 8, 9, 9, 11, 11, 12, 12,
    14, 14, 16, 16, 17, 17, 19, 19, 21, 21, 22, 22, 24, 24, 25, 25,
    1, 1, 3, 3, 4, 4, 6, 6, 8, 8, 9, 9, 11, 11, 13, 13,
    14, 14, 16, 16, 17, 17, 19, 19, 21, 21, 22, 22, 24, 24, 26, 26,
    2, 2, 3, 3, 5, 5, 6, 6, 8, 8, 10, 10, 11, 11, 13, 13,
    15, 15, 16, 16, 18, 18, 19, 19, 21, 21, 23, 23, 24, 24, 26, 26,
    2, 2, 3, 3, 5, 5, 7, 7, 8, 8, 10, 10, 11, 11, 13, 13,
    15, 15, 16, 16, 18, 18, 20, 20, 21, 21, 23, 23, 24, 24, 26, 26,
    2, 2, 4, 4, 5, 5, 7, 7, 9, 9, 10, 10, 12, 12, 13, 13,
    15, 15, 17, 17, 18, 18, 20, 20, 22, 22, 23, 23, 25, 25, 26, 26,
    2, 2, 4, 4, 5, 5, 7, 7, 9, 9, 10, 10, 12, 12, 14, 14,
    15, 15, 17, 17, 18, 18, 20, 20, 22, 22, 23, 23, 25, 25, 27, 27,
    3, 3, 4, 4, 6, 6, 7, 7, 9, 9, 11, 11, 12, 12, 14, 14,
    16, 16, 17, 17, 19, 19, 20, 20, 22, 22, 24, 24, 25, 25, 27, 27,
    3, 3, 4, 4, 6, 6, 8, 8, 9, 9, 11, 11, 12, 12, 14, 14,
    16, 16, 17, 17, 19, 19, 21, 21, 22, 22, 24, 24, 25, 25, 27, 27,
    3, 3, 5, 5, 6, 6, 8, 8, 10, 10, 11, 11, 13, 13, 14, 14,
    16, 16, 18, 18, 19, 19, 21, 21, 23, 23, 24, 24, 26, 26, 27, 27,
    3, 3, 5, 5, 6, 6, 8, 8, 10, 10, 11, 11, 13, 13, 15, 15,
    16, 16, 18, 18, 19, 19, 21, 21, 23, 23, 24, 24, 26, 26, 28, 28,
    1, 2, 3, 3, 4, 5, 6, 7, 8, 8, 9, 10, 11, 12, 12, 13,
    14, 15, 16, 16, 17, 18, 19, 20, 21, 21, 22, 23, 24, 25, 25, 26,
    1, 2, 3, 4, 5, 5, 6, 7, 8, 9, 9, 10, 11, 12, 13, 13,
    14, 15, 16, 17, 18, 18, 19, 20, 21, 22, 22, 23, 24, 25, 26, 26,
    2, 2, 3, 4, 5, 6, 6, 7, 8, 9, 10, 10, 11, 12, 13, 14,
    15, 15, 16, 17, 18, 19, 19, 20, 21, 22, 23, 23, 24, 25, 26, 27,
    2, 3, 3, 4, 5, 6, 7, 7, 8, 9, 10, 11, 12, 12, 13, 14,
    15, 16, 16, 17, 18, 19, 20, 20, 21, 22, 23, 24, 25, 25, 26, 27,
    2, 3, 4, 4, 5, 6, 7, 8, 9, 9, 10, 11, 12, 13, 13, 14,
    15, 16, 17, 17, 18, 19, 20, 21, 22, 22, 23, 24, 25, 26, 26, 27,
    2, 3, 4, 5, 6, 6, 7, 8, 9, 10, 10, 11, 12, 13, 14, 14,
    15, 16, 17, 18, 19, 19, 20, 21, 22, 23, 23, 24, 25, 26, 27, 27,
    3, 3, 4, 5, 6, 7, 7, 8, 9, 10, 11, 11, 12, 13, 14, 15,
    16, 16, 17, 18, 19, 20, 20, 21, 22, 23, 24, 24, 25, 26, 27, 28,
    3, 4, 4, 5, 6, 7, 8, 8, 9, 10, 11, 12, 13, 13, 14, 15,
    16, 17, 17, 18, 19, 20, 21, 21, 22, 23, 24, 25, 26, 26, 27, 28,
    3, 4, 5, 5, 6, 7, 8, 9, 10, 10, 11, 12, 13, 14, 14, 15,
    16, 17, 18, 18, 19, 20, 21, 22, 23, 23, 24, 25, 26, 27, 27, 28,
    3, 4, 5, 6, 7, 7, 8, 9, 10, 11, 11, 12, 13, 14, 15, 15,
    16, 17, 18, 19, 20, 20, 21, 22, 23, 24, 24, 25, 26, 27, 28, 28,
];
