from pathlib import Path
P=Path(__file__).resolve().parent.parent
t=(P/'tools/r484-map527-reference.rs').read_text(encoding='utf-8')
t=t.replace('map::parse_s32','map::parse_native_s32').replace('parse_s32(&raw)','parse_native_s32(&raw)').replace('WorldMap::from_sector_bytes','WorldMap::from_native_sector_bytes')
t=t.replace('"Layer6":sector.tile_dependencies.len(),','"Layer6":sector.tile_dependencies.len(),"layer_ranges":sector.layer_ranges.iter().map(|r|r.as_ref().map(|r|[r.start,r.end])).collect::<Vec<_>>(),"portals":sector.portals.iter().map(|p|json!({"name_hex":p.name.iter().map(|b|format!("{b:02x}")).collect::<String>(),"x":p.x,"y":p.y,"target_map":p.target_map,"legacy_fixed_i32":p.portal_id,"native_kind":p.native_kind})).collect::<Vec<_>>(),')
old='for(slot,t)in decoded{pixels.insert((*id,slot as u8),t);}'
new='''for(slot,mut t)in decoded{if matches!(t.tile_type as u8,22|23){let c:Value=serde_json::from_slice(&fs::read(root.parent().unwrap().parent().unwrap().parent().unwrap().join("logs/continuation-20261003/r485-tile-reference.json"))?)?;let name=format!("{id}-{slot}");let r=c["cases"].as_array().unwrap().iter().find(|r|r["name"]==name).context("required cloud tile has no independent C# reference")?;let rgba=bytes(&r["rgba"])?;ensure!(t.rgba==rgba,"new effect differs from independent entire C# source");t.rgba=rgba;}pixels.insert((*id,slot as u8),t);}'''
assert t.count(old)==1;t=t.replace(old,new)
t=t.replace('R483_FIXED_R483_FULL_SOURCE_DECODER_INDEPENDENT_FULL_VECTOR_CITY_REFERENCE_PASS','R485_ALL_COMPLETE_NATIVE_SOURCE_FULL_VECTOR_CSHARP_NEW_TILE_REFERENCE_PASS')
t=t.replace('let estimated_decoded_bytes=pixels.values().map(|t|t.rgba.len()).sum::<usize>();','let estimated_decoded_bytes=pixels.values().map(|t|t.rgba.len()).sum::<usize>();ensure!(estimated_decoded_bytes<=67108864 && m["source_bytes"].as_u64().unwrap()<=33554432,"unchanged byte budgets");')
(P/'tools/r485-complete-map-reference.rs').write_text(t,encoding='utf-8')
print('R485 separate full-vector reference prepared; C#12 references supply all new effect pixels')
