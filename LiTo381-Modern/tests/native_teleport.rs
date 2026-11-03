use lito_modern::modern_lito::native_services::{parse_availability,parse_endpoint};
use lito_modern::modern_lito::{native_teleport::Pending,native_merchant::Owner,native_inventory};
fn owner()->Owner{Owner{connection:3,generation:7,incarnation:2,control_interruptions:0,inventory:native_inventory::Owner{connection:3,selection:1,character:"測試角色".into()}}}
fn pending()->Pending{Pending::new(owner(),"測試角色".into(),"TeleportMe","LT_TELEPORT_B").unwrap()}
#[test]
fn only_reachable_page_choices_authorize_teleport(){
    for (page,action) in [("TeleportMe_C","LT_TELEPORT_B"),("TeleportMe","LT_TELEPORT_A"),("TeleportMe","LT_TELEPORT_D"),("Maya","LT_TELEPORT_C")]{assert!(Pending::new(owner(),"角色".into(),page,action).is_none());}
    assert_eq!(Pending::new(owner(),"角色".into(),"TeleportMe","LT_TELEPORT_C").unwrap().index,2);
}
#[test]
fn teleport_requires_matching_status_then_endpoint_once(){
    let mut p=pending();let status=parse_availability(&vector("configured-page-status")).unwrap();
    let endpoint=parse_endpoint(&[0xf4,1,127,0,0,3,0x34,0x12]).unwrap();let current="127.0.0.2:4660".parse().unwrap();
    assert!(p.endpoint(&owner(),endpoint.clone(),current).is_err());
    p.status(&owner(),status.clone()).unwrap();assert!(p.status(&owner(),status).is_err());
    let mut other=endpoint.clone();other.index=2;assert!(p.endpoint(&owner(),other,current).is_err());
    assert_eq!(p.endpoint(&owner(),endpoint.clone(),current).unwrap(),"127.0.0.3:4660".parse().unwrap());
    assert!(p.endpoint(&owner(),endpoint,current).is_err());
}
#[test]
fn stale_owner_and_unavailable_slots_cannot_redirect(){
    let mut p=pending();let mut stale=owner();stale.generation+=1;
    let available=parse_availability(&vector("configured-page-status")).unwrap();
    assert!(p.status(&stale,available.clone()).is_err());
    assert!(p.status(&owner(),parse_availability(&[0xf3,1,0,0,0,0]).unwrap()).is_err());
    p.status(&owner(),available).unwrap();
    let endpoint=parse_endpoint(&[0xf4,1,127,0,0,3,0x34,0x12]).unwrap();
    assert!(p.endpoint(&stale,endpoint.clone(),"127.0.0.2:4660".parse().unwrap()).is_err());
    let same="127.0.0.3:4660".parse().unwrap();
    assert_eq!(p.endpoint(&owner(),endpoint.clone(),same).unwrap(),same);
    assert!(p.endpoint(&owner(),endpoint,same).is_err());
}
#[test]
fn every_owner_dimension_rejects_stale_replies_without_consuming_current_pending(){
    let source:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-teleport-controller-current.json")).unwrap();
    assert_eq!(source["reference_sha256"],"C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539");
    let text=source["vectors"]["availability"]["hex"].as_str().unwrap();
    let bytes:Vec<_>=(0..text.len()).step_by(2).map(|i|u8::from_str_radix(&text[i..i+2],16).unwrap()).collect();
    let status=parse_availability(&bytes).unwrap();
    let reply=parse_endpoint(&[0xf4,1,127,0,0,3,0x34,0x12]).unwrap();
    for dimension in 0..7 {
        let mut stale=owner();
        match dimension {0=>stale.connection+=1,1=>stale.generation+=1,2=>stale.incarnation+=1,
            3=>stale.control_interruptions+=1,4=>stale.inventory.connection+=1,
            5=>stale.inventory.selection+=1,6=>stale.inventory.character.push('異'),_=>unreachable!()}
        let mut p=pending();assert!(p.status(&stale,status.clone()).is_err(),"dimension {dimension}");
        p.status(&owner(),status.clone()).unwrap();
        assert!(p.endpoint(&stale,reply.clone(),"127.0.0.2:4660".parse().unwrap()).is_err(),"dimension {dimension}");
        assert_eq!(p.endpoint(&owner(),reply.clone(),"127.0.0.2:4660".parse().unwrap()).unwrap(),reply.endpoint);
    }
}
fn vector(name:&str)->Vec<u8>{
    let source:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-teleport-current.json")).unwrap();
    assert_eq!(source["jar_sha256"],"BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43");
    let text=source["vectors"].as_array().unwrap().iter().find(|x|x["case"]==name).unwrap()["hex"].as_str().unwrap();
    (0..text.len()).step_by(2).map(|i|u8::from_str_radix(&text[i..i+2],16).unwrap()).collect()
}
#[test]
fn formal_status_vectors_preserve_slots_and_missing_slots_are_unavailable(){
    assert!(parse_availability(&vector("status-empty")).unwrap().slots.is_empty());
    let four=parse_availability(&vector("status-four")).unwrap();
    assert_eq!(four.slots,[false,true,false,true]);
    assert!(!four.available(0)&&four.available(1)&&!four.available(4)&&!four.available(255));
    assert_eq!(parse_availability(&vector("status-ten")).unwrap().slots,vec![true;10]);
    assert_eq!(parse_availability(&vector("configured-page-status")).unwrap().slots,[false,true,true]);
}
#[test]
fn formal_endpoint_vectors_keep_little_endian_ip_and_port(){
    for index in [0,3,9]{let p=parse_endpoint(&vector(&format!("ip-{index}"))).unwrap();assert_eq!(p.index,index);assert_eq!(p.endpoint,format!("127.0.0.2:{}",37301+u16::from(index)).parse().unwrap());}
    // Protocol allows the full unsigned IPv4 range; lifecycle checks are separate.
    assert_eq!(parse_endpoint(&[0xf4,9,255,255,255,255,255,255]).unwrap().endpoint,"255.255.255.255:65535".parse().unwrap());
}
#[test]
fn availability_rejects_every_truncation_trailing_data_and_unknown_state(){
    for name in ["status-empty","status-four","status-ten","configured-page-status"]{
        let valid=vector(name);for end in 0..valid.len(){assert!(parse_availability(&valid[..end]).is_err());}
        let mut trailing=valid.clone();trailing.push(0);assert!(parse_availability(&trailing).is_err());
        let mut wrong=valid;wrong[0]=0xfb;assert!(parse_availability(&wrong).is_err());
    }
    for word in [1u32,0x8000_0000,0xffff_fffe]{let mut b=vec![0xf3,1];b.extend(word.to_le_bytes());assert!(parse_availability(&b).is_err());}
    let mut too_many=vec![0xf3,11];too_many.extend([0;44]);assert!(parse_availability(&too_many).is_err());
}
#[test]
fn endpoint_rejects_every_truncation_trailing_data_and_invalid_index_or_port(){
    let valid=vector("ip-3");for end in 0..valid.len(){assert!(parse_endpoint(&valid[..end]).is_err());}
    let mut b=valid.clone();b.push(0);assert!(parse_endpoint(&b).is_err());
    for index in [10,127,255]{let mut b=valid.clone();b[1]=index;assert!(parse_endpoint(&b).is_err());}
    let mut b=valid.clone();b[6]=0;b[7]=0;assert!(parse_endpoint(&b).is_err());
    let mut b=valid;b[0]=0xf3;assert!(parse_endpoint(&b).is_err());
}

fn rental_directory(peer:std::net::SocketAddr)->lito_modern::modern_lito::native_services::Directory {
    use lito_modern::modern_lito::native_services::{Directory,DirectRoom,VsServer};
    Directory {
        rooms:vec![DirectRoom{name:"same rental".into(),endpoint:peer,id:60001,users:0},
            DirectRoom{name:"other rental".into(),endpoint:"127.0.0.9:4660".parse().unwrap(),id:60002,users:1}],
        servers:vec![VsServer{name:"old page".into(),endpoint:peer,users:1,capacity:100}],
    }
}
fn routed_pending(current:std::net::SocketAddr,target:std::net::SocketAddr)->Pending {
    let mut p=pending();p.status(&owner(),parse_availability(&vector("configured-page-status")).unwrap()).unwrap();
    p.endpoint(&owner(),lito_modern::modern_lito::native_services::VsEndpoint{index:1,endpoint:target},current).unwrap();p
}
#[test]
fn same_endpoint_catalog_rebinds_only_matching_routes_once_after_fresh_game_character(){
    let peer="127.0.0.3:4660".parse().unwrap();let mut p=routed_pending(peer,peer);
    p.remember_same_endpoint_services(peer,Some(&rental_directory(peer)));
    let directory=p.take_same_endpoint_services(peer,4,"測試角色").unwrap();
    assert_eq!(directory.rooms.len(),1);assert_eq!(directory.rooms[0].id,60001);assert_eq!(directory.rooms[0].endpoint,peer);
    assert!(directory.servers.is_empty(),"old VS page availability cannot be reused");
    assert!(p.take_same_endpoint_services(peer,4,"測試角色").is_none());
}
#[test]
fn cross_endpoint_or_port_change_never_carries_source_rental_routes(){
    let peer="127.0.0.3:4660".parse().unwrap();
    for target in ["127.0.0.4:4660","127.0.0.3:4661"] {
        let target=target.parse().unwrap();let mut p=routed_pending(peer,target);
        p.remember_same_endpoint_services(peer,Some(&rental_directory(peer)));
        assert!(p.take_same_endpoint_services(target,4,"測試角色").is_none());
    }
}
#[test]
fn old_or_skipped_connection_and_wrong_character_consume_the_catalog(){
    let peer="127.0.0.3:4660".parse().unwrap();
    for (connection,character) in [(3,"測試角色"),(5,"測試角色"),(4,"其他角色")] {
        let mut p=routed_pending(peer,peer);p.remember_same_endpoint_services(peer,Some(&rental_directory(peer)));
        assert!(p.take_same_endpoint_services(peer,connection,character).is_none());
        assert!(p.take_same_endpoint_services(peer,4,"測試角色").is_none(),"a rejected catalog must not revive");
    }
}
#[test]
fn absent_directory_and_routes_for_another_peer_do_not_invent_rental_service(){
    let peer="127.0.0.3:4660".parse().unwrap();
    for directory in [None,Some(Default::default()),Some(rental_directory("127.0.0.8:4660".parse().unwrap()))] {
        let mut p=routed_pending(peer,peer);p.remember_same_endpoint_services(peer,directory.as_ref());
        assert!(p.take_same_endpoint_services(peer,4,"測試角色").is_none());
    }
}
#[test]
fn changing_the_pending_owner_or_destination_cannot_retarget_a_captured_catalog(){
    let peer="127.0.0.3:4660".parse().unwrap();
    for dimension in 0..3 {
        let mut p=routed_pending(peer,peer);p.remember_same_endpoint_services(peer,Some(&rental_directory(peer)));
        match dimension {0=>p.owner.generation+=1,1=>p.character.push('異'),2=>p.destination=Some("127.0.0.3:4661".parse().unwrap()),_=>unreachable!()}
        assert!(p.take_same_endpoint_services(peer,4,"測試角色").is_none());
    }
}
#[test]
fn remembered_directory_is_an_owned_snapshot_and_connection_overflow_cannot_wrap(){
    let peer="127.0.0.3:4660".parse().unwrap();let mut directory=rental_directory(peer);let mut p=routed_pending(peer,peer);
    p.remember_same_endpoint_services(peer,Some(&directory));directory.rooms[0].id=123;
    assert_eq!(p.take_same_endpoint_services(peer,4,"測試角色").unwrap().rooms[0].id,60001);
    let mut high=owner();high.connection=u64::MAX;high.inventory.connection=u64::MAX;
    let mut p=Pending::new(high.clone(),"測試角色".into(),"TeleportMe","LT_TELEPORT_B").unwrap();
    p.status(&high,parse_availability(&vector("configured-page-status")).unwrap()).unwrap();
    p.endpoint(&high,lito_modern::modern_lito::native_services::VsEndpoint{index:1,endpoint:peer},peer).unwrap();
    p.remember_same_endpoint_services(peer,Some(&rental_directory(peer)));
    assert!(p.take_same_endpoint_services(peer,0,"測試角色").is_none());
}
