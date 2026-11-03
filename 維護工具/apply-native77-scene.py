"""Apply backed-up native scene integration; normal validation remains pending."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,zipfile
ROOT=Path(__file__).resolve().parents[1];PROJECT=ROOT/'LiTo381-Modern';SRC=PROJECT/'src/modern_lito'
CANDIDATE=ROOT/'維護工具/candidates/native77-scene-integration'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads'] and not queue.get('active_workload_sessions')
assert not {k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in {'RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'})}
p=read(CANDIDATE/'preparation.json');inputs=read(p['baseline_inputs']);source_inputs=read(Path(p['baseline_inputs']).parent/'source-inputs.json')
for name,h in inputs.items():assert sha(ROOT/name)==h,name
assert queue['current_product_identity']['source_manifest_sha256']==p['baseline_source_manifest_sha256']
names=list(p['baseline'])+p['new_sources'];assert len(names)==len(set(names))==10
for name in p['baseline']:assert sha(SRC/name)==p['baseline'][name]
for name in p['new_sources']:assert not (SRC/name).exists()
for name in names:
    assert (CANDIDATE/name).resolve(strict=True).is_relative_to(ROOT)
    assert (SRC/name).resolve().is_relative_to(ROOT)
    for part in [(SRC/name),*(SRC/name).parents]:
        if part==ROOT.parent:break
        assert not part.is_symlink() and not part.is_junction()
normal=Path(p['baseline_inputs']).parent;c=read(normal/'cohort.json')
for info in c['profiles'].values():assert sha(info['normal']['path'])==info['normal']['sha256']
jar=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar';jar_sha=sha(jar)
assert jar_sha=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
metadata=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json',
    '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
app=ROOT/'驗證紀錄/current/native77-scene-integration'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));app.mkdir(parents=True)
old={n:sha(ROOT/n) for n in metadata+[str((SRC/name).relative_to(ROOT)).replace('\\','/') for name in p['baseline']]}
with zipfile.ZipFile(app/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in old:z.write(ROOT/name,name)
with zipfile.ZipFile(app/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in old.items())
payload={name:(CANDIDATE/name).read_bytes() for name in names};after={name:hashlib.sha256(b).hexdigest().upper() for name,b in payload.items()}
with zipfile.ZipFile(app/'proposal.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name,b in payload.items():z.writestr(name,b)
with zipfile.ZipFile(app/'proposal.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in after.items())
state=dict(status='prepared_native77_scene_integration',before_source_manifest_sha256=p['baseline_source_manifest_sha256'],before=p['baseline'],after=after,new_sources=p['new_sources'],
    before_archive_sha256=sha(app/'before.zip'),proposal_archive_sha256=sha(app/'proposal.zip'),normal_products_before=c['profiles'],reference_jar_sha256=jar_sha,
    applied=False,normal_product_validation_complete=False,new_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
save(app/'application.json',state);(app/'apply-tool.py').write_bytes(Path(__file__).read_bytes())
for name,h in inputs.items():assert sha(ROOT/name)==h,name
for name,b in payload.items():
    temporary=app/('payload-'+name);temporary.write_bytes(b);os.replace(temporary,SRC/name);assert sha(SRC/name)==after[name]
for name,h in after.items():
    rel=(SRC/name).relative_to(ROOT).as_posix();source_inputs[rel]=h;inputs[rel]=h
assert len(source_inputs)==1030 and len(inputs)==1031
save(app/'applied-source-inputs.json',source_inputs);save(app/'applied-compiled-inputs.json',inputs)
state.update(status='applied_native77_scene_pending_normal_validation',applied=True,source_manifest_sha256=sha(app/'applied-source-inputs.json'),source_files=1030,compiled_input_files=1031)
save(app/'application.json',state);save(app.parent/'active-application.json',dict(path=str(app),application_sha256=sha(app/'application.json'),source_manifest_sha256=state['source_manifest_sha256']))
work='究極光裂術已接原生場景、Screen555／Multiply555片段及S23延後hurt；啟動記錄跨模型取消／來源切換保留，音效接有限payload與全域去重，來源已變更且正常兩版／相關測試待驗。舊78A49D5D兩主與隔離候選只保留其限定資格，不移記為當版通過。接著唯一入口正常Debug／Release建置與相關本文，再驗C14權威／ack／冷卻及實際C02施放。無新sessions或施法資格；其他技能／legacy、完整G06／同端點與G01–G12、GUI實體輸入音效／G11G12仍待。登入器唯讀、產物獨立，原時限與預算不變。'
for name in metadata[:5]:
    path=ROOT/name;j=read(path);previous=j['current_product_identity'];j['retained_source78_product_identity']=previous
    j['current_product_identity']=dict(source_manifest_sha256=state['source_manifest_sha256'],source_files=1030,compiled_input_files=1031,source_qualification_pending=True,
        Debug_normal_sha256=previous['Debug_normal_sha256'],Release_normal_sha256=previous['Release_normal_sha256'],existing_normal_source_manifest_sha256=p['baseline_source_manifest_sha256'],
        Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,qualification_scope='pending_native77_scene_related',GUI_verified=False,whole_goal_complete=False)
    for key,value in j.items():
        if key.startswith('current_native') and isinstance(value,dict) and value.get('source_manifest_sha256')==p['baseline_source_manifest_sha256']:
            value.update(applies_to_current_source=False,retained_source_version_only=True)
    j['retained_source78_goal_validation']=j.get('current_source_goal_validation')
    j['current_source_goal_validation']={key:dict(state='native77_current_source_validation_pending',complete=False) for key in (j.get('current_source_goal_validation') or {})}
    j.update(current_native77_scene_application=dict(path=str(app),application_sha256=sha(app/'application.json'),source_manifest_sha256=state['source_manifest_sha256'],applied=True,normal_validation_pending=True),
        source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(app/'application.json'),goal_complete=False,whole_goal_complete=False)
    if name==metadata[4]:j['next_action']=work
    else:j['current_focus']=dict(item='native77-current-normal-related-validation',work=work,applies_to_current_source=True,goal_complete=False)
    save(path,j)
for name in metadata[5:]:
    path=ROOT/name;first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    path.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
assert sha(jar)==jar_sha
save(app/'tracker-readback.json',dict(source_manifest_sha256=state['source_manifest_sha256'],after_files={n:sha(ROOT/n) for n in metadata},normal_products_rebuilt=False,whole_goal_complete=False))
print(json.dumps(dict(application=str(app),source_manifest_sha256=state['source_manifest_sha256'],changed_sources=len(names),new_sources=len(p['new_sources']),normal_validation_complete=False),ensure_ascii=False))
