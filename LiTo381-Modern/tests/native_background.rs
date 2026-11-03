use lito_modern::modern_lito::{native_background::Bank,native_background_geometry::Input,
    native_background_pixels::Target,world_depth::NativeDrawCell};
use sha2::{Digest,Sha256};
use std::{path::Path,sync::atomic::{AtomicBool,Ordering}};

fn load()->Bank {
    Bank::load_map303(Path::new(r"D:\lineage開發\0-客戶端"),&AtomicBool::new(false)).unwrap()
}
fn input()->Input {
    Input {map_id:303,level:0,class:0,cell:NativeDrawCell {half_x:0,y:0},camera:[48,48],offsets:[65484,65534]}
}
#[test]
fn actual_native_first_IMG_bank_matches_all_twelve_original_whole_frames() {
    let bank=load();
    assert_eq!(bank.sources["localized_overrides_absent"],true);
    let members=bank.sources["members"].as_array().unwrap();assert_eq!(members.len(),2);
    for m in members {assert_eq!(m["archive"],"BASE/Sprite");assert_eq!(m["native_index_miss"],true);}
    let raw=include_bytes!("fixtures/native-background-real-original.json");
    assert_eq!(format!("{:X}",Sha256::digest(raw)),"8CDC0BAC8A9E663964A6C81A0E343C871B1E3CCE71DD6827AA06DAF05037C797");
    let fixture:serde_json::Value=serde_json::from_slice(raw).unwrap();
    assert_eq!(fixture["format"],"LITO_BACKGROUND_REAL_IMG_ORIGINAL1");
    let cases=fixture["cases"].as_array().unwrap();assert_eq!(cases.len(),12);
    for (i,c) in cases.iter().enumerate() {
        let p:Vec<_>=c["input"].as_array().unwrap().iter().map(|v|v.as_i64().unwrap()).collect();
        assert_eq!(p.len(),9);
        let input=Input {map_id:p[0].try_into().unwrap(),level:p[1].try_into().unwrap(),class:p[2].try_into().unwrap(),
            cell:NativeDrawCell {half_x:p[3].try_into().unwrap(),y:p[4].try_into().unwrap()},
            camera:[p[5].try_into().unwrap(),p[6].try_into().unwrap()],offsets:[p[7].try_into().unwrap(),p[8].try_into().unwrap()]};
        let mut words=vec![0x1777;744*552];
        let kind=c["gate"].as_u64().unwrap();
        let gate:Vec<u8>=(0..744*552).map(|j|match kind {0=>0,1=>1,7=>if (j/744+j%744)%2==1 {7}else{0},_=>panic!("gate")}).collect();
        let result=bank.paint(input,Target {words:&mut words,pitch:744,height:552,gate:&gate,gate_pitch:744},&AtomicBool::new(false)).unwrap();
        assert_eq!(result.calls,c["pixel_calls"].as_u64().unwrap() as usize,"case{i}");
        let bytes:Vec<_>=words.iter().flat_map(|w|w.to_le_bytes()).collect();
        assert_eq!(format!("{:X}",Sha256::digest(&bytes)),c["frame_sha256"].as_str().unwrap(),"case{i}");
        let changed=words.iter().filter(|&&w|w!=0x1777).count();
        assert_eq!(changed,c["changed_words"].as_u64().unwrap() as usize,"case{i}");
        if kind==1 {assert_eq!(result.written_pixels,0);} else {assert!(result.written_pixels>0);}
    }
}

#[test]
fn invalid_batch_layout_map_phase_or_cancel_never_changes_destination() {
    let bank=load();let cancel=AtomicBool::new(false);let gate=vec![0;744*552];let mut words=vec![0x1777;744*552];
    for c in [Input {map_id:302,..input()},Input {level:32,..input()},Input {camera:[i32::MAX,48],..input()},
        Input {offsets:[384,i32::MAX],..input()},Input {offsets:[-1,0],..input()}] {
        assert!(bank.paint(c,Target {words:&mut words,pitch:744,height:552,gate:&gate,gate_pitch:744},&cancel).is_err());
        assert!(words.iter().all(|&w|w==0x1777));
    }
    for (pitch,height,gate_pitch) in [(743,552,744),(744,551,744),(744,552,743)] {
        assert!(bank.paint(input(),Target {words:&mut words,pitch,height,gate:&gate,gate_pitch},&cancel).is_err());
        assert!(words.iter().all(|&w|w==0x1777));
    }
    assert!(bank.paint(input(),Target {words:&mut words,pitch:744,height:552,gate:&gate[..gate.len()-1],gate_pitch:744},&cancel).is_err());
    cancel.store(true,Ordering::Release);
    assert!(bank.paint(input(),Target {words:&mut words,pitch:744,height:552,gate:&gate,gate_pitch:744},&cancel).is_err());
    assert!(words.iter().all(|&w|w==0x1777));
    let error=Bank::load_map303(Path::new("a_missing_path_that_must_not_be_opened"),&cancel).err().unwrap();
    assert!(error.to_string().contains("background cancelled"));
}
