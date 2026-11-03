import datetime,hashlib,json,pathlib,re,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';initial=p/'target/r540-native-pet-race-start-component';t=p/'target/r540-native-pet-race-start-component-final';inputs=p/'target/r540-native-pet-race-start-inputs'
def read(f):return json.loads(f.read_text(encoding='utf-8-sig'))
def pin(f):
 h=hashlib.sha256()
 with f.open('rb') as s:
  for b in iter(lambda:s.read(1024*1024),b''):h.update(b)
 return {'path':str(f),'bytes':f.stat().st_size,'sha256':h.hexdigest().upper()}
def save(f,v):
 with f.open('x',encoding='utf-8') as s:json.dump(v,s,ensure_ascii=False,indent=2);s.write('\n')
root=l/'r540-native-start-clock-roster-component-handoff.json';assert not root.exists()
prior=read(l/'r539-result-own-key-wire-consumer-handoff.json');previous=dict(prior['preservation']['prior_roots_checked']);previous['r539-result-own-key-wire-consumer-handoff.json']='1AB2356A377CCDAD19E4F70D73256237B4F9731380C258FF6EE297ADEA79555A'
for n,h in previous.items():assert pin(l/n)['sha256']==h
adoption=read(l/'r540-start-source-adoption.json');assert (adoption['root_refs_checked'],adoption['manifest_files_checked'],adoption['supplement_refs_checked'],adoption['normalized_method_pairs'])==(15,1027,18,4)
blocker=read(l/'r540-full-start-blocker-adoption.json');assert (blocker['refs_checked'],blocker['manifest_files_checked'])==(12,18)
for d in [adoption['root'],adoption['supplement'],blocker['root']]:assert pin(pathlib.Path(d['path']))==d
out=(l/'r540-final-targeted-tests.stdout.log').read_text(encoding='utf-8-sig');match=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; .*?(\d+) filtered out; finished in ([0-9.]+)s',out);assert match and match.group(1,2)==('3','0'),out
err=(l/'r540-final-targeted-tests.stderr.log').read_text(encoding='utf-8-sig');exepath=re.search(r'Running unittests .*?\(([^\r\n]+\.exe)\)',err).group(1);exe=t/'r540-focused-tests.exe';shutil.copyfile(p/exepath,exe)
source=read(l/'r540-final-production-source-snapshot.json')
for d in source['files']:assert pin(p/d['path'])['sha256']==d['sha256'] and pin(t/'source'/d['path'])['sha256']==d['sha256']
oldsource=read(l/'r540-production-source-snapshot.json')
for d in oldsource['files']:assert pin(initial/'source'/d['path'])['sha256']==d['sha256']
rows=[json.loads(s) for s in (l/'r540-final-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()];warnings=[r['message'] for r in rows if r.get('reason')=='compiler-message' and r['message']['level']=='warning']
changed=[n.replace('modern_lito/','') for n in source['existing_sources_changed']+source['new_sources'] if n!='modern_lito/mod.rs'];assert not [w for w in warnings if any(any(n in s['file_name'] for n in changed) for s in w['spans'])];assert not any(n+':' in err for n in changed)
assert read(l/'r540-library-build-accounting.json')['exit_code']==101
assert read(l/'r540-final-library-build-accounting.json')['exit_code']==0 and read(l/'r540-final-targeted-tests-accounting.json')['exit_code']==0
keyed=read(l/'r540-start-keyed-component-check.json');clock=read(l/'r540-clock-domain-and-untagged-check.json');retirement=read(l/'r540-start-boundary-retirement-check.json')
assert (keyed['streams'],keyed['source_start_bodies'],keyed['source_teardown_bodies'],keyed['source_READY_commands'],keyed['segmented_merged_patterns'])==(3,36,6,3,12)
assert len(keyed['checks'])==12 and len(clock['untagged'])==3 and retirement['duplicate_READY_alias_does_not_complete_roster']
classes=read(inputs/'source_class_audit.json')['comparisons'];assert len(classes)==183 and sum(x['byte_equal'] for x in classes)==180
assert {x['class'] for x in classes if not x['byte_equal']}=={'local/lito381/server/BattleRules.class','local/lito381/server/BattleRules$MatchState.class','local/lito381/server/BattleRules$1.class'}
assert next(x for x in classes if x['class']=='local/lito381/server/Protocol.class')['byte_equal']
scope={
 'state':'EXACT_5350_ROOM_COMPONENT_REPLAY_ADOPTED_WORLD_CLIENT_PHASE_AND_LOCAL_CLOCK_NOT_GRANTED',
 'version_scope':{'ClientSession_source':'5350B36DD93F2EA1FC9F00639073A1C990C759B26F424BCE704481B646BC959E','ClientSession_classes84_unchanged':True,'recompiled_classes':183,'byte_equal_classes':180,'different_BattleRules_classes':3,'normalized_start_method_pairs_only':4,'whole_class_equivalence':False,'runtime_BattleRules_from_existing_baseline_JAR':True},
 'evidence_levels':read(inputs/'contract.json')['evidence_levels'],
 'adopted_components':['Three own-key incremental receive lanes and three encrypted source READY D2/01 inputs with independently advancing Sender counters.','Existing exact D1/E1 observation parsers and FD1/FD2/FD0 common Session decoder in ROOM, without prediction/terrain or WORLD grant.','Qualified direct Protocol battleActorWithHeading output parsed as metadata; prepared identity/coordinates/gfx plus explicitly synthetic ancillary fields.','Fixed replay source countdown/MatchState RUNNING/local-generation/roster/source clock observations, no generic Read import.','Default local clock Unknown; opt-in restricted zero-origin elapsed clock bounded0..300s reuses existing PetRace Reducer for one source synthetic gate callback; no position/C_MOVE admission.','Separate source endBattle12->13 teardown; no second start. EOF/partial/foreign key retires receiver, Sender tickets/views, component clock and source callbacks.'],
 'milestone_windows':'Chosen per-lane offline observation windows pinned to source component stages; not a cross-socket total order, original scheduler cadence, or Modern receive timing observation.',
 'clock_domain':'Source absolute System.nanoTime stored separately from restricted offline elapsed nanos; no UTC/local synchronization or implicit FD0 clock epoch.',
 'full_WORLD_blocker':blocker,
 'regular_compile_failure_retained':read(l/'r540-initial-compile-failure.json'),
 'repair':'Only new StartComponent source changed from initial freeze: serializable PanelObservation DTO wrapping unchanged reducer PanelTransition. Initial tests executed0; final3 checks first run only.',
 'new_server_SQL_provider_GUI_or_Modern_socket':0,'old_PASS_replays':0,'full_mode_grants':0}
save(l/'r540-adopted-scope-and-version-audit.json',scope)
backlog=read(l/'r539-backlog-scope-update.json')
for item in backlog['items']:
 if item['item']=='Native_WORLD_and_RESULT_return':
  item['current_classification']='ROOM_ready_countdown_start_components_and_RESULT_return_offline_CPU_verified_full_WORLD_blocked'
  item['remaining']='R540 adopts3 own-key ROOM component streams36 start+6 teardown bodies and3 READY commands; MatchState RUNNING/localgen12 are source components, not client BATTLE/WORLD. Full WORLD is blocked by real store constructor stopped auction initializer plus selected identity/bookmarks/profile migration and conditional castle dependencies. Actual C_MOVE/terrain-position/native clock synchronization/render acceptance remain unqualified.'
backlog.update({'schema':'MODERN_R540_BACKLOG_SCOPE_UPDATE_V1','previous_scope_update':pin(l/'r539-backlog-scope-update.json'),'evidence_supported_product_work_done':'Existing incremental Receiver/Sender and ROOM common FD decoder integrated with isolated READY/countdown/roster/source clock/direct HUD actor observations and optional restricted elapsed reducer; no WORLD/scene/movement permissions.','new_full_bindings':0,'GUI':0,'LOADED':False,'overall_completed':False})
save(l/'r540-backlog-scope-update.json',backlog)
tools=['prepare-r540-start-component.py','adopt-r540-full-start-dependency.py','freeze-r540-start-component.py','build-r540-start-component.ps1','freeze-r540-start-component-repair.py','build-r540-start-component-repair.ps1','seal-r540-start-component.py'];td=t/'tools';td.mkdir()
for n in tools:shutil.copyfile(p/'tools'/n,td/n)
names=['r540-start-server-root.json','r540-start-source-adoption.json','r540-full-start-blocker-adoption.json','r540-production-source-snapshot.json','r540-final-production-source-snapshot.json','r540-library-build.jsonl','r540-library-build.stderr.log','r540-library-build-accounting.json','r540-initial-compile-failure.json','r540-final-library-build.jsonl','r540-final-library-build.stderr.log','r540-final-library-build-accounting.json','r540-final-targeted-tests.stdout.log','r540-final-targeted-tests.stderr.log','r540-final-targeted-tests-accounting.json','r540-start-keyed-component-check.json','r540-clock-domain-and-untagged-check.json','r540-start-boundary-retirement-check.json','r540-adopted-scope-and-version-audit.json','r540-backlog-scope-update.json']
refs=[l/n for n in names]+[td/n for n in tools]+sorted(inputs.iterdir())+[l/n for n in previous]+sorted((initial/'tools-initial').iterdir())
refs += [t/'source/src/modern_lito'/n for n in ['mod.rs','native_pet_race_session.rs','native_pet_race_quit_session.rs','native_pet_race_wire_boundary.rs','native_pet_race_start_component.rs','native_pet_race_start_component_tests.rs','native_source_stream.rs','native_source_send.rs','native_pet_race_component.rs','native_java_room_participants.rs','native_java_room_ready.rs']]
refs += [initial/'source/src/modern_lito/native_pet_race_start_component.rs',p/'target/r534-native-pet-race-route-inputs/adopted-r533-component-contract.json']
now=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat()
report={'schema':'MODERN_R540_PINNED_ROOM_START_CLOCK_ROSTER_COMPONENT_CONSUMER_V1','state':'ROOM_START_COMPONENTS_KEYED_WIRE_CPU_VERIFIED_FULL_WORLD_BLOCKED',
 'updated_at':now,'namespace':'NATIVE_LITO_LTMAP_V1','source_profile':scope['version_scope']['ClientSession_source'],
 'production_library':pin(t/'library/liblito_modern.rlib'),'focused_test_executable':pin(exe),'source_count':660,'prior_sources_unchanged':654,'existing_sources_changed':source['existing_sources_changed'],'new_sources':source['new_sources'],
 'existing_Receiver_Sender_reducer_READY_countdown_parsers_changed':False,'source_adoption':adoption,'adopted_scope':scope,'full_WORLD_blocker':blocker,
 'checks':{'keyed_components':keyed,'clock_and_untagged':clock,'retirement_and_atomic':retirement},
 'tests':{'passed':3,'failed':0,'old_tests_filtered':int(match[3]),'seconds':float(match[4]),'old_PASS_replays':0,'initial_failed_library_attempt_tests_executed':0},
 'builds':{'regular_attempts':2,'regular_failure':1,'regular_success':1,'test_compiles':1,'regular_final_warnings':len(warnings),'test_warnings':int(re.search(r'generated (\d+) warnings',err)[1]),'new_component_warnings':0},
 'preservation':{'prior_roots_checked':previous,'historical_snapshots_changed':False,'server_or_BIN_writes':0},
 'execution':{'new_Java_SQL_TLS_provider_GUI':0,'new_Modern_socket_or_TCP_listener':0,'new_player_EXE':False,'old_PASS_replays':0,'own_ASCII_scratch_only':True},
 'remaining':['Full WORLD blocked by real store auction initializer, bookmarks/profile migration, selected identity and conditional castle dependencies; no constructor bypass allowed.','Same-session actual WORLD/START/full serverStartBattle with client BATTLE, authenticated identity/store/scene prerequisites.','Actual C_MOVE input, terrain acceptance and committed actor position, laps/finish/results/clock synchronization and Native renderer/LOADED.','Production scheduler cadence/other BattleRules paths/whole-class equivalence unqualified; background named scene source and24 remaining complete mode bindings.'],
 'new_full_bindings':0,'complete_modes':16,'total_modes':40,'GUI':0,'LOADED':False,'overall_completed':False,'references':[pin(f) for f in refs]}
save(root,report)
human=p/'Native-R540_ROOM開始clock與roster元件交接.txt'
with human.open('x',encoding='utf-8') as f:
 f.write('R540：ROOM開始component接既有Rust模型／增量Receiver\n\n'
 '3AD211核15引用／1027manifest；BattleRules-r2核18引用／4正規化method pairs。183classes僅180 byte相同，三BattleRules class不同；採用限countdown/phase/begin四個指令區塊，不宣稱整class等價。\n'
 '三條新的自己S_KEY串流、36開始body＋6teardown、三個D2/01 READY frame皆逐offset/counter/bytes核對。原Receiver、Sender、reducer及D1/E1純decoder不改；FD1/2/0走既有Session，ROOM保留、沒有prediction/terrain或移動權限。\n'
 '固定source milestone與wire觀察分開；localgen11→12是room開始、12→13是endBattle teardown，非wiregen／下一局。MatchState RUNNING≠ClientSession BATTLE/WORLD。一般Read拒絕source milestone/離線clock，不因HUD/actor擁有scene權限。\n'
 'source absolute nanoTime只列metadata、不是本機clock或UTC；FD0之後預設clock Unknown。明確啟用的離線elapsed0..300s才接既有Reducer、重核來源直接petRaceMoved首gate；合成edge不提交actor位置、非C_MOVE/terrain證據。actor prepared id/位置/gfx來源分列，ancillary合成；未安裝drawable actor。\n'
 f'12分段/bytewise/merged安排、clock界線/untagged batch tail與EOF、atomic prefix/foreign key/READY ticket退役，3新CPU checks首次PASS／{match[3]}舊filtered／{match[4]}秒。初次library E0277序列化失敗／0tests保留，僅新module補Panel DTO後另封660來源，654舊不變，regular兩次／成功一次、test一次、新warning0；舊PASS／server沒有重跑。\n'
 'CB7F full-start依賴核12引用／18manifest：real store constructor會觸及停止的auction initializer，SQL不會消除此條件，另有bookmarks/profile migration與bootstrap條件castle；沒有store/SQL/fullstart執行，也沒有constructor/BATTLE/選角繞過。full WORLD不能預設下一輪就有。\n'
 '完整模式仍16/40、GUI0、LOADEDfalse；full WORLD/C_MOVE/clock同步/native scene與24模式仍未验收。\n\n主交接：'+str(root)+'\n')
progress_path=p.parent/'監製進度.json';progress=read(progress_path)
progress.update({'updated_at':now,'phase':'R540_ROOM_start_components_keyed_receive_CPU_verified_full_WORLD_constructor_blocked','overall_status':'in_progress','overall_completed':False,
 'current_task':'R540三自己S_KEY新ROOM stream／36開始+6teardown／3READY，既有Receiver/Sender/FD decoder接READY倒數roster/sourceclock/directHUDactor；12分段合併與3新PASS，660來源／654舊不變，GUI0。',
 'next_step':'fullWORLD目前被realStore停止auction initializer及bookmarks/profile migration/條件castle與selected identity依賴阻擋；不繞constructor或注入BATTLE。現有有限component完成，actualC_MOVE／terrain-position／clock同步／Native scene、背景及24完整模式仍缺合格證據。'})
progress['r540']={'status':report['state'],'handoff':pin(root),'library':report['production_library'],'source_count':660,'new_checks_passed':3,'old_tests_filtered':int(match[3]),'own_key_streams':3,'start_bodies':36,'teardown_bodies':6,'READY_commands':3,'patterns':12,'client_phase':'ROOM','source_generation':[11,12,13],'wire_generation_known':False,'default_local_clock':'Unknown','restricted_clock':'explicit offline elapsed0..300s only','BattleRules_qualified_methods':4,'BattleRules_whole_class_equivalence':False,'full_WORLD_blocker':blocker['root'],'regular_compile_failure_retained':True,'new_component_warnings':0,'new_Modern_socket':0,'new_full_bindings':0,'complete_modes':16,'total_modes':40,'GUI':0,'LOADED':False}
progress['completed'].append('R540：ROOM開始component既有Receiver/Sender/FD接線，三own-key streams／36+6body／3READY；source RUNNING與client ROOM分離，3新CPU PASS，full WORLD依賴阻礙明列。')
progress['verified'].append(f'R540 3PASS0FAIL／{match[3]}filtered／{match[4]}s；15refs1027manifest＋18refs4methods＋dependency12refs18manifest，12分段合併，clock Unknown／untagged／EOF拒絕越權；初次E0277保留，新warning0。')
with progress_path.open('w',encoding='utf-8') as f:json.dump(progress,f,ensure_ascii=False,indent=2);f.write('\n')
print(json.dumps({'handoff':pin(root),'library':report['production_library'],'references':len(refs),'tests':report['tests'],'builds':report['builds']},ensure_ascii=False))
