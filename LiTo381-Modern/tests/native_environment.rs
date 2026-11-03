use lito_modern::modern_lito::native_environment::{Calendar, State, daylight, map_mask_argument, map_policy_flag};
use sha2::{Digest, Sha256};

fn pinned(bytes: &[u8], sha: &str) { assert_eq!(format!("{:X}", Sha256::digest(bytes)), sha); }
fn time(raw: u32) -> Vec<u8> { let mut b = vec![0x21]; b.extend(raw.to_le_bytes()); b }

#[test]
fn all_223729_actual_C02_calendar_vectors_include_historical_DST_and_native_time_t_bounds() {
    let b = include_bytes!("fixtures/native-environment-calendar.bin");
    pinned(b, "0C5508BA7D9D438E5BA2DA7D68C97E8C183793B7C7A31ED85C0BADFCE44B42AB");
    assert_eq!(b.len(), 223729 * 14);
    for v in b.chunks_exact(14) {
        let c = Calendar::from_raw(u32::from_le_bytes(v[..4].try_into().unwrap())).unwrap();
        assert_eq!((c.year, c.day_of_year), (u16::from_le_bytes([v[4],v[5]]), u16::from_le_bytes([v[6],v[7]])));
        assert_eq!([c.month,c.day,c.hour,c.minute,c.second], [v[8],v[9],v[10],v[11],v[12]]);
        assert_eq!(c.offset_seconds, i32::from(v[13]) * 3600);
    }
    assert!(Calendar::from_raw(0x80000000).is_err());
    assert!(Calendar::from_raw(u32::MAX).is_err());
}

#[test]
fn original_daylight_43920_boundaries_preserve_signed_division_and_three_weather_flags() {
    let b = include_bytes!("fixtures/native-environment-daylight.bin");
    pinned(b, "938312159E92D72249B7051874B54EE82E4D25626AFFB81EE2874A91B054F746");
    assert_eq!(b.len(), 43920 * 10);
    for v in b.chunks_exact(10) {
        let d = u16::from_le_bytes([v[0],v[1]]); let m = u16::from_le_bytes([v[2],v[3]]);
        let actual = daylight(d, m, [v[4]&1, (v[4]>>1)&1, (v[4]>>2)&1]).unwrap();
        assert_eq!((actual.level,actual.sunrise,actual.sunset,actual.minute),
            (v[5],u16::from_le_bytes([v[6],v[7]]),u16::from_le_bytes([v[8],v[9]]),m));
    }
    assert!(daylight(366, 0, [0;3]).is_err());
    assert!(daylight(0, 1440, [0;3]).is_err());
}

#[test]
fn every_native_map_flag_and_all_four_65536_argument_sweeps_match_original_fragments() {
    let flags = include_bytes!("fixtures/native-environment-map-flag.bin");
    let levels = include_bytes!("fixtures/native-environment-map-level.bin");
    pinned(flags, "1F3B212E90E233D9E10D0C0171005063F3331F1B44E95E6EFE90F1A1B7DADFFD");
    pinned(levels, "6D218CDC3A30754CEB4AEB2ACF1AA3AF196832D5669A9FB1C9081F2C39EB08DC");
    assert_eq!(flags.len(),65536); assert_eq!(levels.len(),4*65536);
    for map in 0..65536 {
        assert_eq!(u8::from(map_policy_flag(map as u16)),flags[map]);
        for (i,flag) in [0,1,128,255].into_iter().enumerate() {
            assert_eq!(map_mask_argument(map as u16,flag),levels[i*65536+map]);
        }
    }
    for map in [4,500,501,502,503,510,514,564,573] {assert_eq!(map_mask_argument(map,u8::from(map_policy_flag(map))),15);}
}

#[test]
fn weather_repeats_and_unknown_channels_apply_the_original_ambient_side_effect() {
    let mut s=State::default(); s.begin(2).unwrap();s.accept(2,&time(0)).unwrap();s.map_changed(2,500,Some(0)).unwrap();
    assert_eq!(s.ambient_level,Some(15));
    s.accept(2,&[0x33,1]).unwrap(); assert_eq!((s.weather,s.ambient_level),([1,0,0],Some(14)));
    s.accept(2,&[0x33,1]).unwrap(); assert_eq!(s.ambient_level,Some(13));
    s.accept(2,&[0x33,0xf3]).unwrap(); assert_eq!((s.weather,s.ambient_level),([1,0,0],Some(12)));
    s.accept(2,&[0x33,4]).unwrap(); assert_eq!((s.weather,s.ambient_level),([0,0,0],Some(12)));
    s.accept(2,&[0x33,0x23]).unwrap(); assert_eq!((s.weather,s.ambient_level),([0,0,3],Some(11)));
    s.accept(2,&[0x33,0x13]).unwrap(); assert_eq!((s.weather,s.ambient_level),([0,3,3],Some(10)));
    s.accept(2,&[0x33,0x13]).unwrap(); assert_eq!(s.ambient_level,Some(9));
    s.accept(2,&[0x33,0x13]).unwrap(); assert_eq!(s.ambient_level,Some(9));
    s.accept(2,&time(0)).unwrap(); assert_eq!(s.ambient_level,Some(14));
}

#[test]
fn blind_signed_domain_and_malformed_packets_never_partially_mutate_environment() {
    let mut s=State::default();s.begin(7).unwrap();s.accept(7,&[0x0a,2]).unwrap();
    for raw in [2,3,127,128,255] {let before=s.clone();assert!(s.accept(7,&[0x0a,raw]).unwrap());assert_eq!(s,before);}
    let mut bad_status=vec![0;37];bad_status[0]=0x0c;bad_status[25..29].copy_from_slice(&0x80000000u32.to_le_bytes());
    for body in [vec![0x0a],vec![0x0a,1,0],vec![0x33],vec![0x33,1,0],vec![0x21],time(0x80000000),vec![0x0c;36],bad_status] {
        let before=s.clone();assert!(s.accept(7,&body).is_err());assert_eq!(s,before);
    }
    s.accept(7,&[0x0a,0]).unwrap();assert_eq!(s.blind,0);
}

#[test]
fn map_and_portal_replacements_preserve_GAME_time_weather_blindness_and_water_authority() {
    let mut s=State::default();s.begin(2).unwrap();s.accept(2,&time(338466069)).unwrap();
    s.map_changed(2,500,Some(255)).unwrap();s.accept(2,&[0x0a,1]).unwrap();s.accept(2,&[0x33,0x12]).unwrap();
    let calendar=s.calendar;
    s.map_changed(2,511,None).unwrap();
    assert_eq!((s.calendar,s.weather,s.blind,s.underwater,s.ambient_level,s.map_mask_argument),
        (calendar,[0,2,0],1,true,Some(1),Some(2)));
    s.accept(2,&time(0)).unwrap();assert_eq!(s.ambient_level,Some(1));
    s.map_changed(2,500,None).unwrap();assert_eq!(s.ambient_level,Some(14));
    assert!(s.underwater);s.map_changed(2,500,Some(0)).unwrap();assert!(!s.underwater);
    assert_eq!((s.weather,s.blind),([0,2,0],1));
}

#[test]
fn status_zero_is_authority_and_positive_override_counter_suppresses_only_time_ambient_writes() {
    let mut s=State::default();s.begin(4).unwrap();s.map_changed(4,500,Some(0)).unwrap();s.accept(4,&time(338466069)).unwrap();
    let mut status=vec![0x0c];status.resize(37,0);s.accept(4,&status).unwrap();
    assert_eq!((s.calendar.unwrap().raw,s.time_packets,s.status_time_packets),(0,1,1));
    s.ambient_override_counter=1;let before=s.ambient_level;s.accept(4,&time(60000)).unwrap();assert_eq!(s.ambient_level,before);
    s.accept(4,&[0x33,1]).unwrap();assert_eq!(s.ambient_level,before.map(|v|v-1));
    s.ambient_override_counter=-1;s.accept(4,&time(60000)).unwrap();assert_eq!(s.ambient_level,s.daylight.map(|v|v.level));
}

#[test]
fn retired_epochs_cannot_resurrect_and_new_epochs_start_without_old_scene_authority() {
    let mut s=State::default();assert!(s.begin(0).is_err());s.begin(2).unwrap();s.accept(2,&time(123)).unwrap();
    s.map_changed(2,500,Some(1)).unwrap();s.accept(2,&[0x0a,2]).unwrap();s.accept(2,&[0x33,3]).unwrap();
    let before=s.clone();assert!(!s.begin(2).unwrap());assert_eq!(s,before);
    assert!(s.accept(1,&time(0)).is_err());assert_eq!(s,before);s.retire();
    assert!(s.epoch.is_none()&&s.calendar.is_none()&&s.map.is_none()&&s.ambient_level.is_none());
    assert_eq!((s.blind,s.weather,s.underwater),(0,[0;3],false));
    assert!(s.begin(2).is_err());assert!(s.accept(2,&time(0)).is_err());s.begin(4).unwrap();
    assert_eq!((s.epoch,s.calendar,s.time_packets),(Some(4),None,0));
}

#[test]
fn real_monotonic_water_poll_rearms_once_per_ready_generation_and_retires_game_callbacks() {
    let mut s=State::default();s.begin(2).unwrap();s.map_changed(2,500,Some(1)).unwrap();
    s.water_poll(2,Some(7)).unwrap();
    assert_eq!(s.water_clock.core.events().iter().filter(|e|e.callback==lito_modern::modern_lito::native_water_clock::Callback::Water).count(),1);
    let before=s.water_clock.core.phase();
    std::thread::sleep(std::time::Duration::from_millis(120));
    assert!(s.water_poll(2,Some(7)).unwrap());let after=if before>20 {-20}else{before+1};assert_eq!(s.water_clock.core.phase(),after);
    let fired=s.water_poll(2,Some(8)).unwrap();let after=if fired {if after>20 {-20}else{after+1}}else{after};
    assert_eq!(s.water_clock.core.phase(),after);
    assert_eq!(s.water_clock.core.events().iter().filter(|e|e.callback==lito_modern::modern_lito::native_water_clock::Callback::Water).count(),1);
    s.retire_scene();assert!(s.water_clock.core.events().is_empty());assert!(!s.underwater);assert_eq!(s.water_clock.owner(),Some(2));
    assert_eq!(s.water_clock.core.phase(),after);
    s.retire();assert!(s.water_clock.core.events().is_empty());assert_eq!(s.water_clock.core.phase(),after);
    assert!(s.water_poll(2,Some(9)).is_err());s.begin(4).unwrap();assert_eq!(s.water_clock.core.phase(),after);
}
