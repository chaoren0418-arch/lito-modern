from pathlib import Path
import json,hashlib
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r485-format-study'
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def save(p,v):t=p.with_suffix('.json.tmp');t.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8');t.replace(p)
a=read(L/'r485-resource-input.json');before=read(O/'reference/reference.json');budget_raw=read(O/'budget-reference/reference.json');budget=budget_raw['results'][0];native=read(L/'r485-native-layer-reference.json');old=read(L/'r484-map-binding.json')
assert (before['passed'],before['failed'])==(2,1);assert before['results'][2]['error']=='unchanged byte budgets'
assert budget['state']=='R485_COMPLETE_SOURCE_FORMAT_MEASURED_DECODE_BUDGET_REFUSAL_NO_PREVIEW'and budget['required_decoded_bytes']==72446976 and budget['required_slots']==31444 and not budget['member_failures'];assert budget['draw_references']==286665
results=before['results'][:2]
for result in results+[budget]:
 for s in result['sector_stats']:
  row=next(r for r in native['sectors']if r['map_id']==result['map_id']and Path(r['sector']['path']).stem==f"{s['x']:04x}{s['y']:04x}")['native_reference']
  assert [r for r in s['layer_ranges']if r is not None]==row['ranges'];assert s['effects_retained_not_played']==row['effects']
  assert s['portals']==[{k:p[k]for k in ['name_hex','x','y','target_map','legacy_fixed_i32','native_kind']}for p in row['portals']]
save(O/'reference-combined.json',dict(state='R485_TWO_COMPLETE_NEW_MAP_REFERENCES_AND563_ALL_SUPPORTED_MEMBER_BUDGET_REFUSAL',results=results,passed=2,measured_not_qualified=1,failed=0,map563_budget=budget,first_probe=pin(O/'reference/reference.json'),budget_probe=pin(O/'budget-reference/reference.json'),raw_budget_probe_summary_does_not_classify_new_measurement_state=True,all29_native_layer_ranges_portals_effects_match_independent_pointer_reference=True,GUI=0,LOADED=False))
new=[{k:b[k]for k in ['mode','map_id','namespace','map_root','archive_root','sectors','archives']}|dict(canonical_sha256=r['canonical_sha256'])for b,r in zip(a['batch'][:2],results)]
binding=old|dict(state='R485_THIRTY_NINE_WHOLE_MAP_PLANS_TWO_NEW_CPU_PREVIEWS_PENDING',bindings=old['bindings']+new,unqualified_count=1,remaining_unqualified_maps=1,new_complete_multisector_ids=[506,573],multisector_probe=pin(O/'reference-combined.json'),multi_sector_scope='5066sectors and5734sectors complete formats; Native portal100..102 suffix and observed22/23 cloud-before-half priority only.56319sectors format complete/all31444slots individually supported but72446976decoded bytes exceeds unchanged67108864limit. No partial or cropped preview.',measured_unqualified_city_ids=[563],known_format_gaps=[],remaining_preview_gap='563 Native decoded byte budget exceeded',previous37_plans_unchanged=True)
save(L/'r485-map-binding.json',binding)
b=a['batch'][2];negative=binding|dict(state='R485_INTENTIONAL_FULL563_NEGATIVE_BUDGET_PLAN_NOT_PLAYER_QUALIFICATION',bindings=binding['bindings']+[{k:b[k]for k in ['mode','map_id','namespace','map_root','archive_root','sectors','archives']}|dict(canonical_sha256=budget['canonical_sha256'])]);save(O/'budget563-fixture.json',negative)
t=(P/'tools/prepare-r484-candidate.py').read_text(encoding='utf-8').replace('R484','R485').replace('r484','r485').replace("==37","==39")
t=t.replace("record['UI_assets']='R48537whole-map plans pending new13map consumer; existing24 bindings unchanged;12new regular and52718sector generic catalog-bound row API. Existing shared runtime/window/menu reused. Only50622/23 and573563Layer7 remain unqualified. All budgets unchanged; GUI pending.'","record['UI_assets']='R48539whole-map plans pending new2map consumer; existing37 plans unchanged;5066sector and5734sector whole previews using existing worker/window.563 full19sector format passes but required72446976decoded bytes exceeds unchanged64MiB budget. Native-only portal suffix; observed22/23 uses existing cloud RGBA adaptation. All budgets preserved; GUI pending.'")
(P/'tools/prepare-r485-candidate.py').write_text(t,encoding='utf-8')
t=(P/'tools/build-r484-player-candidate.ps1').read_text(encoding='utf-8').replace('R484','R485').replace('r484','r485');(P/'tools/build-r485-player-candidate.ps1').write_text(t,encoding='utf-8')
print('R48539 plans:old37 unchanged +506/573.563 exact full-source negative plan prepared; no budget expansion.')
