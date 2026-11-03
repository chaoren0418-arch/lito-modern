"""Apply the qualified five-body v3 and eight public resource regressions within ROOT."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,re,zipfile
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
old=V/'native-scoped-catalog-normal/validation-20261018-132210-585372';source=read(old/'source-inputs.json');inputs=read(old/'compiled-inputs.json');old_sha=sha(old/'source-inputs.json');assert(len(source),len(inputs))==(1040,1041)
for n,h in inputs.items():assert sha(ROOT/n)==h,n
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads'] and queue['source_and_corresponding_QA_frozen'] and queue['current_product_identity']['source_manifest_sha256']==old_sha
assert read(old/'completion-observation.json')['recorded_original_owned_processes_all_terminal']
family=V/'native33199-ordinary-admission-v3';proofs={}
for profile in ['Debug','Release']:
    matches=[]
    for run in family.iterdir():
        path=run/'qualified.json'
        if path.is_file():
            q=read(path)
            if q['scope'].get('profile','Debug')==profile:matches.append((path,q))
    assert len(matches)==1,(profile,matches);path,q=matches[0]
    assert q['scope']['passed']==5 and q['scope']['failed']==q['scope']['ignored']==0 and q['scope']['new_world_direct_Model_map_reset_verified'] and q['recorded_owned_all_terminal'] and q['observed_terminal_exit_code']==0
    assert q['source_manifest_sha256']==old_sha and not q['scope']['candidate_applied']
    proof=read(path.parent/'receipt.json');assert len(proof['source_overlays'])==5
    for n,pin in proof['source_overlays'].items():assert sha(path.parent/n)==pin['sha256']
    proofs[profile]={'path':str(path),'sha256':sha(path),'scope':q['scope']}
payload_root=ROOT/'維護工具/candidates/native33199-ordinary-admission-normal-payload';prepared=read(payload_root/'payload.json');assert not prepared['normal_sources_modified'] and len(prepared['files'])==6 and prepared['source_manifest_before']==old_sha
payload={}
for name,pin in prepared['files'].items():
    assert sha(ROOT/name)==pin['before_sha256'] and sha(pin['payload'])==pin['sha256'];assert(ROOT/name).resolve(strict=True).is_relative_to(P)
    payload[name]=Path(pin['payload']).read_bytes()
testkey='LiTo381-Modern/tests/native_world_lighting_scene.rs';assert payload[testkey].startswith((ROOT/testkey).read_bytes());assert len(re.findall(rb'#\[test\]',payload[testkey]))==20
prep=read(ROOT/'維護工具/candidates/native33199-ordinary-admission-v3/preparation.json')
for name,pin in prep['candidate_files'].items():assert sha(pin['path'])==prepared['files']['LiTo381-Modern/src/modern_lito/'+name]['sha256']
meta=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
checkpoint=V/'native1040-G06-controller/20261018-142939-945547/tracker-20261018-145215-734344/readback.json';assert all(sha(ROOT/n)==h for n,h in read(checkpoint)['after'].items())
out=V/'native-ordinary-admission-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
before={n:sha(ROOT/n)for n in [*payload,*meta]};after={n:hashlib.sha256(b).hexdigest().upper()for n,b in payload.items()}
for archive,files in [('before.zip',{n:(ROOT/n).read_bytes()for n in before}),('proposal.zip',payload)]:
    with zipfile.ZipFile(out/archive,'x',zipfile.ZIP_DEFLATED)as z:
        for n,b in files.items():z.writestr(n,b)
    with zipfile.ZipFile(out/archive)as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==hashlib.sha256(b).hexdigest().upper()for n,b in files.items())
state=dict(status='prepared_ordinary_admission_normal_application',applied=False,before_source_manifest_sha256=old_sha,before_files=before,after_files=after,new_files=[],candidate_qualifications=proofs,before_archive_sha256=sha(out/'before.zip'),proposal_archive_sha256=sha(out/'proposal.zip'),payload_manifest_sha256=sha(payload_root/'payload.json'),normal_validation_complete=False,diagnostic_request_API_applied=False,resource_instrumentation_applied=False,normal_GUI_verified=False,physical_audio_verified=False,launcher_artifacts_modified=False,whole_goal_complete=False)
save(out/'application.json',state);(out/'apply-tool.py').write_bytes(Path(__file__).read_bytes())
for n,h in inputs.items():assert sha(ROOT/n)==h
for n,b in payload.items():
    temp=out/Path(n).name;temp.write_bytes(b);os.replace(temp,ROOT/n);assert sha(ROOT/n)==after[n];source[n]=inputs[n]=after[n]
save(out/'applied-source-inputs.json',dict(sorted(source.items())));save(out/'applied-compiled-inputs.json',dict(sorted(inputs.items())))
state.update(status='applied_ordinary_admission_pending_normal_build_and_current_source_validation',applied=True,source_manifest_sha256=sha(out/'applied-source-inputs.json'),source_files=1040,compiled_input_files=1041)
save(out/'application.json',state);save(out.parent/'active-application.json',dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256']))
work='已將同世界的S4A／角色動作提交游標與不含資源的動畫歷史整合五個正常Runtime檔，保留新世界換圖重設；原12個資源場景測試逐位元前綴保留，新增8個回歸。v3兩版各5案候選已獨立核對，v2換圖死亡姿勢缺陷及正常對照保留，v2未套入。正常新來源兩版唯一入口建置、17整合及4選定library群組待驗；此前33199兩版156及G06控制器12案96sessions只保留原來源資格。8／60秒與Release／G11界限不變；相鄰只讀、登入器及重建版產物獨立，正常GUI／實體音效及完整G01–G12未完成。'
for n in meta:
    path=ROOT/n
    if path.suffix=='.json':
        j=read(path);identity=j['current_product_identity'];assert identity['source_manifest_sha256']==old_sha;j['retained_33199_product_identity']=identity;j['retained_33199_goal_validation']=j.get('current_source_goal_validation')
        for k,v in j.items():
            if k.startswith('current_native')and isinstance(v,dict)and v.get('source_manifest_sha256')==old_sha:v.update(applies_to_current_source=False,retained_source_version_only=True)
        j['current_product_identity']=dict(source_manifest_sha256=state['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,source_qualification_pending=True,Debug_normal_sha256=identity['Debug_normal_sha256'],Release_normal_sha256=identity['Release_normal_sha256'],existing_normal_source_manifest_sha256=old_sha,Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        j['current_native_ordinary_admission_application']=dict(path=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=state['source_manifest_sha256'],normal_validation_pending=True,candidate_qualifications=proofs)
        j['current_source_goal_validation']={key:dict(state='ordinary_admission_current_source_validation_pending',complete=False)for key in(j.get('current_source_goal_validation')or{})}
        for item in j.get('items',[]):
            if item['id'].startswith(('G01-','G03-','G06-')):item.update(state='ordinary_admission_current_source_validation_pending',complete=False)
        j.update(source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'application.json'),goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='ordinary-admission-normal-build-and-source-G06',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        save(path,j)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18')and sep;path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'tracker-readback.json',dict(after_files={n:sha(ROOT/n)for n in meta},source_manifest_sha256=state['source_manifest_sha256']))
print(json.dumps(dict(application=str(out),source_manifest_sha256=state['source_manifest_sha256'],changed_files=6,normal_validation_complete=False),ensure_ascii=False))
