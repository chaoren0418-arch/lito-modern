//! Native floor copies retain encoded coverage and ignore overlay alpha effects.
use super::tile::ClassicTileType;
use anyhow::{ensure, Result};
pub const TILE_BYTES:usize=24*24*4;

/// Raw half-diamond orientation comes from the destination half cell, not
/// the low bit in the archive block. Overlay/preview decoding stays separate.
pub fn floor_pixels(kind:ClassicTileType,normal:&[u8],raw:Option<&[u8]>,parity:u8,
    output:&mut [u8;TILE_BYTES])->Result<bool> {
    ensure!(normal.len()==TILE_BYTES && parity<=1,"floor input dimensions/parity");
    if kind as u8 &8!=0 {output.fill(0);return Ok(false);}
    let source=if kind as u8 &0x30!=0 {raw.ok_or_else(||anyhow::anyhow!("floor effect lacks raw encoded coverage"))?}else{normal};
    ensure!(source.len()==TILE_BYTES,"floor raw dimensions");
    let half=matches!(kind,ClassicTileType::RleEvenHalfOpacity|ClassicTileType::RleOddHalfOpacity);
    ensure!(source.chunks_exact(4).all(|p|matches!(p[3],0|255)||(half&&p[3]==128)),"floor source coverage alpha");
    if kind as u8 &2!=0 {
        output.copy_from_slice(source);
        for p in output.chunks_exact_mut(4) {if p[3]!=0 {p[3]=255;}}
    }else{
        output.fill(0);
        for y in 0..24usize {
            let count=if y<12 {2*(y+1)}else{2*(23-y)};
            let from=if kind as u8 &1==0 {24-count}else{0};
            let to=if parity==0 {24-count}else{0};
            output[(y*24+to)*4..(y*24+to+count)*4].copy_from_slice(&source[(y*24+from)*4..(y*24+from+count)*4]);
        }
    }
    Ok(true)
}

/// Per-frame first-pass write provenance. Later special blends never erase
/// these bytes; ordinary object writes may add encoded coverage after floors.
pub struct Gate {width:u32,height:u32,values:Vec<u8>,covered:usize,unqualified_fragments:usize}
impl Gate {
    pub fn new(width:u32,height:u32)->Result<Self> {
        ensure!((1..=1024).contains(&width)&&(1..=1024).contains(&height),"terrain gate budget");
        let count=width as usize*height as usize;let mut values=Vec::new();values.try_reserve_exact(count)?;values.resize(count,0);
        Ok(Self{width,height,values,covered:0,unqualified_fragments:0})
    }
    pub fn dimensions(&self)->[u32;2] {[self.width,self.height]}
    pub fn values(&self)->&[u8] {&self.values}
    pub fn covered(&self)->usize {self.covered}
    pub fn unqualified_fragments(&self)->usize {self.unqualified_fragments}
    pub fn provenance_complete(&self)->bool {self.unqualified_fragments==0}
    pub(crate) fn note_unqualified_fragment(&mut self) {self.unqualified_fragments+=1;}
    /// Validation precedes mutation. An encoded black pixel still marks gate1.
    pub fn mark(&mut self,rgba:&[u8],x:i32,y:i32)->Result<()> {
        ensure!(rgba.len()==TILE_BYTES&&rgba.chunks_exact(4).all(|p|matches!(p[3],0|255)),"terrain gate requires binary encoded coverage");
        let left=(-i64::from(x)).clamp(0,24) as usize;let right=(i64::from(self.width)-i64::from(x)).clamp(0,24) as usize;
        let top=(-i64::from(y)).clamp(0,24) as usize;let bottom=(i64::from(self.height)-i64::from(y)).clamp(0,24) as usize;
        for sy in top..bottom {for sx in left..right {
            if rgba[(sy*24+sx)*4+3]==0 {continue;}
            let at=(i64::from(y)+sy as i64) as usize*self.width as usize+(i64::from(x)+sx as i64) as usize;
            if self.values[at]==0 {self.values[at]=1;self.covered+=1;}
        }}
        Ok(())
    }
}
