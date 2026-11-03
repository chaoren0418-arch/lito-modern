import datetime
import hashlib
import json
import pathlib
import re
import shutil

p=pathlib.Path(__file__).resolve().parent.parent
l=p/'logs/continuation-20261003'
t=p/'target/r536-native-pet-race-quit-session'
root=l/'r536-native-quit-intent-lifecycle-consumer-handoff.json'
assert not root.exists()
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
 'r535-native-session-lifecycle-consumer-handoff.json':'40920AFB5207A63373FF153A28954CA6FE4B070E79D758AFB007888E4F0F395B'}
for n,h in previous.items():assert pin(l/n)['sha256']==h
out=(l/'r536-targeted-tests.stdout.log').read_text(encoding='utf-8-sig')
match=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; .*?(\d+) filtered out; finished in ([0-9.]+)s',out)
assert match and match.group(1,2)==('2','0'),out
err=(l/'r536-targeted-tests.stderr.log').read_text(encoding='utf-8-sig')
exepath=re.search(r'Running unittests .*?\(([^\r\n]+\.exe)\)',err).group(1)
exe=t/'r536-focused-tests.exe';assert not exe.exists();shutil.copyfile(p/exepath,exe)
source=read(l/'r536-production-source-snapshot.json')
for d in source['files']:
    assert pin(p/d['path'])['sha256']==d['sha256']
    assert pin(t/'source'/d['path'])['sha256']==d['sha256']
rows=[json.loads(s) for s in (l/'r536-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()]
warnings=[r['message'] for r in rows if r.get('reason')=='compiler-message' and r['message']['level']=='warning']
newwarnings=[w for w in warnings if any('native_pet_race_quit_session' in s['file_name'] for s in w['spans'])]
assert not newwarnings
assert 'native_pet_race_quit_session.rs:' not in err and 'native_pet_race_quit_session_tests.rs:' not in err
testwarnings=int(re.search(r'generated (\d+) warnings',err)[1])
assert read(l/'r536-library-build-accounting.json')['exit_code']==0
assert read(l/'r536-targeted-tests-accounting.json')['exit_code']==0
actual=read(l/'r536-actual-application-replay-check.json')
counter=read(l/'r536-quit-intent-counterexamples-check.json')
assert (actual['streams'],actual['actual_server_successful_bodies_assumed_delivered'])==(18,39)
assert (actual['local_channel_views'],actual['local_result_views'],actual['unknown_views'])==(3,9,6)
inputs=p/'target/r536-native-pet-race-quit-inputs'
adoption=read(l/'r536-quit-finalization-adoption.json')
assert adoption['manifest_files_checked']==670 and adoption['refs_checked']==11
now=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat()
audit=read(l/'r533-full-backlog-audit.json')
updates=[]
for i in audit['items']:
    row={'item':i['item'],'historical_classification':i['classification'],'remaining':i['specific_remaining_condition']}
    if i['item']=='Native_PetRace_566_569_570':
        row['current_classification']='finite_reducer_terrain_session_and_candidate_quit_consumers_implemented_live_authority_pending'
        row['remaining']='R533 progression, R534 actual Native terrain, R535 wire/local epoch and R536 candidate active quit are library components. Live roster/RUNNING/clock/occupancy, form/Brave/actor effects and scene admission remain unqualified; candidate actual method runtime only map566.'
    elif i['item']=='Native_WORLD_and_RESULT_return':
        row['current_classification']='offline_typed_result_and_active_quit_implemented_reachable_gameplay_missing_admission'
        row['remaining']='Candidate active D9 actual F0+CC offline consumer exists; RESULT ACK retains room. Candidate actual RESULT C_D0 own D0+CC successful/IO/empty-list body output is missing; no live Runtime/world/485F authority added.'
    else:row['current_classification']=i['classification']
    updates.append(row)
save(l/'r536-backlog-scope-update.json',{'schema':'MODERN_R536_BACKLOG_SCOPE_UPDATE_V1','historical_audit':pin(l/'r533-full-backlog-audit.json'),
    'items':updates,'per_mode_map':audit['per_mode_map'],'complete_modes':16,'total_modes':40,'new_full_bindings':0,'GUI':0,'LOADED':False,
    'evidence_supported_product_work_done':'Separate candidate consumer actually delegates to R535 session and R534/R533 terrain/progression; exact contract gates and local intent handling implemented. Historical R535 behavior unchanged except sibling method visibility.',
    'no_generic_WORLD_or_live_adapter_upgrade_inferred':True,'remaining_known_integration_requires_missing_sources_or_live_admission':True,'overall_completed':False})
td=t/'tools';td.mkdir()
toolnames=['prepare-r536-native-quit.py','freeze-r536-native-quit.py','build-r536-native-quit.ps1','seal-r536-native-quit.py']
for n in toolnames:shutil.copyfile(p/'tools'/n,td/n)
names=['r536-quit-finalization-server-root.json','r536-quit-finalization-adoption.json','r536-production-source-snapshot.json',
       'r536-backlog-scope-update.json','r536-library-build.jsonl','r536-library-build.stderr.log','r536-library-build-accounting.json',
       'r536-targeted-tests.stdout.log','r536-targeted-tests.stderr.log','r536-targeted-tests-accounting.json',
       'r536-actual-application-replay-check.json','r536-quit-intent-counterexamples-check.json']
refs=[l/n for n in names]+[td/n for n in toolnames]
refs += [t/'source/src/modern_lito'/n for n in ['mod.rs','native_pet_race_session.rs','native_pet_race_quit_session.rs','native_pet_race_quit_session_tests.rs']]
refs += sorted(inputs.iterdir())
refs += [l/'r535-native-session-lifecycle-consumer-handoff.json',p/'target/r535-native-pet-race-outer-inputs/outer-lifecycle-contract-v1.json']
terrain=p/'target/r534-native-pet-race-route-inputs'
refs += [terrain/n for n in ['adopted-r533-component-contract.json','s32-layer3-to-actual-reader-byte-comparison.json']]
refs += sorted((terrain/'native/566').glob('*.s32'))
report={
 'schema':'MODERN_R536_EXACT_CANDIDATE_ACTIVE_QUIT_AND_RESULT_CONSUMER_V1',
 'state':'NEW_OFFLINE_CANDIDATE_ACTIVE_D9_RESULT_ACK_CONFIRM_INTENTS_AND_UNKNOWN_RETURNS_CPU_VERIFIED',
 'updated_at':now,'namespace':'NATIVE_LITO_LTMAP_V1',
 'profile':{'candidate_source':'00FC5E67441BDAE88CAF40BDA3560F91B5254B694CA6A85A921C1E42D61603B2',
            'candidate_root':'263ABBE9B775743D5D3B581FDA21CDE6E778A30A9EA1C3301552601C7313D4F0',
            'order_contract':'E954D4F58048058C20F59F2DD08A040D97DE3EF935E9E056C6AD1AE270D25A79',
            'version_binding':'E1692FE8F679EB671D63E16B72923668B0360C32A93AE9CE4C06DE5470639566',
            'historical_013A_profile_retained':True,'historical_64A40_5ABC_not_bad_pins':True,'deployed_485F_equivalence_or_adoption':False},
 'production_library':pin(t/'library/liblito_modern.rlib'),'focused_test_executable':pin(exe),
 'source_count':653,'prior_sources_unchanged':649,'existing_sources_changed':source['existing_sources_changed'],'new_sources':source['new_sources'],
 'prior_session_behavior_change':'None; only scope/close_commands made pub(super) for sibling composition.',
 'source_policies':['Exact pinned candidate contract required for new unseeded offline predecessor; no global profile substitution.',
    'Seeded local candidate active quit prepares one-use C_D9 intent; stops prediction before request. No active C_D0.',
    'Only explicit hypothetical fixed receipt sequence of current F0 then later CC0 yields local CHANNEL view; no self-D0 fabricated.',
    'RESULT D9 acknowledge F0 retains RESULT; separate RESULT-only C_D0 confirm requires own D0 then CC0 under offline delivery assumption.',
    'Known result warning/message disables prediction pending D3; missing D3 cannot be supplied by server poststate metadata.',
    'Missing frames/zero-row list at chosen offline window boundary and untagged return bytes latch UNKNOWN; disconnect/foreign epochs remain closed.',
    'Successful server flush is recorded separately from assumed client receipt; no wire generation, membership transaction or delivery ACK proved.'],
 'adoption':adoption,'checks':{'actual_application_replay':actual,'counterexamples':counter},
 'tests':{'passed':2,'failed':0,'old_tests_filtered':int(match[3]),'seconds':float(match[4]),'prior_PASS_or_Java_methods_replayed':False},
 'builds':{'regular_library':1,'test_compiles':1,'regular_warnings':len(warnings),'new_component_warnings':len(newwarnings),'test_warnings':testwarnings},
 'precise_missing_inputs':adoption['missing'],
 'remaining':['Same-epoch live roster/RUNNING/clock/movement/dynamic occupancy and actor/form/Brave/effect publishers.',
    'Candidate server memory IOException local finalization succeeds; real partial delivery, caller disconnect/concurrency and excluded shop/trade/record/persistence domains unqualified.',
    'Actual candidate RESULT-confirm incoming bodies missing; no live WORLD admission or current backend source adoption.',
    'Background named scene/draw producer, actor/provider originals, full24 remaining modes and human GUI/LOADED.'],
 'preservation':{'prior_root_pins_checked':previous,'server_or_BIN_writes':0,'old_profile_changed':False},
 'execution':{'Java_SQL_TLS_provider_GUI':0,'new_player_EXE':False,'actual_socket_write':False,'own_ASCII_scratch_only':True,'old_PASS_replays':0},
 'new_full_bindings':0,'complete_modes':16,'total_modes':40,'GUI':0,'LOADED':False,'references':[pin(f) for f in refs],'overall_completed':False}
save(root,report)
human=p/'Native-R536_賽跑中退出與結果確認分流交接.txt'
with human.open('x',encoding='utf-8') as s:
    s.write('R536：賽跑中退出與結果確認分流\n\n'
        '新server候選263ABBE9／ClientSession00FC5E67核對11引用／670manifest，不代表485F部署。保留013A／64A40／5ABC歷史來源。\n'
        '從六種actual memory方法情境／18玩家序列擷取39成功application body，另固定來源。server flush與假設client收到分開；没有重新執行Java方法。\n'
        'active quit採一次性本地C_D9意圖，立即關閉prediction；只有明示固定離線順序下當次F0＋後續CC0才有本地Channel view，沒有D0。RESULT D9/F0仍留Result；RESULT confirm另給C_D0，但本次沒有候選實際S_D0回覆，缺失不能補造。\n'
        '18序列：3本地Channel、9Result、6Unknown；notice缺D3、F0缺list、空list／未標記晚到都保留Unknown，舊命令失效；斷線為Disconnected。完整server寫出不代表client ACK。\n'
        f'2新CPU checks PASS、{match[3]}舊測試filtered、{match[4]}秒；{counter["malformed_atomic_rejections"]}malformed原子拒絕。653來源另封，649舊來源不變；mod與舊session兩個sibling visibility修改，歷史行為不變。新library／test compile各1次、新warnings0。\n'
        'R532–535凍結與旧PASS保留。仍16/40、GUI0、LOADEDfalse；未啟SQL／provider／Native／GUI，未實際socket送出。\n\n'
        '精確缺項：candidate RESULT C_D0實際自己S_D0＋CC0（含IO／空list）、wire世代標記與零行list完成標記／期限、真partial transport及client receipt。\n'
        '其餘待辦已對照R533 audit：有限地形／reducer／session已接線；live WORLD/admission、actor／effect producer與背景scene來源仍缺，不把局部PASS當全完成。\n\n主交接：'+str(root)+'\n')
progress_path=p.parent/'監製進度.json'
progress=read(progress_path)
progress['updated_at']=now;progress['phase']='R536_candidate_active_quit_RESULT_split_CPU_verified_live_world_and_receipt_authority_open'
progress['overall_status']='in_progress';progress['overall_completed']=False
progress['current_task']='R536 exact00FC candidate consumer完成：11引用／670manifest、39 actual成功body／18序列，3本地Channel9Result6Unknown；activeD9與RESULTack/confirm分流、missing/late守門，2新PASS／653來源另封。舊013A/64A40/5ABC與R532–535保留，未宣告485F採用、client ACK或完整模式。'
progress['next_step']='候選RESULT C_D0成功自己S_D0及CC0实际body（含IO／零行列表）尚缺；同epoch live WORLD roster/RUNNING/clock/occupancy、actor/Brave效果producer與背景scene綁定待合格來源或權限。剩餘24模式及GUI/LOADED未驗，不能以server local CHANNEL或flush取代client receipt。'
progress['r536']={'status':report['state'],'handoff':pin(root),'library':report['production_library'],'source_count':653,
    'new_checks_passed':2,'old_tests_filtered':int(match[3]),'actual_successful_application_bodies':39,'offline_sequences':18,
    'local_channel_views':3,'local_result_views':9,'unknown_views':6,'candidate_manifest_checked':670,
    'new_component_warnings':0,'candidate_RESULT_confirm_actual_incoming_bodies_missing':True,
    'new_full_bindings':0,'complete_modes':16,'total_modes':40,'GUI':0,'LOADED':False}
progress['completed'].append('R536：00FC candidate activeD9／RESULT分流元件接R535既有typed wire，39實際body／18離線序列／2新CPU PASS，653來源另封；missing/late維持Unknown，GUI0。')
progress['verified'].append(f'R536 2PASS0FAIL／{match[3]}filtered／{match[4]}s，新warnings0；11引用／670manifest、39body，3Channel9Result6Unknown，{counter["malformed_atomic_rejections"]}原子拒絕。')
with progress_path.open('w',encoding='utf-8') as s:json.dump(progress,s,ensure_ascii=False,indent=2);s.write('\n')
print(json.dumps({'handoff':pin(root),'library':report['production_library'],'tests':report['tests'],'builds':report['builds']},ensure_ascii=False))
