use lito_modern::modern_lito::{native_light_color::Format,native_light_mask::Mask,
    native_water_light::{Channels,Params,Shade}};
use std::collections::BTreeMap;
struct Fixture {tables:&'static[u8],shade:&'static[u8],samples:&'static[u8]}
fn fixture()->Fixture {
    let b=include_bytes!("fixtures/native-water-original.bin");assert_eq!(&b[..8],b"LTWTR01\0");
    let sizes:Vec<_>=b[8..24].chunks_exact(4).map(|b|u32::from_le_bytes(b.try_into().unwrap()) as usize).collect();
    assert_eq!(sizes,[1472,122880,268800,24192]);let at=24+sizes[0];let middle=at+sizes[1];let end=middle+sizes[2];
    assert_eq!(b.len(),end+17*sizes[3]);Fixture{tables:&b[at..middle],shade:&b[middle..end],samples:&b[end..]}
}
#[derive(Clone,Copy)]
struct Sample {x:usize,y:usize,raw:u8,color:u16,shade:u8,level:u8,actual:u16}
fn word(b:&[u8],at:usize)->u16 {u16::from_le_bytes(b[at..at+2].try_into().unwrap())}

#[test]
fn all_native_water_channel_tables_and_private_CRT_resource_plane_match() {
    let f=fixture();
    for (mode,format) in [Format::Rgb555,Format::Rgb565].into_iter().enumerate() {
        let channels=Channels::new(format);let bytes=&f.tables[mode*61440..(mode+1)*61440];
        let expected:Vec<_>=bytes.chunks_exact(2).map(|b|word(b,0)).collect();assert_eq!(channels.native_words(),expected);
    }
    let raw=std::fs::read(r"D:\lineage開發\0-客戶端\shade.dat").unwrap();
    let shade=Shade::parse_resource(&raw).unwrap();assert_eq!(shade.values(),f.shade);
    let mut bad=raw;bad[0]^=1;assert!(Shade::parse_resource(&bad).is_err());
    assert!(Shade::from_clamped(&f.shade[..f.shade.len()-1]).is_err());
}

#[test]
fn original_water_raster_samples_and_surface_guards_match_in_both_formats() {
    let f=fixture();let shade=Shade::from_clamped(f.shade).unwrap();
    let mut groups=BTreeMap::<(u8,i8,u16,u16),Vec<Sample>>::new();
    for b in f.samples.chunks_exact(17) {
        groups.entry((b[0],b[1] as i8,word(b,2),word(b,4))).or_default().push(Sample{
            x:usize::from(word(b,6)),y:usize::from(word(b,8)),raw:b[10],color:word(b,11),shade:b[13],level:b[14],actual:word(b,15)});
    }
    assert_eq!(groups.len(),56);let mut checked=0;
    for ((mode,phase,sx,sy),samples) in groups {
        assert_eq!(samples.len(),432);let params=Params{phase,scroll:[sx,sy]};
        let format=if mode==0 {Format::Rgb555}else{Format::Rgb565};let channels=Channels::new(format);
        let mut pixels:Vec<_>=(0..800*414).map(|i|((i as i64)*7919+i64::from(phase)*31+i64::from(sx)) as u16).collect();let before=pixels.clone();
        let raw:Vec<_>=(0..320*204).map(|i|(i*37+usize::from(sy)) as u8).collect();
        let mut mask=Mask::new(320,204).unwrap();mask.compose_plane(&raw,320,204,0,0).unwrap();
        let row=std::array::from_fn(|i|((i*13+7)%32) as u8);
        let receipt=channels.apply_words(&mut pixels,800,[33,3],[640,408],&mask,&row,&shade,params).unwrap();assert_eq!(receipt.transformed,640*408);
        for sample in samples {
            let at=(sample.y+3)*800+sample.x+33;
            assert_eq!(before[at],sample.color);assert_eq!(raw[(sample.y/2)*320+sample.x/2],sample.raw);
            assert_eq!(row[usize::from(sample.raw>>3)],sample.level);
            assert_eq!(shade.sample(params,sample.x,sample.y).unwrap(),sample.shade);
            assert_eq!(pixels[at],sample.actual,"mode{mode} phase{phase} scroll{sx},{sy} at{},{}",sample.x,sample.y);checked+=1;
        }
        // Observed native counterexample retained from the failed scalar audit:
        // viewport(0,11), native Y408 ->1249; naive wrapping Y0 ->2306.
        if mode==0 && phase==-21 && sx==0 && sy==419 {assert_eq!(pixels[14*800+33],1249);}
        for y in 0..414 {for x in 0..800 {
            if !(3..411).contains(&y) || !(33..673).contains(&x) {assert_eq!(pixels[y*800+x],before[y*800+x]);}
        }}
    }
    assert_eq!(checked,24192);
}

#[test]
fn rgba_water_has_no_level31_identity_shortcut_and_preserves_alpha_and_atomic_errors() {
    let f=fixture();let shade=Shade::from_clamped(f.shade).unwrap();let channels=Channels::new(Format::Rgb555);
    let mut mask=Mask::new(2,2).unwrap();mask.compose_plane(&[0,31,127,255],2,2,0,0).unwrap();
    let params=Params{phase:-21,scroll:[639,419]};let row=[31;32];
    let original:Vec<u16>=vec![0,0x7fff,0x1234,0x4321,0x4210,0x1111,0x2222,0x3333,0x5555];
    let expand=|v:u16|{let v=(v&31) as u8;(v<<3)|(v>>2)};
    let mut rgba=Vec::new();for (i,c) in original.iter().enumerate(){rgba.extend([expand(c>>10),expand(c>>5),expand(*c),i as u8]);}
    let before=rgba.clone();let mut words=original.clone();
    channels.apply_words(&mut words,3,[0,0],[3,3],&mask,&row,&shade,params).unwrap();
    channels.apply_rgba555(&mut rgba,[3,3],&mask,&row,&shade,params).unwrap();assert_ne!(rgba,before);
    for (i,c) in words.iter().enumerate(){assert_eq!(&rgba[i*4..i*4+4],&[expand(c>>10),expand(c>>5),expand(*c),i as u8]);}
    for bad in [Params{phase:-128,..params},Params{scroll:[640,0],..params},Params{scroll:[0,420],..params}] {
        let mut pixels=before.clone();assert!(channels.apply_rgba555(&mut pixels,[3,3],&mask,&row,&shade,bad).is_err());assert_eq!(pixels,before);
    }
    let mut pixels=before.clone();let mut bad_row=row;bad_row[17]=32;
    assert!(channels.apply_rgba555(&mut pixels,[3,3],&mask,&bad_row,&shade,params).is_err());assert_eq!(pixels,before);
    let mut short=original.clone();assert!(channels.apply_words(&mut short,usize::MAX,[0,1],[3,3],&mask,&row,&shade,params).is_err());assert_eq!(short,original);
}
