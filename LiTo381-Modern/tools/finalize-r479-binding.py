from pathlib import Path
import json,hashlib
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r479-map-batch'
def pin(p):
 b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8')
input=read(L/'r479-resource-inventory-input.json');raw=read(L/'r479-native38-inventory.json');probe=read(O/'probe/batch-results.json');old=read(L/'r477-multi-map-binding.json')
assert probe['passed']==4 and probe['failed']==0
new=[]
for result,b in zip(probe['results'],input['batch']):
 assert result['map_id']==b['map_id'] and result['mode']==b['mode'] and result['all_complete_source_static_slots_decoded']
 for s in b['sectors']:assert pin(Path(s['pin']['path']))['sha256']==s['pin']['sha256']
 new.append({k:b[k] for k in ['mode','map_id','map_root','archive_root','sectors','archives']}|dict(namespace='NATIVE_LITO_LTMAP_V1',canonical_sha256=result['canonical_sha256']))
binding={**old,'state':'R479_CATALOG40_SIX_TRUE_NATIVE_SINGLE_SECTOR_PREVIEW_BINDINGS','bindings':old['bindings']+new,'new_complete_single_sector_ids':[b['map_id'] for b in new],'new_map_probe':pin(O/'probe/batch-results.json'),'remaining38_inventory':pin(L/'r479-native38-inventory.json'),'remaining_unqualified_maps':34,'camera_adaptation':'Existing Modern inspection camera1136,468,800x600 relative to actual selected sector; no native spawn or wire admission claim','multi_sector_scope':'All four added maps have exactly one complete sector in native directory/R474 pins. No partial map selection or invented seam. Multi-sector preview remains next work for34 maps.'}
save(L/'r479-map-binding.json',binding)
failures=[]
for r in raw['rows']:
 if r['parse_or_static_reference_error']:
  origin=next(x for x in input['rows'] if (x['mode'],x['map_id'])==(r['mode'],r['map_id']));failed=origin['sectors'][len(r['parsed_sectors'])];failures.append(dict(mode=r['mode'],map_id=r['map_id'],source_sector=failed,error=r['parse_or_static_reference_error'],scope='Existing supported S32 decoder rejects this actual Layer7 input. Original corruption is not inferred; needs original format/variant investigation, remains unqualified.'))
audit=dict(state='R479_REMAINING38_RESOURCE_COMPLETENESS_AND_BATCH_DECISION_RECORDED',input=pin(L/'r479-resource-inventory-input.json'),raw_inventory=pin(L/'r479-native38-inventory.json'),fixed_library_probe=pin(L/'r479-probe-candidate.json'),rows=38,source_sectors=305,missing_original_sectors=0,static_tile_index_missing_members_rows=0,fully_parsed_rows=36,decoder_rejected_rows=failures,selected_new_maps=[b['map_id'] for b in new],selected_complete_source_sector_count=4,qualified_preview_maps=6,remaining_unqualified_maps=34,remaining_multi_sector_maps=34,why_no_new_seam_code='Selected finite batch includes all four remaining complete single-sector mode1 maps. Existing geometry/Native adapter already accepts multi-sector plans; player flow is deliberately single-sector. No seam work necessary for this whole-map batch;34 remaining maps still require exact full-sector flow, camera bounds and tests.',static_gaps='Source effects/opacity/Layer6 retained but no new runtime action/roof/actor/mode grant. Every floor/overlay/object tile slot decoded for4 selected maps; other map index presence is not a decoding/playability grant.',next_required_player_flow=['Qualify exact complete multi-sector plans and adapt player flow, sector anchor camera and boundary tests for remaining34 maps','Investigate actual Layer7 variants of mode3/map573 and563 without guessing source damage','Visible GUI/DPI/focus/load/close acceptance','Source runtime flags/mode packet trigger, thumb drag/hold-repeat where proven','Native spawn/network/mode rules/actors/effects/roof/loading admission'],GUI=0,LOADED=False,complete_modes=16,total_modes=40)
save(L/'r479-resource-completeness-audit.json',audit)
print('R479 four additional exact complete mode1 single-sector bindings ready; qualified6, pending34')
