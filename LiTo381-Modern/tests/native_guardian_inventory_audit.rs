use lito_modern::modern_lito::native_inventory::Inventory;
#[test]
#[ignore="passive read of one owned guardian capture"]
fn inspect_owned_guardian_weapon_inventory(){
 let root=std::path::PathBuf::from(std::env::var("LITO_GUARDIAN_AUDIT_ROOT").unwrap());assert!(root.canonicalize().unwrap().starts_with(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap()));
 let captures=root.join("captures");let folder=std::fs::read_dir(captures).unwrap().next().unwrap().unwrap().path();let text=std::fs::read_to_string(folder.join("frames.jsonl")).unwrap();
 let mut batches=vec![];
 for line in text.lines(){let row:serde_json::Value=serde_json::from_str(line).unwrap();if row["direction"]!="server_to_client"||row["opcode"]!="0x7C"{continue}
  let hex=row["plain"].as_str().unwrap().replace(' ',"");let bytes=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect::<Vec<_>>();let mut inv=Inventory::default();inv.begin(2,"capture-audit").unwrap();inv.confirm_selection();inv.accept(2,&bytes).unwrap();
  batches.push(serde_json::json!({"weapons":inv.items.values().filter(|i|i.use_type==1).collect::<Vec<_>>()}));
 }
 std::fs::write(root.join("weapon-inventory-passive-audit.json"),serde_json::to_vec_pretty(&batches).unwrap()).unwrap();
}
