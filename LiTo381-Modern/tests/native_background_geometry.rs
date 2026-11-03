use lito_modern::modern_lito::native_background_geometry::{self as geometry, Input};
use lito_modern::modern_lito::native_background_pixels::{Blend, Strength};
use lito_modern::modern_lito::world_depth::NativeDrawCell;
use sha2::{Digest, Sha256};

fn input() -> Input {
    Input { map_id:303, level:0, class:0, cell:NativeDrawCell { half_x:0, y:0 },
        camera:[48,48], offsets:[0xffcc,0xfffe] }
}

#[test]
fn all_2688_original_cases_and_17472_ordered_calls_match_rust() {
    let raw=include_bytes!("fixtures/native-background-geometry-original.json");
    assert_eq!(format!("{:X}",Sha256::digest(raw)),
        "F4B9F257B740B197CCE25F9BA2369FF5F13A1F16355BBF27E5F8F4E35BE36091");
    let fixture:serde_json::Value=serde_json::from_slice(raw).unwrap();
    assert_eq!(fixture["format"],"LITO_BACKGROUND_GEOMETRY_ORIGINAL1");
    assert_eq!(fixture["cell_x_is_native_half_cell"],true);
    let cases=fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(),2688);
    let mut checked=0;
    for (index,case) in cases.iter().enumerate() {
        let p:Vec<_>=case["input"].as_array().unwrap().iter().map(|v|v.as_i64().unwrap()).collect();
        assert_eq!(p.len(),9);
        let calls=geometry::plan(Input {map_id:p[0].try_into().unwrap(),level:p[1].try_into().unwrap(),
            class:p[2].try_into().unwrap(),cell:NativeDrawCell {half_x:p[3].try_into().unwrap(),y:p[4].try_into().unwrap()},
            camera:[p[5].try_into().unwrap(),p[6].try_into().unwrap()],offsets:[p[7].try_into().unwrap(),p[8].try_into().unwrap()]}).unwrap();
        let expected=case["calls"].as_array().unwrap();
        assert_eq!(calls.len(),expected.len(),"case{index}");
        for (ordinal,(call,expected)) in calls.iter().zip(expected).enumerate() {
            let paint=call.paint;
            let strength=match paint.strength {Strength::Full=>32,Strength::Attenuated(n)=>i64::from(n)};
            let blend=match paint.blend {Blend::Screen=>0,Blend::Multiply=>1};
            let actual=vec![i64::from(call.base_id),i64::from(call.overlay_id),paint.source.left as i64,
                paint.source.top as i64,paint.source.right as i64,paint.source.bottom as i64,
                paint.destination[0] as i64,paint.destination[1] as i64,
                i64::from(paint.offsets[0]),i64::from(paint.offsets[1]),strength,blend];
            let expected:Vec<_>=expected.as_array().unwrap().iter().map(|v|v.as_i64().unwrap()).collect();
            assert_eq!(actual,expected,"case{index} call{ordinal}");
            checked+=1;
        }
    }
    assert_eq!(checked,17472);
}

#[test]
fn signed_negative_odd_phase_and_conditional_522_bottom_are_preserved() {
    let calls=geometry::plan(Input {cell:NativeDrawCell {half_x:-192,y:0},..input()}).unwrap();
    assert_eq!(calls[0].paint.destination,[0,0]);
    assert_eq!(calls[0].paint.source.top,192);
    let calls=geometry::plan(input()).unwrap();
    assert_eq!(calls[0].paint.source.bottom,384);
    assert_eq!(calls[1].paint.destination,[0,384]);
    assert_eq!(calls[1].paint.source.bottom,138);
    assert_eq!(calls[1].paint.destination[1]+calls[1].paint.source.bottom,522);
}

#[test]
fn malformed_coordinates_and_light_domain_return_errors_without_panics() {
    for level in [32,255] {assert!(geometry::plan(Input {level,..input()}).is_err());}
    for value in [i32::MIN,i32::MAX] {
        for cell in [NativeDrawCell {half_x:value,y:value},NativeDrawCell {half_x:0,y:value}] {
            assert!(geometry::plan(Input {cell,..input()}).is_err());
        }
        for camera in [[value,48],[48,value]] {
            assert!(geometry::plan(Input {camera,..input()}).is_err());
        }
    }
}
