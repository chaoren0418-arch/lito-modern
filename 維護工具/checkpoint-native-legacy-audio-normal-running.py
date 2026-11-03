"""Register the original in-flight normal build, retaining all eight old records."""
from pathlib import Path
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
run=V/'native-legacy-audio-normal/validation-20261018-104256-823814';c=read(run/'cohort.json');app=Path(c['application'])
assert c['status']=='building_legacy_audio_normal'
for n,h in read(run/'compiled-inputs.json').items():assert sha(ROOT/n)==h
before=read(app/'tracker-readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/'active-checkpoint';out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
e=dict(status='running_original_normal_build_and_related_regressions',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],
       original_session_id=78702,original_runner=read(run/'original-runner-process.json'),current_profile=c['active_profile'],
       snapshot_owned_process=c['owned_process'],original_terminal_exit_code=None,normal_related_validation_complete=False,
       compiled_source_archive=c['compiled_source_archive'],integration_targets_each=17,selected_library_groups_each=2,
       normal_GUI_runtime_inferred=False,whole_goal_complete=False)
save(out/'evidence.json',e)
work='來源8CC5A560已整合一般WAV場景範圍與一次性交付，原78702正經唯一入口重建正常Debug／Release及每版17整合目標與2選定library群組；須沿原程序等待實際退出，不從候選移記正常產物資格。候選跨端點兩版12sessions與環境延遲／取消各2案已獨立核對，診斷C_ACTION入口未套入。新版本G06、廣域／完整library、正常GUI及實體音效仍待；8／60秒、Release／G11預算不變，登入器與重建版獨立且相鄰只讀，完整G01–G12未完成。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        x['current_native_legacy_audio_normal_active']=e
        x.update(active_workloads={'legacy_audio_normal':str(run)},active_workload_sessions={'legacy_audio_normal':78702},
                 source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        x['current_focus']=dict(item='legacy-audio-normal-build-and-regression',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
print(json.dumps(dict(checkpoint=str(out),session=78702,terminal_exit_code=None),ensure_ascii=False))
