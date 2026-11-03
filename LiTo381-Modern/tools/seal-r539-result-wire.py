import datetime,hashlib,json,pathlib,re,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';initial=p/'target/r539-native-pet-race-result-wire';t=p/'target/r539-native-pet-race-result-wire-final';inputs=p/'target/r539-native-pet-race-result-wire-inputs'
def read(f):return json.loads(f.read_text(encoding='utf-8-sig'))
def pin(f):
    h=hashlib.sha256()
    with f.open('rb') as s:
        for b in iter(lambda:s.read(1024*1024),b''):h.update(b)
    return {'path':str(f),'bytes':f.stat().st_size,'sha256':h.hexdigest().upper()}
def save(f,v):
    with f.open('x',encoding='utf-8') as s:json.dump(v,s,ensure_ascii=False,indent=2);s.write('\n')
root=l/'r539-result-own-key-wire-consumer-handoff.json';assert not root.exists()
prior=read(l/'r538-keyed-incremental-wire-consumer-handoff.json');previous=dict(prior['preservation']['prior_roots_checked']);previous['r538-keyed-incremental-wire-consumer-handoff.json']='4AC931DE3FEAF8306FDE2B1752DFF1B8ABA2419F2630045C44C6B73EBF107F7E'
for n,h in previous.items():assert pin(l/n)['sha256']==h
adoption=read(l/'r539-result-wire-adoption.json');assert (adoption['references_checked'],adoption['manifest_files_checked'],adoption['endpoint_refs_checked'],adoption['exact_source_excerpts_checked'])==(13,617,6,9)
assert pin(pathlib.Path(adoption['root']['path']))==adoption['root'];assert pin(pathlib.Path(adoption['endpoint_supplement']['path']))==adoption['endpoint_supplement']
out=(l/'r539-targeted-tests.stdout.log').read_text(encoding='utf-8-sig');match=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; .*?(\d+) filtered out; finished in ([0-9.]+)s',out)
assert match and match.group(1,2)==('3','0'),out
err=(l/'r539-targeted-tests.stderr.log').read_text(encoding='utf-8-sig')
exe=initial/'r539-focused-tests.exe';assert exe.exists()
final_rows=[json.loads(s) for s in (l/'r539-final-test-compile.jsonl').read_text(encoding='utf-8-sig').splitlines()]
final_artifacts=[r for r in final_rows if r.get('reason')=='compiler-artifact' and r['target']['name']=='lito_modern' and r['profile']['test'] and r.get('executable')];assert len(final_artifacts)==1
final_exe=t/'r539-compiled-tests-no-run.exe';shutil.copyfile(final_artifacts[0]['executable'],final_exe)
source=read(l/'r539-final-production-source-snapshot.json')
assert source['production_runtime_changes_since_initial_PASS']==0
assert pin(t/'library/liblito_modern.rlib')['sha256']==pin(initial/'library/liblito_modern.rlib')['sha256']
for d in source['files']:
    assert pin(p/d['path'])['sha256']==d['sha256'] and pin(t/'source'/d['path'])['sha256']==d['sha256']
rows=[json.loads(s) for s in (l/'r539-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()]
warnings=[r['message'] for r in rows if r.get('reason')=='compiler-message' and r['message']['level']=='warning']
changed=[n.replace('modern_lito/','') for n in source['existing_sources_changed']+source['new_sources']]
assert not [w for w in warnings if any(any(n in s['file_name'] for n in changed) for s in w['spans'])]
initial_testwarnings=int(re.search(r'generated (\d+) warnings',err)[1])
final_warnings=[r['message'] for r in final_rows if r.get('reason')=='compiler-message' and r['message']['level']=='warning']
assert not [w for w in final_warnings if any(any(n in s['file_name'] for n in changed) for s in w['spans'])]
testwarnings=len(final_warnings)
assert read(l/'r539-library-build-accounting.json')['exit_code']==0 and read(l/'r539-targeted-tests-accounting.json')['exit_code']==0 and read(l/'r539-final-test-compile-accounting.json')['exit_code']==0
bidirectional=read(l/'r539-result-wire-bidirectional-check.json');untagged=read(l/'r539-untagged-result-boundary-check.json');retirement=read(l/'r539-result-wire-retirement-check.json')
assert (bidirectional['streams'],bidirectional['actual_unique_source_bodies'],bidirectional['actual_unique_source_commands'],bidirectional['segmented_and_merged_replay_patterns'])==(6,22,8,24)
assert untagged['RESULT_peers_return_unknown']==4 and retirement['missing_confirm_cannot_return']
now=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat()
audit={'state':'SOURCE_QUALIFIED_RESULT_INITIAL_FIXTURE_AND_EXISTING_RECEIVER_SENDER_COMPOSITION',
    'source_profile':'5350B36DD93F2EA1FC9F00639073A1C990C759B26F424BCE704481B646BC959E',
    'initial_state':['CHANNEL watcher outside room with no local race','Two RESULT peers with own room and raw own-D0 display alias'],
    'receiver_source_changed':False,'sender_source_changed':False,'terrain_or_previous_combat_created':False,
    'source_current_and_frozen_hashes_checked':658,'old_sources_unchanged':654,
    'new_fixture_binding_rejects_foreign_phase_alias_lane':True,'unseeded_or_partial_S_KEY_cannot_prepare':True,
    'counter_rules':'Both directions independently seed from their own complete emitted S_KEY. Receiver advances only complete body size; Sender advances at reservation commit before assumed writes. Headers excluded.',
    'request_windows':'Pinned source stages choose offline command positions before corresponding per-lane replies. These are not observations of Modern socket timing or cross-socket total ordering.',
    'D0_completion_rule':'Only after all3 encrypted frame bytes are reported written does the existing CandidateSession commit its ResultConfirm; partial/EOF/write failure retire its scope and opaque tickets/views.',
    'endpoint_provenance':adoption['endpoint_provenance'],
    'prepayload_read_only_lookup_failures':['src/native_pet_race_wire_boundary.rs missing; corrected to src/modern_lito/native_pet_race_wire_boundary.rs','src/modern_lito/native_city_runtime/native_source_send.rs missing; corrected to src/modern_lito/native_source_send.rs'],
    'overbroad_read_only_rg':'Initial root rg also traversed target snapshots; completed normally, then subsequent searches were narrowed to source/tools paths.',
    'initial_test_warning_and_repair':read(l/'r539-initial-seal-rejection.json'),
    'final_test_name_only_repair':source['test_name_only_repair'],'final_test_executable_compiled_no_run':True,
    'production_library_unchanged_after_initial_PASS':True,'new_Java_SQL_TLS_provider_GUI_or_network_runs':0,'old_PASS_replays':0,'old_historical_roots_unchanged':previous}
save(l/'r539-source-and-counter-audit.json',audit)
closure={'state':'NEW_CA3C_CONNECTIONS_SUPPLY_EXACT_RESULT_START_OWN_KEYS_NOT_HISTORICAL_66ED_REPAIR',
    'old_request_unchanged':pin(l/'r538-exact-result-wire-source-request.json'),'new_source_root':adoption['root'],
    'six_new_own_keys_and_offsets_consumed':True,'all22_application_bodies_byte_equal_66ED':True,
    'historical_66ED_keys_still_absent':True,'historical_66ED_wire_repaired':False,
    'previous_combat_or_D3_qualified':False,'Modern_socket_or_player_GUI_qualified':False}
save(l/'r539-result-wire-source-closure.json',closure)
backlog=read(l/'r538-backlog-scope-update.json')
for item in backlog['items']:
    if item['item']=='Native_WORLD_and_RESULT_return':
        item['current_classification']='new_own_key_RESULT_start_wire_and_bidirectional_counter_consumer_CPU_verified_live_WORLD_and_GUI_pending'
        item['remaining']='R539 consumes six new actual RESULT-start connections with own keys and offsets,22 return bodies and8 D9/D0 commands. Historical66ED wire keys remain absent; no prior combat, authenticated WORLD, Modern actual socket or GUI receipt qualification. Native roster/RUNNING/clock/occupancy/actor effects remain open.'
backlog.update({'schema':'MODERN_R539_BACKLOG_SCOPE_UPDATE_V1','previous_scope_update':pin(l/'r538-backlog-scope-update.json'),'evidence_supported_product_work_done':'Pinned lifecycle-only CHANNEL/RESULT fixture initialization; existing incremental Receiver and Sender with per-lane source wire binding and offline write completion, untagged authority and EOF retirement checks.','new_full_bindings':0,'GUI':0,'LOADED':False,'overall_completed':False})
save(l/'r539-backlog-scope-update.json',backlog)
tools=['prepare-r539-result-wire.py','freeze-r539-result-wire.py','build-r539-result-wire.ps1','freeze-r539-result-wire-warning-repair.py','build-r539-result-wire-warning-repair.ps1','seal-r539-result-wire.py'];td=t/'tools';td.mkdir()
for n in tools:shutil.copyfile(p/'tools'/n,td/n)
names=['r539-result-server-root.json','r539-result-wire-adoption.json','r539-production-source-snapshot.json','r539-final-production-source-snapshot.json','r539-library-build.jsonl','r539-library-build.stderr.log','r539-library-build-accounting.json','r539-targeted-tests.stdout.log','r539-targeted-tests.stderr.log','r539-targeted-tests-accounting.json','r539-result-wire-bidirectional-check.json','r539-untagged-result-boundary-check.json','r539-result-wire-retirement-check.json','r539-initial-seal-rejection.json','r539-final-test-compile.jsonl','r539-final-test-compile.stderr.log','r539-final-test-compile-accounting.json','r539-source-and-counter-audit.json','r539-result-wire-source-closure.json','r539-backlog-scope-update.json']
refs=[l/n for n in names]+[td/n for n in tools]+sorted(inputs.iterdir())+[l/n for n in previous]
refs += [t/'source/src/modern_lito'/n for n in ['native_pet_race_session.rs','native_pet_race_quit_session.rs','native_pet_race_wire_boundary.rs','native_pet_race_result_wire_tests.rs','native_source_stream.rs','native_source_send.rs']]
refs += sorted((initial/'tools-initial').iterdir())+[initial/'source/src/modern_lito/native_pet_race_result_wire_tests.rs',initial/'library/liblito_modern.rlib']
report={'schema':'MODERN_R539_NEW_RESULT_START_OWN_KEY_BIDIRECTIONAL_WIRE_CONSUMER_V1',
    'state':'SIX_NEW_SOURCE_CONNECTIONS_EXISTING_RECEIVER_SENDER_RESULT_RETURN_CPU_VERIFIED_LIVE_AUTHORITY_CLOSED',
    'updated_at':now,'namespace':'NATIVE_LITO_LTMAP_V1','source_profile':audit['source_profile'],
    'production_library':pin(t/'library/liblito_modern.rlib'),'focused_test_executable':pin(exe),'final_compiled_test_executable_not_run':pin(final_exe),
    'source_count':658,'prior_sources_unchanged':654,'existing_sources_changed':source['existing_sources_changed'],'new_sources':source['new_sources'],
    'existing_Receiver_and_Sender_changed':False,'synthetic_previous_combat_or_D3':False,
    'source_adoption':adoption,'endpoint_provenance':adoption['endpoint_provenance'],
    'source_scope':{'new_connections':6,'own_S_KEY_offset':0,'first_encrypted_server_frame_offset':7,'unique_server_return_bodies':22,'unique_client_D9_D0_commands':8,'actual_source_EOF':6,'source_CLASS_changes':0,'not_historical_66ED_key_repair':True},
    'checks':{'bidirectional':bidirectional,'untagged':untagged,'retirement':retirement},
    'tests':{'passed':3,'failed':0,'old_tests_filtered':int(match[3]),'seconds':float(match[4]),'old_PASS_replays':0,'qualified_PASS_executable':'retained initial executable','final_no_run_compile_tests_executed':0,'only_difference_is_lowercase_test_name':True},
    'builds':{'regular_library':1,'test_compiles':2,'regular_warnings':len(warnings),'initial_test_warnings':initial_testwarnings,'initial_new_test_naming_warning':1,'final_test_warnings':testwarnings,'new_component_warnings':0,'runtime_library_rebuild_for_test_name_only_repair':False},
    'preservation':{'prior_roots_checked':previous,'server_or_BIN_writes':0,'historical_snapshots_changed':False},
    'execution':{'new_Java_SQL_TLS_provider_GUI':0,'new_Modern_socket_or_TCP_listener':0,'new_player_EXE':False,'own_ASCII_scratch_only':True,'old_PASS_replays':0},
    'remaining':['Historical66ED own wire keys remain absent; new CA3C sessions are not repaired historical captures.','Previous combat/authentication/player WORLD, same-epoch live roster/RUNNING, native clocks/occupancy/actor/form/Brave/effect producers and GUI unqualified.','No complete-catalog end marker, full player receipt ACK or cross-socket total order; new fault/stress domains remain unqualified.','Background named scene/draw source and24 remaining complete mode qualifications.'],
    'new_full_bindings':0,'complete_modes':16,'total_modes':40,'GUI':0,'LOADED':False,'overall_completed':False,'references':[pin(f) for f in refs]}
save(root,report)
human=p/'Native-R539_RESULT同連線key與雙向counter交接.txt'
with human.open('x',encoding='utf-8') as s:
    s.write('R539：新RESULT起點連線的自己的S_KEY／原始offset接既有Receiver與Sender\n\n'
        'CA3C核13引用／617manifest，endpoint補註225994核6引用／9來源原行；六條新連線、22回包、8個D9／D0全以自己的key與offset核對。22body等於66ED返回子集，沒有替歷史66ED補借key。\n'
        'watcher直接以CHANNEL／room0、兩peer直接以RESULT與來源room/alias建立離線fixture；沒有terrain、先前combat或合成D3、勝方、roster。固定lane的私有binding阻擋跨peer起始模型。\n'
        '原Receiver／Sender不改：完整7byte S_KEY才seed，收與送各自counter；接收完整body才advance、header不算；送方commit先advance，完整3byte寫入假設才提交RESULT D0。半包／EOF／zero-write使scope/ticket/view退役。\n'
        '24分段/bytewise/合併CPU安排涵蓋六lane；RESULT D9/F0保留RESULT、peerD0和owner無自身確認、ownD0→CC本地返回、default reset及nondefault CF刪3原序全部核。來源六EOF另列。一般Read保持untagged，四RESULT peer返回Unknown，不能取得live authority。\n'
        f'3新checks首次PASS、{match[3]}舊測試filtered、{match[4]}秒；regular library編譯1次。初次test compile命名EOF帶1新warning，PASS/exe/source保留；僅改test名稱小寫並另封最後658來源，第二次test compile --no-run、新warning0，未重跑PASS或重建library。654舊來源不變；Java／server/provider沒有重跑。\n'
        'endpoint的0.0.0.0是accepted actual socket在FIN／join／EOF之後保存的accessor輸出，不是live bind觀察；127.0.0.1:port是source172/177加既有runtime port推導。無新endpoint觀察。\n'
        '此為Modern離線CPU回放，非Modern實際socket／GUI／full-player ACK。完整模式仍16/40、GUI0、LOADEDfalse；live WORLD、效果producer、24模式及背景scene仍未完成。\n\n主交接：'+str(root)+'\n')
progress_path=p.parent/'監製進度.json';progress=read(progress_path)
progress.update({'updated_at':now,'phase':'R539_new_RESULT_start_own_key_bidirectional_wire_CPU_verified_live_WORLD_GUI_open','overall_status':'in_progress','overall_completed':False,
    'current_task':'R539六條新RESULT起點連線以自己S_KEY／offset接既有Receiver和Sender，直接CHANNEL watcher／RESULT peers、22body／8commands、24分段合併安排；3新CPU PASS，658來源／654舊不變，GUI0。',
    'next_step':'新CA3C來源已消費；歷史66ED自身key仍未供，不混作修復。live WORLD同epoch roster/RUNNING、native clock/occupancy/actor/effect、Modern實際socket與GUI、背景scene及24完整模式仍待合格来源／驗證。'})
progress['r539']={'status':report['state'],'handoff':pin(root),'library':report['production_library'],'source_count':658,'new_checks_passed':3,'old_tests_filtered':int(match[3]),'new_connections':6,'own_keys':6,'return_bodies':22,'commands':8,'patterns':24,'source_EOF_scopes':6,'synthetic_D3_or_combat':False,'existing_Receiver_and_Sender_reused':True,'new_component_warnings':0,'new_Modern_socket':0,'new_full_bindings':0,'complete_modes':16,'total_modes':40,'GUI':0,'LOADED':False}
progress['completed'].append('R539：六新連線／自己S_KEY與offset／22body／8commands接既有Receiver＋Sender；直接RESULT／CHANNEL fixture，不合成D3，3新CPU PASS，GUI0。')
progress['verified'].append(f'R539 3PASS0FAIL／{match[3]}filtered／{match[4]}s；13refs617manifest＋endpoint6refs9原行、24分段合併、untagged Unknown／EOF ticket退役，新warning0。')
with progress_path.open('w',encoding='utf-8') as s:json.dump(progress,s,ensure_ascii=False,indent=2);s.write('\n')
print(json.dumps({'handoff':pin(root),'library':report['production_library'],'references':len(refs),'tests':report['tests'],'builds':report['builds']},ensure_ascii=False))
