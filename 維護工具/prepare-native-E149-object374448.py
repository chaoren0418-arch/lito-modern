"""Prepare actor-target remove-curse, cancellation and bless-weapon sources."""
from pathlib import Path
from datetime import datetime,timezone
import re,hashlib,json
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'
API=V/'native-E149-object374448-contract/20261024-033519-068975/inspection.json'

def prepare():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'workload/freeze active');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'deferral/formal changed')
 ref=w['current_E149_shadow107_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'previous skill qualification changed');iq=read(ref['qualification']);check_pins(iq['source_pins']);cp=Path(iq['candidate']);require(sha(cp)==iq['candidate_sha256'],'previous candidate changed');base=read(cp)
 api=read(API);require(api['formal_target_sha256']==FORMAL and api['game_classes_instantiated']==0 and all(x['exit_code']==0 for x in api['commands']),'original API failed');check_pins(api['outputs'])
 client=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';require(sha(client)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','original ClientSession changed');catalog=V/'native-E149-slow29-advance79-contract/20261024-002909-204866/BattleSpellCatalog.stdout.txt'
 raw_path=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(raw_path)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','original list differs');rows=raw_path.read_bytes().splitlines();effects=[];specs=[]
 for gfx,bank,sound in [(227,227,122),(870,232,140),(2176,2176,800)]:
  start=next(i for i,row in enumerate(rows) if re.match(rb'#'+str(gfx).encode()+rb'(?:\D|$)',row));end=next(i for i in range(start+1,len(rows)) if rows[i].startswith(b'#'));block=rows[start:end];body=b'\n'.join(block)
  frames=[(int(a),int(b),int(t),[] if not s else [int(s)]) for a,b,t,s in re.findall(rb'(\d+)[.](\d+):(\d+)(?:\[(\d+))?',body)];attrs=[(int(k),list(v)) for k,v in re.findall(rb'(\d+)[.](?:type|attr)\(([^)]*)\)',body)]
  require([f[3] for f in frames]==[[],[sound],*([[]]*(len(frames)-2))],'original sound placement differs')
  require([f[2] for f in frames]==([4]*17 if gfx==2176 else [4,2,2,4,4,2,2]) and [f[1] for f in frames]==([16,*range(16)] if gfx==2176 else list(range(7))),'original effect cadence differs')
  require(attrs==([(102,[48]),(104,[56])] if gfx==2176 else [(104,[56])]),'original attributes differ')
  effects.append(dict(graphics=gfx,bank=bank,sound=sound,header=list(block[0]),frames=frames,attributes=attrs))
  if gfx!=2176:
   nums=lambda values:'['+','.join(str(v) for v in values)+']'
   f=','.join('('+','.join([str(a),str(b),str(t),'&'+nums(s)])+')' for a,b,t,s in frames)
   specs.append('Spec{sprite:'+str(gfx)+',bank:'+str(bank)+',header:&'+nums(block[0])+',attrs:&[(104,&[56])],frames:&['+f+']},\n')
 files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);require(len(files)==21,'previous projection differs')
 fref=w['current_E149_fixture_binding'];require(sha(fref['qualification'])==fref['qualification_sha256'],'fixture binding changed');binding=read(fref['qualification'])
 for n,h in binding['current_effective_overlay_files'].items():
  if n.endswith('.java'):require(files[n]==h and paths[n]==binding['current_effective_overlay_paths'][n],'current Java context differs')
 check_pins({paths[n]:h for n,h in files.items()});out=owned(T/'candidates'/('native-E149-object374448-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir(parents=True)
 (out/'base-overlay.json').write_bytes(encoded(dict(candidate_files=files,candidate_paths=paths,previous_skill_qualification=ref,current_fixture_binding=fref)))
 source=client.read_text('utf-8-sig');heads=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',source,re.M));selected={}
 for name in ['useWeaponBuffSpell','sourcePoisonBody$useCurseRemovalSpell','sourcePoisonBody$useCancellationSpell','nativeMaterialWorldCancellationBody']:
  matches=[i for i,m in enumerate(heads) if name+'(' in m[0]];require(len(matches)==1,'original source method ambiguous');i=matches[0];selected[name]=source[heads[i].start():heads[i+1].start()]
 require('Protocol.objectSpellRequest:' in selected['useWeaponBuffSpell'] and 'Protocol.objectSpellRequest:' in selected['sourcePoisonBody$useCurseRemovalSpell'] and 'BattleSpellCatalog.isInNativeScreen:' in selected['useWeaponBuffSpell'] and 'BattleSpellCatalog.isInNativeScreen:' in selected['sourcePoisonBody$useCurseRemovalSpell'],'original actor/screen request differs')
 require('NativeCancellationMaterialAction.battle:' in selected['sourcePoisonBody$useCancellationSpell'] and 'CancellationSpell.maximumRange:' in selected['nativeMaterialWorldCancellationBody'] and 'java/lang/Math.max:(II)I' in selected['nativeMaterialWorldCancellationBody'],'original cancellation route/range differs')
 contract=dict(schema='E149-object374448-source-contract-v1',formal_target_sha256=FORMAL,skills={37:dict(row=4,column=4,graphics=227,MP_base=20,reuse_millis=10,range='native_screen',cures_curse_paralysis=True),44:dict(row=5,column=3,graphics=870,MP_base=40,reuse_millis=500,range=3,material=40318,material_count=2,failure_message=280),48:dict(row=5,column=7,graphics=2176,MP_base=20,reuse_millis=10,range='native_screen',duration_seconds=1200,melee_hit=2,melee_damage=2,ranged_hit=2,ranged_damage=0,missing_weapon=79,success_message=161)},effects=effects,original_methods=selected,
  original_pins={str(p):sha(p) for p in [API,client,catalog,raw_path,*[Path(p) for p in api['outputs']]]},costs_cures_cancellation_and_weapon_stats_server_owned=True,tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract))
 changes={
  'LiTo381-Modern/src/modern_lito/native_skills.rs':[
   ('1 | 8 | 9 | 19 | 26 | 35 | 42 | 43 | 57 | 79 =>','1 | 8 | 9 | 19 | 26 | 35 | 37 | 42 | 43 | 44 | 48 | 57 | 79 =>'),
   ('&& if matches!(skill, 8 | 9 | 26 | 42 | 43 | 79) {','&& if skill==44 {own.x.abs_diff(actor.x).max(own.y.abs_diff(actor.y))<=3} else if matches!(skill, 8 | 9 | 26 | 37 | 42 | 43 | 48 | 79) {'),
   ('20|29|49|57|72=>500','20|29|44|49|57|72=>500'),
   ('伺服器拒絕此物品附魔','伺服器拒絕此物品或武器附魔'),
   ('/// Invalid-item79 is only a response in the inventory-spell domain.','/// Code79 is scoped to an inventory or actor weapon-buff pending request.'),
   ('pub fn observe_item(&self,world:&World,now:Instant)->Feedback {','pub fn observe_item(&self,world:&World,now:Instant)->Feedback {self.observe_equipment(world,now)}\n    pub fn observe_equipment(&self,world:&World,now:Instant)->Feedback {')],
  'LiTo381-Modern/src/modern_lito/native_skill_selection.rs':[
   ('                        107 => "#107 暗影之牙（選取背包武器；消耗 1 顆二級黑魔石）".into(),','                        107 => "#107 暗影之牙（選取背包武器；消耗 1 顆二級黑魔石）".into(),\n                        37 => "#37 解除詛咒（選取目標／自己）".into(),\n                        44 => "#44 相消術（選取目標／自己，三格內；消耗 2 顆魔法寶石）".into(),\n                        48 => "#48 祝福魔法武器（選取目標／自己）".into(),')],
  'LiTo381-Modern/src/modern_lito/native_additional_finite_effects.rs':[
   ('6342,752,747,748,2951];','6342,752,747,748,2951,227,870];'),('[(6342,751)];','[(6342,751),(870,232)];'),('const SPECS:&[Spec]=&[\n','const SPECS:&[Spec]=&[\n'+''.join(specs))],
  'LiTo381-Modern/src/modern_lito/native_login.rs':[]}
 changed={};before={}
 for n,replacements in changes.items():
  raw=Path(paths[n]);text=raw.read_text('utf-8-sig');before[n]=sha(raw)
  if n.endswith('/native_login.rs'):
   require(text.count('skill_item_pending')==10,'original pending state/reset count differs');text=text.replace('skill_item_pending','skill_equipment_pending');text=text.replace('self.skill_equipment_pending=item.is_some();','self.skill_equipment_pending=item.is_some()||matches!(skill,8|48);').replace('pending.observe_item(&self.state.world,Instant::now())','pending.observe_equipment(&self.state.world,Instant::now())')
  for old,new in replacements:require(text.count(old)==1,'source anchor differs: '+old);text=text.replace(old,new)
  p=out/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(text,encoding='utf-8',newline='\n');changed[n]=sha(p)
 n='LiTo381-Modern/tests/native_object374448.rs';p=out/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes((T/'templates/native_object374448.rs').read_bytes());changed[n]=sha(p)
 for n,h in changed.items():files[n]=h;paths[n]=str(out/n)
 record=dict(base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate_files=changed,base_files=before,base_overlay=str(out/'base-overlay.json'),base_overlay_sha256=sha(out/'base-overlay.json'),previous_skill_qualification=ref,current_fixture_binding=fref,effective_overlay_files=files,effective_overlay_paths=paths,original_metadata_cohort=str(Path(iq['compile_run'])/'cohort.json'),changed_runtime_files=list(changes),contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),new_tests_written=9,retained_skill_tests_written=37,tests_executed=0,production_applied=False,canonical_registry_selected=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_files=5,runtime_changes=4,effective_files=22,new_cases=9,tests_executed=0),ensure_ascii=False))

if __name__=='__main__':prepare()
