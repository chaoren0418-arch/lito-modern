use lito_modern::modern_lito::native_background_pixels::{self as bg, Blend, Format, Image, Paint, Rect, Strength, Target};
use sha2::{Digest, Sha256};

fn fixture() -> serde_json::Value {
    let bytes = include_bytes!("fixtures/native-background-pixels-original.json");
    assert_eq!(format!("{:X}", Sha256::digest(bytes)), "1E4F702BA605DBEF9C31ECA4B3EAA135208FB680245BAD8021179069D58E995B");
    let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    assert_eq!(value["format"], "LITO_BACKGROUND_PIXELS_ORIGINAL1");
    value
}
fn format(n: u64) -> Format {
    match n { 0 => Format::Rgb555, 1 => Format::Rgb565, _ => panic!("fixture format") }
}
fn blend(n: u64) -> Blend {
    match n { 0 => Blend::Screen, 1 => Blend::Multiply, _ => panic!("fixture blend") }
}
fn strength(n: u64) -> Strength {
    if n == 32 { Strength::Full } else { assert!(n <= 9); Strength::Attenuated(n as u8) }
}
fn image(width: usize, height: usize, format: Format, overlay: bool) -> Vec<u16> {
    let mut result = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let rgb = if overlay { [(5*x+3*y)%32, (7*x+y)%32, (x+11*y)%32] }
                else { [x%32, y%32, (x+3*y)%32] };
            let mut word = ((rgb[0] as u16) << if format == Format::Rgb555 {10} else {11})
                | ((rgb[1] as u16) << if format == Format::Rgb555 {5} else {6}) | rgb[2] as u16;
            if format == Format::Rgb565 && (x+y)%2 != 0 { word |= 0x20; }
            result.push(word);
        }
    }
    result
}

#[test]
fn all_original_135168_packed_channel_words_match_rust() {
    let f = fixture();
    let raw = include_bytes!("fixtures/native-background-channels-original.bin");
    assert_eq!(format!("{:X}", Sha256::digest(raw)), "03D786BB84587F2D06892E90AAFB8428018F2EB3DEA69A877157E7B1E7942FFB");
    let mut checked = 0;
    for block in f["table_blocks"].as_array().unwrap() {
        let format = format(block["format"].as_u64().unwrap());
        let blend = blend(block["blend_mode"].as_u64().unwrap());
        let shift = block["shift"].as_u64().unwrap() as u32;
        let mut offset = block["offset"].as_u64().unwrap() as usize;
        let start = offset;
        for level in block["strengths"].as_array().unwrap() {
            let strength = strength(level.as_u64().unwrap());
            for a in 0u16..32 {
                for b in 0u16..32 {
                    let expected = u16::from_le_bytes([raw[offset], raw[offset+1]]);
                    assert_eq!(bg::mix_word(format, a<<shift, b<<shift, blend, strength).unwrap(), expected,
                               "format{format:?} blend{blend:?} strength{strength:?} shift{shift} a{a} b{b}");
                    offset += 2; checked += 1;
                }
            }
        }
        assert_eq!(offset-start, block["words"].as_u64().unwrap() as usize*2);
    }
    assert_eq!(checked, 135168);
}

#[test]
fn all_1584_original_whole_frame_hashes_and_clipped_rectangles_match_rust() {
    let f = fixture();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 1584);
    let mut changed = 0;
    for (index, case) in cases.iter().enumerate() {
        let p: Vec<_> = case["params"].as_array().unwrap().iter().map(|v|v.as_u64().unwrap() as usize).collect();
        assert_eq!(p.len(), 15); assert_eq!(p[0], index); assert_eq!(p[14], 1);
        let format = format(p[1] as u64);
        let base = image(32,32,format,false);
        let overlay = image(64,64,format,true);
        let mut pixels = vec![0x1777; p[2]*552];
        let gate: Vec<_> = (0..744*552).map(|i| match p[5] {0=>0,1=>1,7=>if (i/744+i%744)%2!=0 {7} else {0},_=>panic!("fixture gate")}).collect();
        let stats = bg::paint(Image::new(32,32,&base,format).unwrap(), Image::new(64,64,&overlay,format).unwrap(),
            Paint {source:Rect {left:p[10],top:p[11],right:p[12],bottom:p[13]},destination:[p[8],p[9]],
                   offsets:[p[6] as i32,p[7] as i32],blend:blend(p[4] as u64),strength:strength(p[3] as u64)},
            Target {words:&mut pixels,pitch:p[2],height:552,gate:&gate,gate_pitch:744}).unwrap();
        assert_eq!(stats.clipped_source_bottom, case["clipped_source_bottom"].as_u64().unwrap() as usize, "case{index}");
        let packed: Vec<_> = pixels.iter().flat_map(|v|v.to_le_bytes()).collect();
        assert_eq!(format!("{:X}",Sha256::digest(&packed)), case["frame_sha256"].as_str().unwrap(), "case{index}");
        let count = pixels.iter().filter(|&&v|v!=0x1777).count();
        assert_eq!(count, case["changed_words"].as_u64().unwrap() as usize, "case{index}"); changed += count;
    }
    assert_eq!(changed, 327376);
}

#[test]
fn invalid_images_layouts_phases_or_strength_fail_before_destination_changes() {
    let base_words = [0;4]; let overlay_words = [0;16];
    let base = Image::new(2,2,&base_words,Format::Rgb555).unwrap();
    let overlay = Image::new(4,4,&overlay_words,Format::Rgb555).unwrap();
    assert!(Image::new(0,2,&[],Format::Rgb555).is_err());
    assert!(Image::new(2,2,&[0;3],Format::Rgb555).is_err());
    assert!(Image::new(2,2,&[0x8000;4],Format::Rgb555).is_err());
    assert!(bg::mix_word(Format::Rgb555,0x8000,0,Blend::Screen,Strength::Full).is_err());
    let valid = Paint {source:Rect {left:0,top:0,right:2,bottom:2},destination:[0,0],offsets:[0,0],blend:Blend::Screen,strength:Strength::Full};
    let invalid = [Paint {strength:Strength::Attenuated(10),..valid},
        Paint {source:Rect {right:3,..valid.source},..valid},
        Paint {offsets:[-1,0],..valid}, Paint {offsets:[2,i32::MAX],..valid},
        Paint {destination:[usize::MAX,0],..valid}, Paint {destination:[0,usize::MAX],..valid}];
    let gate = [0;16];
    for call in invalid {
        let mut pixels = [0xabcd;16];
        assert!(bg::paint(base,overlay,call,Target {words:&mut pixels,pitch:4,height:4,gate:&gate,gate_pitch:4}).is_err());
        assert_eq!(pixels,[0xabcd;16]);
    }
    let other_format = Image::new(4,4,&overlay_words,Format::Rgb565).unwrap();
    let small_overlay = Image::new(2,2,&base_words,Format::Rgb555).unwrap();
    for overlay in [other_format, small_overlay] {
        let mut pixels = [0xabcd;16];
        assert!(bg::paint(base,overlay,valid,Target {words:&mut pixels,pitch:4,height:4,gate:&gate,gate_pitch:4}).is_err());
        assert_eq!(pixels,[0xabcd;16]);
    }
    let mut pixels = [0xabcd;16];
    assert!(bg::paint(base,overlay,valid,Target {words:&mut pixels,pitch:4,height:4,gate:&gate[..15],gate_pitch:4}).is_err());
    assert_eq!(pixels,[0xabcd;16]);
}

#[test]
fn encoded_black_writes_and_every_nonzero_gate_value_preserves_the_old_word() {
    let base = [0;4]; let overlay = [0;16]; let gate = [0,1,7,255]; let mut pixels = [0xffff;4];
    let stats = bg::paint(Image::new(2,2,&base,Format::Rgb555).unwrap(),Image::new(4,4,&overlay,Format::Rgb555).unwrap(),
        Paint {source:Rect {left:0,top:0,right:2,bottom:2},destination:[0,0],offsets:[0,0],blend:Blend::Screen,strength:Strength::Full},
        Target {words:&mut pixels,pitch:2,height:2,gate:&gate,gate_pitch:2}).unwrap();
    assert_eq!(pixels,[0,0xffff,0xffff,0xffff]); assert_eq!(stats.written_pixels,1);
}
