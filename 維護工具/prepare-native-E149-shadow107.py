"""Prepare Shadow Fang from pinned formal item-spell and original list bytes."""
from pathlib import Path
from datetime import datetime,timezone
import json,re,hashlib
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def prepare():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'workload/freeze active');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'deferral/target changed')
 ref=w['current_E149_magical73_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'previous skill qualification changed');q=read(ref['qualification']);check_pins(q['source_pins']);prior=Path(q['candidate']);require(sha(prior)==q['candidate_sha256'],'previous skill candidate changed');base=read(prior)
 files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);fref=w['current_E149_fixture_binding'];require(sha(fref['qualification'])==fref['qualification_sha256'],'current fixture binding changed');binding=read(fref['qualification'])
 for n,h in binding['current_effective_overlay_files'].items():
  if n.endswith('.java'):files[n]=h;paths[n]=binding['current_effective_overlay_paths'][n]
 require(len(files)==20,'current skill/fixture projection differs');check_pins({paths[n]:h for n,h in files.items()})
 api=V/'native-E149-item-spell-contract/20261024-014418-307290/inspection.json';inspection=read(api);require(inspection['formal_target_sha256']==FORMAL and all(x['exit_code']==0 for x in inspection['commands']),'original item-spell inspection failed');check_pins(inspection['outputs'])
 action=api.parent/'NativeItemWeaponCastAction.stdout.txt';metadata=api.parent/'BattleSpellCatalog$ItemWeaponEnchantSpell.stdout.txt'
 require('bipush        107' in action.read_text('utf-8-sig') and 'ClientSession.nativeMaterialItemWeaponEnchantBody:' in action.read_text('utf-8-sig'),'original107 action bridge differs')
 catalog=V/'native-E149-slow29-advance79-contract/20261024-002909-204866/BattleSpellCatalog.stdout.txt';client=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';require(sha(client)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','original ClientSession changed')
 original_list=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(original_list)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','original list namespace/content differs');raw=original_list.read_bytes();rows=raw.splitlines();start=next(i for i,row in enumerate(rows) if re.match(rb'#2951(?:\D|$)',row));end=next(i for i in range(start+1,len(rows)) if rows[i].startswith(b'#'));block=rows[start:end];require(block[0]==b'#2951 1 shadow fang','original2951 header differs')
 frames=[(int(a),int(b),int(t),[] if not sound else [int(sound)]) for a,b,t,sound in re.findall(rb'(\d+)[.](\d+):(\d+)(?:\[(\d+))?',b'\n'.join(block[1:]))]
 require(len(frames)==13 and [f[1] for f in frames]==[12,*range(12)] and [f[2] for f in frames]==[4,2,2,2,2,3,6,5,4,4,4,4,4] and [f[3] for f in frames]==[[],[1510],*([[]]*11)],'original2951 cadence/events differ')
 attrs=[(int(k),body) for row in block[1:] for k,body in re.findall(rb'^\s*(\d+)[.](?:type|attr)\(([^)]*)\)\s*$',row)];require(attrs==[(102,b'0'),(104,b'8')],'original2951 attributes differ')
 static_manifest=Path(w['current_E149_owned_mysql_development']['static_manifest']);static=read(static_manifest);sql=Path(static['SQL']);material_rows=[line for line in sql.read_text('utf-8-sig').splitlines() if re.match(r"INSERT INTO `etcitem` VALUES \(NULL, 40321,",line)];require(len(material_rows)==1 and "'二級黑魔石'" in material_rows[0],'material40321 name differs')
 out=owned(T/'candidates'/('native-E149-shadow107-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir(parents=True)
 baseline=dict(schema='E149-shadow107-base-overlay-v1',candidate_files=files,candidate_paths=paths,previous_skill_qualification=ref,current_fixture_binding=fref);(out/'base-overlay.json').write_bytes(encoded(baseline))
 def nums(values):return '['+','.join(str(n) for n in values)+']'
 frame_source=','.join('('+','.join([str(a),str(b),str(t),'&'+nums(sounds)])+')' for a,b,t,sounds in frames)
 spec='Spec{sprite:2951,bank:2951,header:&'+nums(block[0])+',attrs:&[(102,&[48]),(104,&[56])],frames:&['+frame_source+']},\n'
 changes={
  'LiTo381-Modern/src/modern_lito/native_skills.rs':[('matches!(skill,12|21|73)','matches!(skill,12|21|73|107)'),('60|73=>0','60|73|107=>0')],
  'LiTo381-Modern/src/modern_lito/native_skill_selection.rs':[('                        73 => "#73 創造魔法武器（選取背包武器；消耗 10 顆魔法寶石）".into(),','                        73 => "#73 創造魔法武器（選取背包武器；消耗 10 顆魔法寶石）".into(),\n                        107 => "#107 暗影之牙（選取背包武器；消耗 1 顆二級黑魔石）".into(),')],
  'LiTo381-Modern/src/modern_lito/native_additional_finite_effects.rs':[('6342,752,747,748];','6342,752,747,748,2951];'),('const SPECS:&[Spec]=&[\n','const SPECS:&[Spec]=&[\n'+spec)],
  'LiTo381-Modern/tests/native_item_spells.rs':[('learned_skills.extend([12,21,73])','learned_skills.extend([12,21,73,107])'),('[(12,1,3),(21,2,4),(73,9,0)]','[(12,1,3),(21,2,4),(73,9,0),(107,13,2)]'),('if id==73{0}else{10}','if matches!(id,73|107){0}else{10}')],
 }
 changed={};before={}
 for n,replacements in changes.items():
  source=Path(paths[n]);text=source.read_text('utf-8-sig');before[n]=sha(source)
  for old,new in replacements:require(text.count(old)==1,'source anchor differs: '+old);text=text.replace(old,new)
  p=out/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(text,encoding='utf-8',newline='\n');changed[n]=sha(p)
 n='LiTo381-Modern/tests/native_shadow107.rs';p=out/n;p.write_bytes((T/'templates/native_shadow107.rs').read_bytes());changed[n]=sha(p)
 for n,h in changed.items():files[n]=h;paths[n]=str(out/n)
 contract=dict(schema='E149-shadow107-source-contract-v1',formal_target_sha256=FORMAL,skill=107,row=13,column=2,action=19,effect=2951,MP_base=30,reuse_millis=0,server_duration_seconds=192,source_type=2,native_id=4,melee_damage=5,ranged_damage=5,material_id=40321,material_count=1,material_name='二級黑魔石',invalid_item_message=79,success_message=0,expiration_message=308,
  original_header=list(block[0]),original_block=[list(row) for row in block],frames=frames,attrs=[(k,list(body)) for k,body in attrs],audio_sound=1510,audio_mode='SuppressWhileSameIdPlaying',client_predicts_cost_or_item_effect=False,
  original_pins={str(p):sha(p) for p in [api,action,metadata,catalog,client,original_list,static_manifest,sql]},tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract));record=dict(schema='E149-shadow107-candidate-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,previous_skill_qualification=ref,current_fixture_binding=fref,base_overlay=str(out/'base-overlay.json'),base_overlay_sha256=sha(out/'base-overlay.json'),base_files=before,candidate_files=changed,effective_overlay_files=files,effective_overlay_paths=paths,original_metadata_cohort=str(Path(q['compile_run'])/'cohort.json'),changed_runtime_files=[n for n in changes if '/src/' in n],contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),new_tests_written=6,retained_skill_tests_written=31,tests_executed=0,production_applied=False,canonical_registry_selected=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_files=5,effective_files=21,new_cases=6,tests_executed=0),ensure_ascii=False))

if __name__=='__main__':prepare()
