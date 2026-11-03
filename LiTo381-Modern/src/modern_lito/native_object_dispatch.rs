//! First-pass object selection, qualified against every original 16-bit flag.
//! This selects a route; producers still own source flags and pixel coverage.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Route {
    Skip=0,
    Half=1,
    Flag8=2,
    Flag80=3,
    Flag10=4,
    Green=5,
    Dim=6,
    Plain=7,
    Roof=8,
}
impl Route {
    pub fn writes_background_gate(self,map_id:u16)->bool {
        self==Self::Plain && map_id==303
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Input {
    pub flags:u16,
    /// Four signed source header bytes; negative values suppress the object.
    pub header:[i8;4],
    /// Bits0..3 admit source flag1000/2000/4000/8000 respectively.
    pub visibility:u8,
    pub focus:bool,
    pub roof_weight:i8,
    pub counter:u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Selection {
    pub route:Route,
    /// Raw green pulse16..28 or roof weight0..31; zero for other routes.
    pub argument:u8,
}
pub fn select(input:Input)->Selection {
    let selected=|route,argument|Selection {route,argument};
    let skip=selected(Route::Skip,0);
    if input.header.iter().any(|&n|n<0) {return skip;}
    for (flag,option) in [(0x1000,1),(0x2000,2),(0x4000,4),(0x8000,8)] {
        if input.flags&flag!=0 && input.visibility&option==0 {return skip;}
    }
    let f=input.flags;
    if f&0x40!=0 {return if input.focus {selected(Route::Half,0)} else {skip};}
    if f&0xc00!=0 {return if (0..32).contains(&input.roof_weight) {
        selected(Route::Roof,input.roof_weight as u8)
    } else {skip};}
    if f&2!=0 {return skip;}
    if f&8!=0 {return selected(Route::Flag8,0);}
    if f&0x101!=0 {return selected(Route::Half,0);}
    if f&0x80!=0 {return selected(Route::Flag80,0);}
    if f&0x10!=0 {return selected(Route::Flag10,0);}
    if f&0x20!=0 {return selected(Route::Green,(12u8.abs_diff((input.counter%24) as u8))+16);}
    if f&0x200!=0 {return selected(Route::Dim,0);}
    selected(Route::Plain,0)
}

/// Prepared SPR/TIL sources have already passed their structural decoder. The
/// raster retains the selected route kind, not a fabricated source header.
pub(crate) fn prepared_route(flags:u16,roof_weight:i8)->Route {
    select(Input {flags,header:[0;4],visibility:15,focus:true,roof_weight,counter:0}).route
}
