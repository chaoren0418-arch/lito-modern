import datetime,hashlib,json,pathlib,re,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';t=p/'target/r537-native-pet-race-return-session-final'
root=l/'r537-native-result-return-partial-consumer-handoff.json';assert not root.exists()
def read(f):return json.loads(f.read_text(encoding='utf-8-sig'))
def pin(f):
    h=hashlib.sha256()
    with f.open('rb') as s:
        for b in iter(lambda:s.read(1024*1024),b''):h.update(b)
    return {'path':str(f),'bytes':f.stat().st_size,'sha256':h.hexdigest().upper()}
def save(f,v):
    with f.open('x',encoding='utf-8') as s:json.dump(v,s,ensure_ascii=False,indent=2);s.write('\n')
previous={
 'r532-coverage-witness-correction-handoff.json':'7BB72B2DB3B800F14C286E88B45A20EB5D8D32C217E5ADDB115DA7DE8F315119',
 'r533-full-backlog-and-pet-race-handoff.json':'F0098FFE958638E2C52678E28970456709253861CB7843D5F6EABC8172E54AFA',
 'r534-native-terrain-offline-consumer-handoff.json':'A14ED497A194DFEAFB535774170D1629139A6B9B4D84CF8165E265C03977FAFF',
 'r535-native-session-lifecycle-consumer-handoff.json':'40920AFB5207A63373FF153A28954CA6FE4B070E79D758AFB007888E4F0F395B',
 'r536-native-quit-intent-lifecycle-consumer-handoff.json':'110A6B38FBF5424D79A243003DDBE1447ABAFEC07DAB2B90893B037CDB8AE284'}
for n,h in previous.items():assert pin(l/n)['sha256']==h
out=(l/'r537-targeted-tests.stdout.log').read_text(encoding='utf-8-sig')
match=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; .*?(\d+) filtered out; finished in ([0-9.]+)s',out)
assert match and match.group(1,2)==('2','0'),out
err=(l/'r537-targeted-tests.stderr.log').read_text(encoding='utf-8-sig')
exepath=re.search(r'Running unittests .*?\(([^\r\n]+\.exe)\)',err).group(1)
exe=t/'r537-focused-tests.exe';assert not exe.exists();shutil.copyfile(p/exepath,exe)
source=read(l/'r537-final-production-source-snapshot.json')
for d in source['files']:
    assert pin(p/d['path'])['sha256']==d['sha256']
    assert pin(t/'source'/d['path'])['sha256']==d['sha256']
rows=[json.loads(s) for s in (l/'r537-library-build-repair1.jsonl').read_text(encoding='utf-8-sig').splitlines()]
warnings=[r['message'] for r in rows if r.get('reason')=='compiler-message' and r['message']['level']=='warning']
newwarnings=[w for w in warnings if any('native_pet_race_return_profile' in s['file_name'] or 'native_pet_race_quit_session' in s['file_name'] for s in w['spans'])]
assert not newwarnings
assert 'native_pet_race_return_profile.rs:' not in err and 'native_pet_race_return_profile_tests.rs:' not in err and 'native_pet_race_quit_session.rs:' not in err
testwarnings=int(re.search(r'generated (\d+) warnings',err)[1])
assert read(l/'r537-library-build-accounting.json')['exit_code']!=0
assert read(l/'r537-library-build-repair1-accounting.json')['exit_code']==0 and read(l/'r537-targeted-tests-accounting.json')['exit_code']==0
initial_rows=[json.loads(s) for s in (l/'r537-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()]
initial_errors=[r['message'] for r in initial_rows if r.get('reason')=='compiler-message' and r['message']['level']=='error']
assert len(initial_errors)==1 and initial_errors[0]['code']['code']=='E0433'
actual=read(l/'r537-result-return-replay-check.json');counter=read(l/'r537-return-partial-counterexamples-check.json')
assert (actual['sequences'],actual['actual_setup_bodies'],actual['actual_return_bodies'])==(6,17,22)
assert len(counter['partial_failures'])==2
inputs=p/'target/r537-native-pet-race-return-inputs';adoption=read(l/'r537-caller-return-adoption.json')
assert adoption['manifest_files_checked']==1036 and adoption['references_checked']==13
now=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat()
audit=read(l/'r536-backlog-scope-update.json')
for item in audit['items']:
    if item['item']=='Native_WORLD_and_RESULT_return':
        item['current_classification']='exact_5350_offline_RESULT_return_consumed_live_WORLD_authority_still_pending'
        item['remaining']='R537 now consumes actual successful candidate RESULT C_D0 own D0/CC, peer D0/D505 and CF bodies with independent5350 profile. No actual client receipt, complete-catalog marker, live WORLD admission, renderer or deployed485F upgrade.'
    if item['item']=='Native_PetRace_566_569_570':
        item['current_classification']='finite_progression_terrain_session_quit_and_return_consumers_implemented_live_authority_pending'
        item['remaining']='R533–537 finite components composed. Runtime source/caller fixture map566 only; actual actor/form/Brave/effect, server clock/dynamic occupancy and live mode admission remain missing.'
audit.update({'schema':'MODERN_R537_BACKLOG_SCOPE_UPDATE_V1','previous_scope_update':pin(l/'r536-backlog-scope-update.json'),
    'evidence_supported_product_work_done':'New sealed5350 ReturnContract composes existing terrain/session/quit consumer; typed CF/D505/catalog observations, exact current-body own-exit matching and retained partial fail-close state integrated.',
    'new_full_bindings':0,'GUI':0,'LOADED':False,'overall_completed':False})
save(l/'r537-backlog-scope-update.json',audit)
td=t/'tools';td.mkdir();tools=['prepare-r537-native-return.py','freeze-r537-native-return.py','build-r537-native-return.ps1','seal-r537-native-return.py',
    'freeze-r537-native-return-repair1.py','build-r537-native-return-repair1.ps1']
for n in tools:shutil.copyfile(p/'tools'/n,td/n)
names=['r537-caller-return-server-root.json','r537-caller-return-adoption.json','r537-production-source-snapshot.json','r537-final-production-source-snapshot.json','r537-backlog-scope-update.json',
       'r537-library-build.jsonl','r537-library-build.stderr.log','r537-library-build-accounting.json','r537-targeted-tests.stdout.log',
       'r537-targeted-tests.stderr.log','r537-targeted-tests-accounting.json','r537-result-return-replay-check.json','r537-return-partial-counterexamples-check.json',
       'r537-library-build-repair1.jsonl','r537-library-build-repair1.stderr.log','r537-library-build-repair1-accounting.json']
refs=[l/n for n in names]+[td/n for n in tools]
refs += [t/'source/src/modern_lito'/n for n in ['mod.rs','native_pet_race_session.rs','native_pet_race_quit_session.rs','native_pet_race_return_profile.rs','native_pet_race_return_profile_tests.rs']]
refs += sorted(inputs.iterdir())
refs += [l/'r536-native-quit-intent-lifecycle-consumer-handoff.json',p/'target/r535-native-pet-race-outer-inputs/outer-lifecycle-contract-v1.json']
terrain=p/'target/r534-native-pet-race-route-inputs';refs += [terrain/n for n in ['adopted-r533-component-contract.json','s32-layer3-to-actual-reader-byte-comparison.json']]+sorted((terrain/'native/566').glob('*.s32'))
report={'schema':'MODERN_R537_EXACT_RESULT_RETURN_AND_PARTIAL_FAILURE_CONSUMER_V1',
 'state':'NEW_5350_PROFILE_RESULT_SELF_PEER_OWNER_DELETE_CATALOG_AND_PARTIAL_FAIL_CLOSE_CPU_VERIFIED',
 'updated_at':now,'namespace':'NATIVE_LITO_LTMAP_V1',
 'profile':{'candidate_source':'5350B36DD93F2EA1FC9F00639073A1C990C759B26F424BCE704481B646BC959E',
    'candidate_root':'66EDC23BAA24A9C98D9837E7FC73A02CB64AB37BEEEE4E62AFEF28113C4502BB',
    'version_lifecycle_contract':'6A69E16EEEF5A1177BB6826CA18D1AB62AADCFE0227E756C1D1036EF46DEC3A6',
    'historic_00FC_013A_profiles_retained':True,'deployed_485F_adoption_or_equivalence':False},
 'production_library':pin(t/'library/liblito_modern.rlib'),'focused_test_executable':pin(exe),
 'source_count':655,'prior_sources_unchanged':650,'existing_sources_changed':source['existing_sources_changed'],'new_sources':source['new_sources'],
 'source_policies':['Sealed new version contract constructs5350 successor; historical00FC factory/source remains available and extra incoming formats gated to successor.',
    'RESULT D9/F0 retains RESULT; own confirm intent and current matching D0 then CC0 yields only a hypothetical fixed-delivery local Channel view.',
    'Current D0 must itself match room/display alias; a latched prior self-exit flag cannot qualify a foreign peer D0.',
    'Peer D0 and D505 owner display updates have no own exit authority; CF changes observed room rows but is not confirm/catalog-completion ACK.',
    'CC0 rows preserve raw displayedUsers/mode/mapword and remain incomplete catalog observations; displayedUsers1 never infers actual occupancy.',
    'Partial encrypted prefixes are preserved separately, not decrypted/padded/dispatched as application bodies; offline failure closes scope and commands.',
    'Missing own D0/list and untagged late replies remain Unknown; only receiver ordinals order inputs, with no cross-socket total order or live receipt claim.'],
 'adoption':adoption,'checks':{'actual_application_return':actual,'counterexamples_and_partial':counter},
 'tests':{'passed':2,'failed':0,'old_tests_filtered':int(match[3]),'seconds':float(match[4]),'old_PASS_or_actual_Java_methods_replayed':False},
 'builds':{'regular_library_attempts':2,'regular_library_successful':1,'test_compiles':1,'regular_warnings':len(warnings),'new_component_warnings':0,'test_warnings':testwarnings,
    'initial_error_retained':'E0433 room catalog module path; initial regular build failed before any tests; one path correction then regular build and first focused test execution.',
    'initial_tests_executed':0,'initial_source_and_tools_retained':True},
 'remaining':['Live WORLD authority, same-epoch roster/RUNNING/clock/movement/dynamic occupancy and actor/form/Brave/effect producer integration.',
    'Actual client receipt/real socket concurrency and partial delivery; production intercepted shop/trade/prize/persistence domains.',
    'No zero-total-catalog completion marker/deadline or wire intent/generation field; default row floor1 and deleted target do not supply these.',
    'Player GUI/LOADED, named background scene/draw source and remaining24 full mode qualifications.'],
 'preservation':{'prior_root_pins_checked':previous,'server_or_BIN_writes':0,'historical_root_and_source_snapshots_changed':False},
 'execution':{'Java_SQL_TLS_provider_GUI':0,'new_player_EXE':False,'actual_socket_write':False,'own_ASCII_scratch_only':True,'old_PASS_replays':0},
 'new_full_bindings':0,'complete_modes':16,'total_modes':40,'GUI':0,'LOADED':False,'references':[pin(f) for f in refs],'overall_completed':False}
save(root,report)
human=p/'Native-R537_RESULT返回與部分寫入交接.txt'
with human.open('x',encoding='utf-8') as s:
    s.write('R537：RESULT返回與部分寫入\n\n新5350B36D／root66EDC23B版本獨立綁定；13引用／1036manifest核對，不代表485F採用。R532–536封存與013A／00FC歷史profile保留。\n'
        '6玩家序列、39成功body：17必要setup＋22返回，僅各receiver ordinal有序；4 RESULT D9 acknowledge、4 RESULT D0 confirm，以及peer D0／2 owner更新／3 CF刪房離線接線。\n'
        'RESULT D9／F0保持Result；本地confirm意圖、當個body同房同顯示名D0及CC0才有假設固定收包下的Channel view。peer D0、owner顯示名或刪房不能代替自己的D0。\n'
        'default空房actual users0／wire displayed1，僅保留原wire欄位；非default目標刪除但default row仍在，不能當整個catalog0row／completion marker。\n'
        '2個新candidate partial：保留3/4與5/23 bytes encrypted prefix，不解碼／補齊為application body。offline失敗假設關閉scope／舊命令，狀態Disconnected，server原始IO與狀態另存，非client收到證明。\n'
        f'2新CPU checks PASS、{match[3]}舊測試filtered、{match[4]}秒；{counter["malformed_atomic_rejections"]}malformed原子拒絕。655初次／最終來源另封，650舊來源不變。首次E0433引用路徑錯誤／0tests保留，修一個引用後regular重編1次、test compile1次並首次执行PASS，新warning0。\n'
        '缺包與未標記晚到仍Unknown；不假造零列表完成訊號或實際occupancy。未重跑舊PASS／Java，未啟SQL／provider／Native／GUI，仍16/40、GUI0、LOADEDfalse。\n'
        '仍欠live WORLD authority、actor/效果producer、實際client receipt／transport concurrency、GUI與其餘24模式驗收。\n\n主交接：'+str(root)+'\n')
progress_path=p.parent/'監製進度.json';progress=read(progress_path)
progress.update({'updated_at':now,'phase':'R537_exact_RESULT_return_and_partial_consumer_CPU_verified_live_WORLD_GUI_mode_authority_open',
    'overall_status':'in_progress','overall_completed':False,
    'current_task':'R537新5350 profile消費6序列39body（17setup22return），RESULT D9/F0保持Result，C_D0自己D0+CC分流；peer/owner/CF不誤判，2partial encrypted prefix保留並fail-close。13引用／1036manifest、2新PASS／655來源另封，不宣告485F／client receipt／full模式。',
    'next_step':'有限candidate RESULT incoming body缺項已在5350版本補齊；仍欠live WORLD同epoch roster/RUNNING/clock/occupancy及actor/form/Brave/effect producer、實際transport/receipt/concurrency、背景scene與GUI/LOADED／24模式。零總列完成／wire世代標記仍不存在，不能補造。'})
progress['r537']={'status':report['state'],'handoff':pin(root),'library':report['production_library'],'source_count':655,'new_checks_passed':2,
    'old_tests_filtered':int(match[3]),'sequences':6,'actual_successful_application_bodies':39,'setup_bodies':17,'return_bodies':22,
    'partial_prefixes_retained':2,'manifest_items_checked':1036,'new_component_warnings':0,
    'R536_actual_candidate_return_input_gap_closed_only_for_new5350_profile':True,
    'client_receipt_live_world_proven':False,'new_full_bindings':0,'complete_modes':16,'total_modes':40,'GUI':0,'LOADED':False}
progress['completed'].append('R537：5350独立profile RESULT自己D0／peer／owner／CF／CC与partial fail-close元件完成2新CPU checks；6序列39body及2partial，655來源另封，GUI0。')
progress['verified'].append(f'R537 2PASS0FAIL／{match[3]}filtered／{match[4]}s，新warning0，13引用／1036manifest，17setup22return，{counter["malformed_atomic_rejections"]}malformed原子拒絕；無client ACK推論。')
with progress_path.open('w',encoding='utf-8') as s:json.dump(progress,s,ensure_ascii=False,indent=2);s.write('\n')
print(json.dumps({'handoff':pin(root),'library':report['production_library'],'tests':report['tests'],'builds':report['builds']},ensure_ascii=False))
