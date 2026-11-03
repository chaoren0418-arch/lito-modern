use lito_modern::modern_lito::native_object_dispatch::{self as dispatch,Input};
use sha2::{Digest,Sha256};

#[test]
fn all_263423_original_object_routes_arguments_and_gate_bytes_match_rust() {
    let json=include_bytes!("fixtures/native-object-dispatch-original.json");
    let outcomes=include_bytes!("fixtures/native-object-dispatch-outcomes.bin");
    assert_eq!(format!("{:X}",Sha256::digest(json)),"E65641622CA1B1F62E15B6879C1ACC37FC22AD23C1C48184DAB24A3476E3B427");
    assert_eq!(format!("{:X}",Sha256::digest(outcomes)),"D86A7AFC086201C82D7F393EE155E3806F3E3B6B20E659EB2B60AFE5D4C97A4B");
    let fixture:serde_json::Value=serde_json::from_slice(json).unwrap();
    assert_eq!(fixture["format"],"LITO_OBJECT_DISPATCH_ORIGINAL1");
    assert_eq!(outcomes.len(),263423*3);
    let mut checked=0;
    let mut check=|p:[i64;6]| {
        let [flags,map,options,header,roof,counter]=p;
        let mut bytes=[0;4];if header>0 {bytes[(header as usize-1)%4]=if header<=4 {-128}else{-1};}
        let result=dispatch::select(Input {flags:flags.try_into().unwrap(),header:bytes,
            visibility:(options as u8)&15,focus:options&16!=0,roof_weight:roof.try_into().unwrap(),counter:counter.try_into().unwrap()});
        let actual=[result.route as u8,result.argument,u8::from(result.route.writes_background_gate(map.try_into().unwrap()))];
        assert_eq!(actual.as_slice(),
            &outcomes[checked*3..checked*3+3],"case{checked} input{p:?}");checked+=1;
    };
    for options in [0,31] {for map in [302,303] {for flags in 0..65536 {check([flags,map,options,0,15,0]);}}}
    assert_eq!(fixture["targeted_inputs"].as_array().unwrap().len(),1279);
    for p in fixture["targeted_inputs"].as_array().unwrap() {
        let p:Vec<_>=p.as_array().unwrap().iter().map(|v|v.as_i64().unwrap()).collect();
        check(p.try_into().unwrap());
    }
    assert_eq!(checked,263423);
}
