from pathlib import Path
import json
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r485-format-study'
a=json.loads((L/'r485-resource-input.json').read_text(encoding='utf-8'));a['batch']=[r for r in a['batch']if r['map_id']==563]
(L/'r485-budget-input.json').write_text(json.dumps(a,ensure_ascii=False,indent=2),encoding='utf-8');(O/'budget-reference').mkdir(exist_ok=False)
t=(P/'tools/r485-complete-map-reference.rs').read_text(encoding='utf-8')
marker=' let mut pixels=BTreeMap::new();'
insert=''' let needed_slots=required.values().map(|s|s.len()).sum::<usize>();if needed_slots*2304>67108864{let mut supported=0;let mut encoded=0;let mut failures=Vec::new();for(id,slots)in &required{let &(off,len)=index.get(&format!("{id}.til")).context("required member absent")?;encoded+=len;match decode_classic_til_selected_with_empty_tail(&pak[off..off+len],TileLimits::default(),slots){Ok(v)=>{supported+=v.len();drop(v);},Err(e)=>failures.push(json!({"member":id,"error":format!("{e}"),"slots":slots}))}}return Ok(json!({"state":"R485_COMPLETE_SOURCE_FORMAT_MEASURED_DECODE_BUDGET_REFUSAL_NO_PREVIEW","mode":m["mode"],"map_id":id,"sectors":sectors.len(),"canonical_sha256":canonical.iter().map(|b|format!("{b:02X}")).collect::<String>(),"bounds":bounds,"sector_stats":stats,"required_slots":needed_slots,"required_decoded_bytes":needed_slots*2304,"decoded_limit":67108864,"draw_references":draws.len(),"tile_members":required.len(),"encoded_required_member_bytes":encoded,"individually_supported_slots":supported,"member_failures":failures,"only_one_member_output_retained_at_once":true,"full_static_preview_qualified":false,"GUI":0,"LOADED":false}));}
'''
assert t.count(marker)==1;t=t.replace(marker,insert+marker)
(P/'tools/r485-map563-budget-reference.rs').write_text(t,encoding='utf-8')
old=(P/'tools/build-r485-map-reference.ps1').read_text(encoding='utf-8')
old=old.replace("[IO.File]::Copy((Artifact 'lito_modern'),$lib,$false)","if(-not(Test-Path -LiteralPath $lib)){throw 'Fixed R485 library absent'}")
old=old.replace('r485-complete-map-reference','r485-map563-budget-reference').replace('r485_complete_map_reference','r485_map563_budget_reference').replace('r485-reference-','r485-budget-reference-').replace('r485-reference.','r485-budget-reference.').replace("r485-resource-input.json","r485-budget-input.json").replace("'reference\\reference.json'","'budget-reference\\reference.json'")
(P/'tools/build-r485-budget-reference.ps1').write_text(old,encoding='utf-8')
print('563 budget-only probe prepared; no repeated506/573 PASS views')
