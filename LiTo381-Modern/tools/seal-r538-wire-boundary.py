import datetime,hashlib,json,pathlib,re,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';t=p/'target/r538-native-pet-race-wire-boundary'
root=l/'r538-keyed-incremental-wire-consumer-handoff.json';assert not root.exists()
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
 'r536-native-quit-intent-lifecycle-consumer-handoff.json':'110A6B38FBF5424D79A243003DDBE1447ABAFEC07DAB2B90893B037CDB8AE284',
 'r537-native-result-return-partial-consumer-handoff.json':'7A71A5AE510FE632E8880DD571EA06938BA4665F9767A58477CC77E1469E8ADD'}
for n,h in previous.items():assert pin(l/n)['sha256']==h
out=(l/'r538-targeted-tests.stdout.log').read_text(encoding='utf-8-sig')
match=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; .*?(\d+) filtered out; finished in ([0-9.]+)s',out)
assert match and match.group(1,2)==('3','0'),out
err=(l/'r538-targeted-tests.stderr.log').read_text(encoding='utf-8-sig')
exepath=re.search(r'Running unittests .*?\(([^\r\n]+\.exe)\)',err).group(1)
exe=t/'r538-focused-tests.exe';assert not exe.exists();shutil.copyfile(p/exepath,exe)
source=read(l/'r538-production-source-snapshot.json')
for d in source['files']:
    assert pin(p/d['path'])['sha256']==d['sha256'] and pin(t/'source'/d['path'])['sha256']==d['sha256']
rows=[json.loads(s) for s in (l/'r538-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()]
warnings=[r['message'] for r in rows if r.get('reason')=='compiler-message' and r['message']['level']=='warning']
newwarnings=[w for w in warnings if any('native_pet_race_wire_boundary' in s['file_name'] for s in w['spans'])]
assert not newwarnings
assert 'native_pet_race_wire_boundary.rs:' not in err and 'native_pet_race_wire_boundary_tests.rs:' not in err
testwarnings=int(re.search(r'generated (\d+) warnings',err)[1])
assert read(l/'r538-library-build-accounting.json')['exit_code']==0 and read(l/'r538-targeted-tests-accounting.json')['exit_code']==0
complete=read(l/'r538-incremental-wire-complete-check.json');partial=read(l/'r538-partial-eof-and-binding-errors-check.json');loop=read(l/'r538-actual-loopback-receive-order-check.json')
assert complete['split_merge_patterns']==12 and complete['actual_keyed_application_bodies']==2
assert len(partial['actual_memory_partial_streams'])==2
assert (loop['streams'],loop['actual_received_complete_bodies'],loop['actual_partial_tails'],loop['source_observed_EOF_applied'])==(6,13,2,4)
adoption=read(l/'r538-loopback-wire-adoption.json');assert adoption['references_checked']==14 and adoption['manifest_files_checked']==642
memory=p/'target/r538-native-pet-race-wire-inputs';actual=p/'target/r538-native-pet-race-loopback-wire-inputs'
now=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat()
request={
 'schema':'MODERN_R538_EXACT_RESULT_WIRE_SUPPLY_REQUEST_V1','state':'SIX_ORIGINAL_66ED_RESULT_RECEIVER_KEYS_AND_CONTINUITY_STILL_MISSING',
 'source_profile':'5350B36DD93F2EA1FC9F00639073A1C990C759B26F424BCE704481B646BC959E',
 'receiver_lanes':[scenario+'/'+role for scenario in ['default_room_last_member_reset','nondefault_room_last_member_delete'] for role in ['quitter','first-peer','healthy-peer']],
 'required':['For each receiver, its actual emitted S_KEY frame and complete encrypted receive stream from that seed through 17 setup/22 RESULT return bodies; exact per-receiver offsets/ordinals and plaintext pins.',
    'If resumed midstream, an independently owner-published source-bound start counter, exact consumed prior encrypted bytes and receiver identity/offset continuity; no borrowed key or guessed preset zero counter.',
    'Actual received EOF/partial tails separately pinned and associated to the same receiver; no zero-row completion marker or client ACK invented.'],
 'EC96_loopback_streams_are_different_sessions_not_keys_for_66ED_RESULT':True,'new_runtime_execution_requested':False,
 'live_WORLD_GUI_authority_request_not_granted':True,'complete_modes':16,'total_modes':40,'GUI':0,'overall_completed':False}
save(l/'r538-exact-result-wire-source-request.json',request)
audit=read(l/'r537-backlog-scope-update.json')
for item in audit['items']:
    if item['item']=='Native_WORLD_and_RESULT_return':
        item['current_classification']='finite_RETURN_body_and_keyed_receive_boundary_implemented_original_RESULT_wire_keys_live_WORLD_pending'
        item['remaining']='R538 reuses existing Receiver for verified5350 stream/key lanes, including actual owner loopback receive. Six original66ED RESULT-return receiver key/continuity sources still absent; live WORLD/session authority, player receipt/GUI remain unqualified.'
audit.update({'schema':'MODERN_R538_BACKLOG_SCOPE_UPDATE_V1','previous_scope_update':pin(l/'r537-backlog-scope-update.json'),
    'evidence_supported_product_work_done':'Existing Native incremental frame/cipher Receiver composed to R537 consumer with pinned per-session wire data, distinct fixed-replay versus untagged Read construction, EOF/partial/retirement handling.',
    'new_full_bindings':0,'GUI':0,'LOADED':False,'overall_completed':False})
save(l/'r538-backlog-scope-update.json',audit)
td=t/'tools';td.mkdir();tools=['prepare-r538-wire-boundary.py','prepare-r538-loopback-wire.py','freeze-r538-wire-boundary.py','build-r538-wire-boundary.ps1','seal-r538-wire-boundary.py']
for n in tools:shutil.copyfile(p/'tools'/n,td/n)
names=['r538-receiver-reuse-and-source-audit.json','r538-loopback-server-root.json','r538-loopback-wire-adoption.json','r538-production-source-snapshot.json',
    'r538-exact-result-wire-source-request.json','r538-backlog-scope-update.json','r538-library-build.jsonl','r538-library-build.stderr.log','r538-library-build-accounting.json',
    'r538-targeted-tests.stdout.log','r538-targeted-tests.stderr.log','r538-targeted-tests-accounting.json','r538-incremental-wire-complete-check.json',
    'r538-partial-eof-and-binding-errors-check.json','r538-actual-loopback-receive-order-check.json']
refs=[l/n for n in names]+[td/n for n in tools]+sorted(memory.iterdir())+sorted(actual.iterdir())
refs += [t/'source/src/modern_lito'/n for n in ['mod.rs','native_source_stream.rs','native_pet_race_wire_boundary.rs','native_pet_race_wire_boundary_tests.rs','native_pet_race_quit_session.rs']]
refs += [l/'r537-native-result-return-partial-consumer-handoff.json',p/'target/r537-native-pet-race-return-inputs/version_lifecycle_contract.json',
    p/'target/r537-native-pet-race-return-inputs/new_candidate_partial_result.json',p/'target/r535-native-pet-race-outer-inputs/outer-lifecycle-contract-v1.json']
terrain=p/'target/r534-native-pet-race-route-inputs';refs += [terrain/n for n in ['adopted-r533-component-contract.json','s32-layer3-to-actual-reader-byte-comparison.json']]+sorted((terrain/'native/566').glob('*.s32'))
report={'schema':'MODERN_R538_EXISTING_RECEIVER_KEYED_WIRE_CONSUMER_V1','state':'INCREMENTAL_KEYED_WIRE_AND_ACTUAL_OWNER_LOOPBACK_RECEIVE_REPLAY_CPU_VERIFIED_UNTAGGED_AUTHORITY_CLOSED',
 'updated_at':now,'namespace':'NATIVE_LITO_LTMAP_V1','source_profile':'5350B36DD93F2EA1FC9F00639073A1C990C759B26F424BCE704481B646BC959E',
 'production_library':pin(t/'library/liblito_modern.rlib'),'focused_test_executable':pin(exe),'source_count':657,'prior_sources_unchanged':654,
 'existing_sources_changed':source['existing_sources_changed'],'new_sources':source['new_sources'],'existing_receiver_cipher_or_R537_consumer_modified':False,
 'source_policies':['Reuses native_city_runtime::source_stream::Receiver::next/end; no second LE16/cipher implementation or guessed counter setter.',
    'Pinned5350 per-receiver binding requires exact own S_KEY and complete source wire hash/order. Foreign seed/tail, missing seed and unknown receiver rejected.',
    'Incremental bounded4096-byte input, eight-frame batch cap, existing65533-byte body buffer; generic Read preserves unconsumed bounded tail.',
    'Only complete frames decrypt and dispatch; Native counter advances by body size, excluding header and uncompleted EOF payload.',
    'FixedOfflineTrace and UntaggedReadBoundary are construction policies; generic Read refuses hypothetical policy, untagged F0/CC cannot gain return authority.',
    'EOF retains raw partial/header prefix and receiver before-state, retires receiver/scope/commands. No padding/partial application decode.',
    'Actual loopback D0 is peer departure even before D3; original per-stream ordering is preserved, with no own-confirm or cross-socket total-order inference.'],
 'checks':{'memory_complete':complete,'memory_partial_and_bindings':partial,'actual_owner_loopback_streams':loop},
 'actual_owner_loopback_adoption':adoption,'exact_missing_supply':request,
 'tests':{'passed':3,'failed':0,'old_tests_filtered':int(match[3]),'seconds':float(match[4]),'old_PASS_or_Java_server_runs_replayed':False},
 'builds':{'regular_library':1,'test_compiles':1,'regular_warnings':len(warnings),'new_component_warnings':0,'test_warnings':testwarnings},
 'remaining':['Original66ED six RESULT receive keys/stream continuity absent; EC96 different-session keys cannot repair that gap.',
    'Live WORLD/admitted roster/RUNNING/clock/dynamic occupancy, actor/form/Brave/effect and renderer producers.',
    'No full-player/authentication/GUI receipt ACK, original wire generation or catalog completion marker; other real fault/stress interleavings and production domains unqualified.',
    'Background named scene/draw source, player GUI/LOADED and24 remaining full mode qualifications.'],
 'preservation':{'prior_roots_checked':previous,'server_or_BIN_writes':0,'historical_snapshots_changed':False},
 'execution':{'new_Java_SQL_TLS_provider_GUI':0,'new_socket_or_TCP_listener':0,'new_player_EXE':False,'old_PASS_replays':0,'own_ASCII_scratch_only':True},
 'new_full_bindings':0,'complete_modes':16,'total_modes':40,'GUI':0,'LOADED':False,'references':[pin(f) for f in refs],'overall_completed':False}
save(root,report)
human=p/'Native-R538_既有接收器與實際串流交接.txt'
with human.open('x',encoding='utf-8') as s:
    s.write('R538：既有接收器接到R537 consumer\n\n重用原Native Receiver的LE16增量收包／Cipher／EOF，不再造decoder；657來源另封、654舊來源不變，原receiver與R537 consumer來源不改。\n'
        '明確5350／per-receiver lane：自己的actual S_KEY、固定wire hash與原始順序全核；拒絕跨session借key、缺key及錯序。完整frame才解密／交body，半包不advance receive counter，不補body。\n'
        '66ED三個memory key串流：2完整body、2partial EOF；12必要分段／合併模式驗證。新EC96核14引用／642manifest，6實際loopback接收串流／13完整body／2個5byte partial／4來源已觀察EOF，健康peer EOF未造假。\n'
        '保留healthy [72,0F,D3,D0] 或[72,D0,0F,D3]，quitter[F0,CC]或[F0,D0,CC]；D0僅peer離開，沒有當自己的RESULT confirm。\n'
        'FixedOfflineTrace與UntaggedReadBoundary分開建立；一般Read拒絕hypothetical政策，untagged F0／CC維持Unknown。半包EOF保留原prefix／receiver前狀態并關閉scope／命令。此CPU回放假設Modern收到；來源owner loopback真收到另列，非全玩家ACK。\n'
        f'3新CPU checks PASS、{match[3]}舊測試filtered、{match[4]}秒，新library／test compile各1次、新warning0；旧R537 PASS／Java／server未重跑，未重用已退休PID／listener。\n'
        '原66ED六個RESULT串流仍各缺自己S_KEY／起點counter continuity；EC96是別的session，不能補借。精確缺項另封request。\n'
        '仍16/40、GUI0、LOADEDfalse；live WORLD authority、actor/effect producer、正式收包/GUI與24模式未驗收。\n\n主交接：'+str(root)+'\n')
progress_path=p.parent/'監製進度.json';progress=read(progress_path)
progress.update({'updated_at':now,'phase':'R538_existing_incremental_receiver_and_actual_loopback_stream_replay_CPU_verified_original_RESULT_keys_live_WORLD_open',
    'overall_status':'in_progress','overall_completed':False,
    'current_task':'R538原Native增量Receiver接R537，657來源／654旧不變；三memory-key lanes12分段模式、6 actual loopback streams13body2partial／4已觀察EOF，peerD0不當自己confirm，3新PASS。Fixed trace與untagged Read隔離，未啟server/socket/GUI，不把假設Modern receipt當live權限。',
    'next_step':'原66ED六條RESULT receive session缺自己S_KEY／wire offsets或owner start-counter continuity，已有精確request；不能借EC96不同session key。live WORLD同epoch roster/RUNNING/clock/occupancy、actor/效果producer、背景scene及GUI／24模式仍待合格來源／驗證。'})
progress['r538']={'status':report['state'],'handoff':pin(root),'library':report['production_library'],'source_count':657,'new_checks_passed':3,
    'old_tests_filtered':int(match[3]),'existing_receiver_reused':True,'actual_loopback_streams':6,'actual_loopback_complete_bodies':13,
    'actual_loopback_partial_tails':2,'source_observed_EOF_applied':4,'source_missing_request':pin(l/'r538-exact-result-wire-source-request.json'),
    'generic_Read_hypothetical_policy_allowed':False,'new_component_warnings':0,'new_full_bindings':0,'complete_modes':16,'total_modes':40,'GUI':0,'LOADED':False}
progress['completed'].append('R538：既有Native Receiver→完整body→R537 consumer接線，固定session/key與untagged Read政策隔離；6 actual loopback streams／13body／2partial維持原序，3新CPU PASS，GUI0。')
progress['verified'].append(f'R538 3PASS0FAIL／{match[3]}filtered／{match[4]}s，新warning0；14引用／642manifest、12split/merge、peerD0/EOF拒絕誤判，原66ED RESULT key仍缺。')
with progress_path.open('w',encoding='utf-8') as s:json.dump(progress,s,ensure_ascii=False,indent=2);s.write('\n')
print(json.dumps({'handoff':pin(root),'library':report['production_library'],'tests':report['tests'],'builds':report['builds']},ensure_ascii=False))
