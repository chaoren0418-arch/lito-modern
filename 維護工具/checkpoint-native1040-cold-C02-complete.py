"""Record limited current-source cold C02 proof after independent qualification."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(path):return json.loads(Path(path).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(path,value):Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
parser=argparse.ArgumentParser();parser.add_argument('run',type=Path);args=parser.parse_args()
run=args.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native1040-cone-cold-cohort')
c=read(run/'cohort.json');qp=run/'qualified/qualified.json';q=read(qp)
assert c['status']=='executed_current_cold_C02_cohort_qualification_pending' and c['owned_process']is None
assert q['status']=='independently_qualified_current1040_cold_C02_six_source_body_cases'
assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['original_exit_code']==0 and q['recorded_owned_all_terminal']
assert (q['actual_cases'],q['actual_sessions'],q['actual_requests'],q['actual_accepted_casts'],q['actual_refused_requests'])==(6,24,10,8,2)
assert q['normal_sources_body_compiled']and not q['source_overrides_used']and not q['normal_EXE_runtime_inferred']
assert not q['GUI_verified']and not q['physical_audio_verified']and not q['whole_goal_complete']
assert sha(qp.parent/'cohort-raw.zip')==q['raw_archive_sha256']
with zipfile.ZipFile(qp.parent/'cohort-raw.zip')as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in q['raw_files'].items())
images=[]
for key,proof in q['cases'].items():
    p=Path(proof['path']);case=read(p);assert sha(p)==proof['sha256']and case['source_manifest_sha256']==c['source_manifest_sha256']
    assert sha(p.parent/'raw.zip')==case['raw_archive_sha256']
    with zipfile.ZipFile(p.parent/'raw.zip')as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in case['raw_files'].items())
    if key.endswith('-moving'):
        movement=proof['movement'];assert sha(movement['path'])==movement['sha256'];m=read(movement['path'])
        assert m['admission_qualification_sha256']==sha(p)and m['impact_anchor']==m['target_after']and m['wire_captured_destination']==m['target_before']and not m['performance_acceptance']
    ip=Path(case['run'])/'image-observation.json';image=read(ip)
    assert image['qualification_sha256']==sha(p)and image['source_manifest_sha256']==c['source_manifest_sha256']
    assert image['image_count']>0 and not image['GUI_verified']and not image['physical_audio_verified']
    images.append(dict(case=key,path=str(ip),sha256=sha(ip),matched_images=image['image_count'],actually_viewed=image['actually_viewed_count']))
assert sum(row['actually_viewed']for row in images)>=4
for name,digest in read(run/'compiled-inputs.json').items():assert sha(ROOT/name)==digest
before=read(run/'active-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/name)==digest for name,digest in before.items())
for name in before:
    if name.endswith('.json'):assert read(ROOT/name)['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
out=run/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for name in before:z.write(ROOT/name,name)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in before.items())
record=dict(status='current_normal_source_body_cold_C02_both_profiles_qualified_limited',run=str(run),qualification=str(qp),qualification_sha256=sha(qp),source_manifest_sha256=c['source_manifest_sha256'],original_session_id=q['original_session_id'],original_exit_code=0,terminal_tool_chunk=q['terminal_tool_chunk'],actual_cases=6,actual_sessions=24,actual_requests=10,actual_accepted_casts=8,actual_refused_requests=2,actual_diagnostic_executables=4,normal_sources_body_compiled=True,source_overrides_used=False,normal_EXE_runtime_inferred=False,recorded_owned_identities=q['recorded_owned_identities'],recorded_owned_all_terminal=True,cases=q['cases'],diagnostic_images=images,unplayed_source_audio_cross_endpoint_rejection_verified=False,controlled_local_S21_replay_executed=False,metadata_clarification='Client footer retains the old controlled-S21 descriptive field. Cold probes only emit that field; no local S21 replay is executed. Actual runner controlled_visual_night_fixture is false; original raw files remain unchanged.',GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源4E1F7A4E之正常來源本體冰錐診斷已完成兩版正向／拒絕／移動目標六案，原'+str(q['original_session_id'])+'actual exit0；24 LOGIN/GAME sessions、10個C14請求／8接受2拒絕、S23與扣魔／一次命中、FakeSink sound715及兩個實際C0A／遠端S12移動之固定飛行與當前命中錨點均獨立核對，'+str(q['recorded_owned_identities'])+'記錄身分終止。僅當版來源本體C02診斷，非正常EXE／GUI／實體音效資格。正常兩版各90相關檢查、當版控制器與Presentation各12案／96sessions維持其範圍；35辨識／39缺口仍是靜態請求範圍。下一步當版廣域112整合目標＋主程式、完整library，再正常GUI與必要資源／輸入音效／可攜交付；未播放舊音效跨端點拒絕、同端點與其他服務及完整G06／G01–G12仍待。既有S21說明欄位不代表本地重播；原raw保留。8／60秒、Release與G11預算不變，登入器與重建版產物獨立，相鄰只讀。'
for name in before:
    p=ROOT/name
    if p.suffix=='.json':
        value=read(p);value['current_native1040_cold_C02']=record
        value['current_native1040_cold_C02_active'].update(status='completed_qualified_source_body_scope',qualification=str(qp),qualification_sha256=sha(qp),original_exit_code=0,terminal_tool_chunk=q['terminal_tool_chunk'])
        value.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        if 'source_freeze_for_current_G06_controller'in value:value['source_freeze_for_current_G06_controller']['frozen']=False
        if 'current_native_retired_join_normal'in value:value['current_native_retired_join_normal'].update(current_source_C02_runtime_verified=True,current_source_C02_scope='cold skill38 exact source-body diagnostics only',normal_EXE_runtime_inferred=False,current_source_C02_qualification=str(qp),current_source_C02_qualification_sha256=sha(qp))
        value['current_focus']=dict(item='current1040-broad-and-library-default',work=work,applies_to_current_source=True,goal_complete=False)
        if name.endswith('next-validation-action.json'):value['next_action']=work
        save(p,value)
    else:
        first,separator,rest=p.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18')and separator
        p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json',dict(before_files=before,after_files={name:sha(ROOT/name)for name in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/name)==digest for name,digest in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json',dict(**record,checkpoint=str(out),must_poll_original_session_before_restarting=False,next_action='Run exact-source broad112 integrations plus main, then full library; keep all normal GUI and physical gates open.'))
print(json.dumps(dict(checkpoint=str(out),actual_sessions=24,actual_accepted_casts=8,whole_goal_complete=False),ensure_ascii=False))
