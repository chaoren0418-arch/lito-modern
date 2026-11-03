"""Record bounded current evidence without promoting historical or pending jobs."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, zipfile

ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def write(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')

natural=ROOT/'驗證紀錄/current/native-real-peers/20261016-170537-478-rental-hotel-expiry/independent-natural-qualified/qualified.json'
terminal=natural.parent.parent/'terminal-completion-observation.json'
resources=ROOT/'驗證紀錄/current/native-resource-optin/current1021-ten-bodies-20261016-185523-877619/qualified.json'
n=read(natural);t=read(terminal);o=read(resources)
assert n['status']=='independently_qualified_immutable_Release_real_time_hotel_expiry'
assert n['maps']==[501,503] and n['actual_formal_sessions']==4 and n['original_world_PNGs']==8
assert t['actual_terminal_exit_code']==0 and t['original_session_id']==15175 and t['qualification_sha256']==sha(natural)
assert o['status']=='independently_readback_qualified_current_ten_unique_original_resource_bodies_both_profiles'
assert o['unique_previously_ignored_bodies']==10 and o['actual_test_invocations']==20 and o['actual_test_executables']==2
assert all(p['passed_optin_bodies']==10 and p['failed']==0 for p in o['profiles'].values())
assert o['other_library_ignored_bodies_not_qualified_by_these_scopes']==463
middle=ROOT/'驗證紀錄/current/native-real-peers/20261016-190144-533-rental-hotel-middle-expiry'
identities=read(middle/'owned-process-observation.json')
assert {v['ProcessId'] for v in identities}=={145248,142688,145104,143592}
start=read(middle/'natural-expiry-start.json');assert start['exe_sha256']==n['exe_sha256'] and start['jar_sha256']==n['jar_sha256']
assert sha(start['immutable_exe'])==n['exe_sha256']
cohort=ROOT/'驗證紀錄/current/native-default109/descriptor-continuation-20261016-182724-792'
completion=cohort/'completion-observation.json'
done=completion.is_file()
if done:
    d=read(completion);assert d['original_session_id']==75822 and d['observed_terminal_exit_code']==0
    assert d['exact_owned_PIDs_and_creation_times_all_terminal']
else:
    assert read(cohort/'cohort.json')['status']=='continuing_current_default109_with_exact_descriptor_pair_readback'
release=ROOT/'驗證紀錄/current/default-native-regression/20261016-182731-649-Release'
rr=read(release/'receipt.json')
now=datetime.now(timezone.utc); stamp=now.strftime('%Y%m%d-%H%M%S-%f')
out=ROOT/'驗證紀錄/current/native-hotel-current'/('natural-and-optins-progress-'+stamp);out.mkdir()
paths=[ROOT/p for p in ['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','README.md','docs/HISTORY.md']]
before={p.relative_to(ROOT).as_posix():sha(p) for p in paths}
archive=out/'progress-before.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for p in paths:z.write(p,p.relative_to(ROOT).as_posix())
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist())==set(before)
    for name,h in before.items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h

following=('當版Debug109整合＋main的110實際EXEs已480/0/21獨立核對，原Debug未重跑；'+
    ('Release已由原75822實際exit0並完成獨立退出觀察，兩版counts以completion-observation.json為準。' if done else 'Release仍沿原75822執行，不能重啟、改來源或四份已pin QA，等實際terminal後才執行observe-native-current-default109-completion.py。')+
    '固定正常Release139B/C02/TW+extended的旅館map501一小時、503兩小時自然到期已獨立核對4sessions/8PNG/9key removals/4S5A/4C08，原15175exit0且owned退出；兩張回村圖已目視。'+
    '缺少的map502兩個獨立一小時租約（4/2把鑰匙）沿原26024執行，未改時鐘/權威，未到期不得記PASS。'+
    '當版1021 canonical93384350的10unique原資源opt-in本文兩版各10過0敗，20invocations/2實際EXEs；54737/30496exit0。'+
    'default473ignored不改，另外463本文不在這兩個scope資格內；Disintegrate root pack缺失，未執行第六scene本文。'+
    '正常GUI/physical/IME/多DPI、同端點、其他服務、必要ignored、可攜/穩定性及完整G01–G12仍待。重建版與登入器為獨立專案，登入器只讀參考，不替換或部署其產物。')
natural_summary=dict(status=n['status'],qualification=str(natural),sha256=sha(natural),terminal_observation=str(terminal),original_session_id=15175,terminal_exit_code=0,maps=[501,503],actual_formal_sessions=4,original_world_PNGs=8,exact_expired_key_removals=9,exact_S5A=4,exact_C08=4,normal_Release_sha256=n['exe_sha256'],jar_sha256=n['jar_sha256'],client_and_server_real_time_ms={v['role']:{'expected_seconds':v['expected_seconds'],'client_ms':v['client_ms'],'server_ms':v['server_ms']} for v in n['checks']},raw_readback_files=len(n['raw_files']),exact_original_owned_processes_all_terminal=True,GUI_verified=False,whole_goal_complete=False)
optin_summary=dict(status=o['status'],qualification=str(resources),sha256=sha(resources),source_files=1021,source_manifest_sha256=o['source_manifest_sha256'],profiles=o['profiles'],unique_previously_ignored_bodies=10,actual_test_invocations=20,actual_test_executables=2,actual_formal_sessions=0,default_ignored_count_unchanged=True,other_library_ignored_bodies_not_qualified_by_these_scopes=463,sixth_scene_Disint_test_qualified=False,GUI_verified=False,audio_device_verified=False,all_resources_or_ignored_bodies_verified=False)
pending=dict(status='original_middle_case_real_one_hour_expiry_running_not_qualified',session_id=26024,case=str(middle),immutable_exe=start['immutable_exe'],exe_sha256=start['exe_sha256'],jar_sha256=start['jar_sha256'],keys_by_role={'a':4,'b':2},expected_seconds_by_role={'a':3600,'b':3600},maps=[502],process_identity_observation=str(middle/'owned-process-observation.json'),processes=identities,client_identities=str(middle/'terminal-client-identities.json'),expected_qualifier=str(ROOT/'維護工具/qualify-native-current-hotel-natural-expiry.py'),clock_or_authority_injection=False,whole_long_test_qualified=False)
for p in paths[:4]:
    v=read(p)
    if v.get('current_original_resource_optins',{}).get('qualification')!=str(resources):
        v['retained_original_resource_optins_before_current1021_ten']=v.get('current_original_resource_optins')
    v.update(current_original_resource_optins=optin_summary,native_hotel_current_outer_natural_expiry=natural_summary,native_hotel_current_middle_natural_expiry=pending,
        native_hotel_current_runtime_work={'status':'outer_natural_expiry_qualified_middle_original_session_running','completed_outer':natural_summary,'pending_middle':pending},
        active_workload_sessions={'hotel_middle_natural_expiry':26024}|({} if done else {'current_default109_Release_continuation':75822}),
        completed_current_resource_optin_sessions=[54737,30496],completed_outer_hotel_natural_expiry_session=15175,
        source_freeze_for_live_default109_job=not done,next_task=following,next_focus=following,
        current_focus={'item':'G06.3d-current-runtime-and-retirement','work':following,'goal_complete':False},
        last_goal_turn_classification='progress',last_goal_turn_evidence=[str(natural),str(resources),str(middle/'owned-process-observation.json')],
        progress_updated_utc=now.isoformat(),updated_date='2026-10-17',whole_goal_complete=False)
    if 'goal_complete' in v:v['goal_complete']=False
    if 'native_hotel_current_one_hour_partial' in v:
        v['native_hotel_current_one_hour_partial']['historical_partial_record']=True
        v['native_hotel_current_one_hour_partial']['superseded_by_full_case_qualification']=str(natural)
        v['native_hotel_current_one_hour_partial']['A_two_hour_result_pending']=False
    if done:
        v['current_default109'].update(status=d['status'],completion_observation=str(completion),completion_observation_sha256=sha(completion),Release_qualified=True,profiles=d['profiles'],source_and_corresponding_QA_frozen=False,owned_process=None)
    else:
        v['current_default109'].update(current_build_phase=rr['status'],owned_process=rr.get('owned_process'),Release_qualified=False)
    write(p,v)

line=('2026-10-17：固定正常Release139B5247/C02/TW+extended旅館map501一小時、503兩小時自然到期已獨立核對，4實際sessions/8原PNG/9到期key移除/4S5A/4C08，原15175實際exit0且owned全退，兩張回村圖已目視。當版1021的10unique原資源opt-in本文兩版各10過0敗（20invocations/2實際EXEs），其餘463不在此scope資格內；Disintegrate未執行。map502一小時沿原26024、Release default109沿75822；正常GUI/physical及完整G01–G12仍待。')
if done:line=line.replace('Release default109沿75822','Release default109已沿原75822完成獨立terminal核對')
p=paths[4];text=p.read_text(encoding='utf-8-sig');heading,rest=text.split('\n',1)
p.write_text(heading+'\n\n'+line+' 見[自然到期限定資格](驗證紀錄/current/native-real-peers/20261016-170537-478-rental-hotel-expiry/independent-natural-qualified/qualified.json)與[十項原資源限定資格](驗證紀錄/current/native-resource-optin/current1021-ten-bodies-20261016-185523-877619/qualified.json)。\n'+rest,encoding='utf-8',newline='\n')
p=paths[5];text=p.read_text(encoding='utf-8-sig');heading,rest=text.split('\n',1)
p.write_text(heading+'\n\n## 2026-10-17 當版原資源十項與固定正常主的真實旅館到期\n\n'+line+'\n\n完整進度收據：`'+out.relative_to(ROOT).as_posix()+'/progress-record.json`。首次新reader因Win32 extended path與普通path的字面祖先比較不同而停止，原碼與失敗保留；samefile核對後只修reader命名空間比較，未重跑長測或改產品。106份raw封存逐項讀回；不是GUI/physical或完整framebuffer oracle。'+rest,encoding='utf-8',newline='\n')
after={p.relative_to(ROOT).as_posix():sha(p) for p in paths}
(out/'reconciler.py').write_bytes(Path(__file__).read_bytes())
write(out/'progress-record.json',dict(status='recorded_current_natural_expiry_and_ten_resource_optins',before_archive_sha256=sha(archive),before_files=before,after_files=after,natural_qualification_sha256=sha(natural),resource_qualification_sha256=sha(resources),Release_continuation_terminal_qualified=done,original_middle_session_id=26024,whole_goal_complete=False,recorded_utc=now.isoformat()))
for p in paths[:4]:
    v=read(p);assert not v['whole_goal_complete'] and v['active_workload_sessions']['hotel_middle_natural_expiry']==26024
print(json.dumps({'record':str(out/'progress-record.json'),'natural_maps':[501,503],'resource_unique_bodies':10,'middle_session':26024,'Release_completion_observed':done},ensure_ascii=False))
