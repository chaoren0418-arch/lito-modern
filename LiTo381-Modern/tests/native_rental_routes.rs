use lito_modern::modern_lito::{world_map::{SectorInput,WorldMap},resources::SharedMapSectorKey,native_navigation::Navigation,native_world::destination};
use std::{path::Path,sync::atomic::AtomicBool};
#[test]
fn formal_rental_npcs_are_reachable_on_original_geometry_without_crossing_actors(){
    let root=Path::new(r"D:\lineage開發\0-客戶端\LiTo\LTmap\500");let mut files:Vec<_>=std::fs::read_dir(root).unwrap().map(|e|e.unwrap().path()).filter(|p|p.extension().is_some_and(|x|x=="s32")).collect();files.sort();
    let inputs=files.iter().map(|p|{let name=p.file_stem().unwrap().to_str().unwrap();SectorInput{key:SharedMapSectorKey{map_id:500,sector_x:u16::from_str_radix(&name[..4],16).unwrap(),sector_y:u16::from_str_radix(&name[4..],16).unwrap()},bytes:std::fs::read(p).unwrap()}}).collect();
    let map=WorldMap::from_native_sector_bytes(inputs,&AtomicBool::new(false)).unwrap();let nav=Navigation::from_geometry(&map).unwrap();
    let fixture:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-rental-npcs-current.json")).unwrap();let npcs=fixture["npcs"].as_array().unwrap();assert_eq!(npcs.len(),13);
    let blocked=|x:u16,y:u16|[x,y]==[32770,32818]||npcs.iter().any(|n|n["x"]==u64::from(x)&&n["y"]==u64::from(y));
    let mut position=[32770u16,32818u16];let mut rows=vec![];
    for resource in [1,2,3,4,0,10,11,12,91,92,93,94,95]{
        let n=npcs.iter().find(|n|n["resource"]==resource).unwrap();let target=[n["x"].as_u64().unwrap() as u16,n["y"].as_u64().unwrap() as u16];
        let path=nav.route_to_adjacent_avoiding(position,target,blocked).unwrap_or_else(|e|panic!("NPC {resource}: {e}"));let start=position;let mut steps=vec![];
        for heading in path{assert!(nav.can_step(i32::from(position[0]),i32::from(position[1]),heading));let (x,y)=destination(position[0],position[1],heading).unwrap();assert!(!blocked(x,y));
            steps.push(serde_json::json!({"heading":heading,"key":([0x68,0x69,0x66,0x63,0x62,0x61,0x64,0x67][heading as usize]),"before":position,"after":[x,y]}));position=[x,y];}
        assert_eq!(position[0].abs_diff(target[0]).max(position[1].abs_diff(target[1])),1);
        rows.push(serde_json::json!({"resource":resource,"npc":n,"start":start,"end":position,"steps":steps}));
    }
    let output=Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/rental-npc-routes.json");std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    let result=serde_json::json!({"map":500,"canonical_sha256":map.canonical_digest(),"jar_sha256":fixture["jar_sha256"],"routes":rows,"failed_manual_edge_passable":nav.can_step(32768,32820,6),"scope":"Static original-geometry route plan; live server movement acceptance remains separate"});
    std::fs::write(output,serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}
