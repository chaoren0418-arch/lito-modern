"""Apply only the four qualified lifecycle bodies and resource regressions."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=V/'native-retired-join-normal/validation-20261018-063154-317211'
source=read(normal/'source-inputs.json');inputs=read(normal/'compiled-inputs.json');assert(len(source),len(inputs))==(1040,1041)
q=read(ROOT/'工作佇列.json');assert not q.get('active_workloads') and not q.get('source_and_corresponding_QA_frozen')
before_manifest=sha(normal/'source-inputs.json');assert q['current_product_identity']['source_manifest_sha256']==before_manifest
for n,h in inputs.items():assert sha(ROOT/n)==h,n
candidate=ROOT/'維護工具/candidates/native-legacy-audio-lifecycle/20261018-095648-877889';prep=read(candidate/'preparation.json')
resource=V/'native-legacy-audio-lifecycle-tests/20261018-104039-878327';rq=read(resource/'qualified.json')
assert rq['recorded_owned_all_terminal'] and all((v['passed'],v['failed'],v['ignored'])==(2,0,0) for v in rq['profiles'].values())
pins={str(resource/'qualified.json'):sha(resource/'qualified.json')}
for v in rq['profiles'].values():
    pins[v['previous_qualification']]=v['previous_qualification_sha256'];assert sha(v['previous_qualification'])==v['previous_qualification_sha256']
payload={}
for name,pin in prep['source_overlays'].items():
    assert sha(pin['candidate_path'])==pin['candidate_sha256'] and sha(pin['original_path'])==pin['original_sha256']
    payload[Path(pin['original_path']).relative_to(ROOT).as_posix()]=Path(pin['candidate_path']).read_bytes()
test=P/'tests/native_world_lighting_scene.rs';rp=read(resource/'preparation.json')
assert sha(test)==rp['original_sha256'] and sha(resource/'tests.rs')==rq['candidate_test_sha256']
payload[test.relative_to(ROOT).as_posix()]=(resource/'tests.rs').read_bytes();assert len(payload)==5
previous=V/'native1040-teleport-legacy-audio/run-20261018-103201-021942/qualified-checkpoint-20261018-103442-681207/readback.json'
meta=read(previous)['after_files'];assert len(meta)==8 and all(sha(ROOT/n)==h for n,h in meta.items())
out=V/'native-legacy-audio-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
before={n:sha(ROOT/n) for n in [*payload,*meta]}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
after={n:hashlib.sha256(b).hexdigest().upper() for n,b in payload.items()}
with zipfile.ZipFile(out/'proposal.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,b in payload.items():z.writestr(n,b)
with zipfile.ZipFile(out/'proposal.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in after.items())
state=dict(status='prepared_legacy_audio_normal_application',applied=False,before_source_manifest_sha256=before_manifest,
           before_files=before,after_files=after,new_files=[],qualification_pins=pins,before_archive_sha256=sha(out/'before.zip'),
           proposal_archive_sha256=sha(out/'proposal.zip'),normal_validation_complete=False,diagnostic_request_API_applied=False,
           GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'application.json',state);(out/'apply-tool.py').write_bytes(Path(__file__).read_bytes())
for n,h in inputs.items():assert sha(ROOT/n)==h
for n,b in payload.items():
    target=ROOT/n;assert target.resolve(strict=True).is_relative_to(P)
    temp=out/Path(n).name;temp.write_bytes(b);os.replace(temp,target);assert sha(target)==after[n];inputs[n]=source[n]=after[n]
save(out/'applied-source-inputs.json',dict(sorted(source.items())));save(out/'applied-compiled-inputs.json',dict(sorted(inputs.items())))
state.update(status='applied_legacy_audio_pending_normal_build_and_current_source_validation',applied=True,
             source_manifest_sha256=sha(out/'applied-source-inputs.json'),source_files=1040,compiled_input_files=1041)
save(out/'application.json',state);save(out.parent/'active-application.json',dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256']))
work='已將一般WAV音效的場景範圍與一次性交付接入正常來源，保留play_legacy播放規則；環境延遲與取消退役兩版各2案、候選真實C1F／S20跨端點兩版12sessions已獨立核對。只套四個Runtime模組及一個資源回歸測試，診斷C_ACTION請求API未套入。新來源正常兩主及相關測試／最終來源G06尚待重建重驗；4E1舊資格保留原版本，不移記。8／60秒、Release與G11預算不變；登入器及伺服器相鄰來源只讀、產物獨立，正常GUI／實體音效與完整G01–G12未完成。'
for n in meta:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);old=value['current_product_identity'];assert old['source_manifest_sha256']==before_manifest
        value['retained_source_4E1_product_identity']=old;value['retained_source_4E1_goal_validation']=value.get('current_source_goal_validation')
        for key,item in value.items():
            if key.startswith('current_native') and isinstance(item,dict) and item.get('source_manifest_sha256')==before_manifest:item.update(applies_to_current_source=False,retained_source_version_only=True)
        value['current_product_identity']=dict(source_manifest_sha256=state['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,
            source_qualification_pending=True,Debug_normal_sha256=old['Debug_normal_sha256'],Release_normal_sha256=old['Release_normal_sha256'],
            existing_normal_source_manifest_sha256=before_manifest,Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        value['current_native_legacy_audio_application']=dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256'],normal_validation_pending=True)
        value['current_source_goal_validation']={key:dict(state='legacy_audio_current_source_validation_pending',complete=False) for key in(value.get('current_source_goal_validation') or {})}
        for item in value.get('items',[]):
            if item['id'].startswith(('G01-','G03-')):item.update(state='legacy_audio_current_source_validation_pending',complete=False)
        value.update(source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'application.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='legacy-audio-normal-build-and-final-source-G06',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8')
save(out/'tracker-readback.json',dict(after_files={n:sha(ROOT/n) for n in meta},source_manifest_sha256=state['source_manifest_sha256']))
assert all(sha(ROOT/n)==h for n,h in read(out/'tracker-readback.json')['after_files'].items())
print(json.dumps(dict(application=str(out),source_manifest_sha256=state['source_manifest_sha256'],changed_files=5,normal_validation_complete=False),ensure_ascii=False))
