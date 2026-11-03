use lito_modern::modern_lito::{native_services,native_room::{self,Packet}};
fn bytes(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
#[test]
fn direct_room_and_vs_directory_preserve_separate_unsigned_fields(){
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-services-current.json")).unwrap();
    for v in f["vectors"].as_array().unwrap(){let b=bytes(v["body"].as_str().unwrap());let d=native_services::parse(&b).unwrap();assert_eq!(d.rooms.len(),v["rooms"].as_u64().unwrap() as usize);assert_eq!(d.servers.len(),v["servers"].as_u64().unwrap() as usize);
        for r in &d.rooms{assert_eq!(r.endpoint.to_string(),"127.0.0.2:65535");assert_eq!(r.users,u32::MAX);}
        for r in &d.servers{assert_eq!(r.endpoint.to_string(),"127.0.0.1:1");assert_eq!(r.users,0x80000000);assert_eq!(r.capacity,u32::MAX);}
        // Boundary and last-byte truncations plus all truncations of the ordinary one-row form.
        let ends:Vec<usize>=if b.len()<1024{(0..b.len()).collect()}else{vec![0,1,4,8,9,b.len()-1]};for n in ends{assert!(native_services::parse(&b[..n]).is_err(),"truncated {n}");}
        let mut tail=b;tail.push(0);assert!(native_services::parse(&tail).is_err());
    }
    for b in [[0xfb,128,0,0,0,0,0,0,0],[0xfb,0,0,0,0,11,0,0,0]]{assert!(native_services::parse(&b).is_err());}
}
#[test]
fn direct_room_join_keeps_mode_six_and_map_without_inventing_channel_membership(){
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-services-current.json")).unwrap();
    let Packet::Joined(room)=native_room::parse(&bytes(f["rental_join"].as_str().unwrap())).unwrap()else{panic!()};
    assert_eq!((room.id,room.mode,room.map_id,room.users_summary),(u32::MAX,6,500,2));assert!(room.members.is_empty());
}
