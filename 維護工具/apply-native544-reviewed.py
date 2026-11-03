"""Apply the source-pinned exact marker; retain old-version evidence as history."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads') and not queue.get('active_workload_sessions')
assert not {k:v for k,v in os.environ.items() if v and (k.startswith('CARGO_PROFILE_') or k in {'RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS'})}
candidate=queue['current_native544_candidate'];assert candidate['qualified'] and candidate['needs_application']
qp=Path(candidate['qualification']);assert sha(qp)==candidate['qualification_sha256'];q=read(qp)
assert q['candidate_unit_passed']==18 and q['candidate_unit_failed']==0 and q['normalized_empty_frames']==32
assert q['flat_pixel_matches']==q['compact_pixel_matches']==31 and q['baseline_control_actual_exit_code']==101
normal=ROOT/'驗證紀錄/current/native-selector-integration/validation-20261017-123504-346151'
before=read(normal/'source-inputs.json');compiled=read(normal/'compiled-inputs.json');assert len(before)==1028 and len(compiled)==1029
old_source_sha=sha(normal/'source-inputs.json');assert old_source_sha==q['source_manifest_sha256']==queue['current_product_identity']['source_manifest_sha256']
for n,h in compiled.items():assert sha(ROOT/n)==h,n
for info in q['normal_mains'].values():assert sha(info['path'])==info['sha256']
name='LiTo381-Modern/src/modern_lito/sprite.rs';destination=ROOT/name;src=Path(q['candidate_source'])
for path in [destination,src]:
    assert path.resolve(strict=True).is_relative_to(ROOT)
    for parent in [path,*path.parents]:
        if parent==ROOT.parent:break
        assert not parent.is_symlink() and not parent.is_junction(),str(parent)
assert sha(destination)==q['baseline_source_sha256']==before[name] and sha(src)==q['candidate_source_sha256']
jar=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
jar_sha=sha(jar);assert jar_sha=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
app=ROOT/'驗證紀錄/current/native544-integration'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));app.mkdir(parents=True)
(app/'apply-tool.py').write_bytes(Path(__file__).read_bytes())
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json',
    '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
old={n:sha(ROOT/n) for n in [name,*names]}
with zipfile.ZipFile(app/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in old:z.write(ROOT/n,n)
with zipfile.ZipFile(app/'before.zip') as z:
    assert set(z.namelist())==set(old)
    for n,h in old.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
payload=src.read_bytes()
with zipfile.ZipFile(app/'proposal.zip','x',zipfile.ZIP_DEFLATED) as z:z.writestr(name,payload)
with zipfile.ZipFile(app/'proposal.zip') as z:assert hashlib.sha256(z.read(name)).hexdigest().upper()==q['candidate_source_sha256']
state=dict(status='prepared_exact544_source_application',before_source_manifest_sha256=old_source_sha,candidate_qualification=str(qp),candidate_qualification_sha256=sha(qp),
    before={name:old[name]},after={name:q['candidate_source_sha256']},metadata_before={n:old[n] for n in names},before_archive_sha256=sha(app/'before.zip'),proposal_archive_sha256=sha(app/'proposal.zip'),
    normal_products_before=q['normal_mains'],reference_jar_sha256=jar_sha,source_files=1028,compiled_input_files=1029,applied=False,normal_product_validation_complete=False,
    new_formal_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
save(app/'application.json',state)
for n,h in compiled.items():assert sha(ROOT/n)==h,n
temporary=app/'source-payload.tmp';temporary.write_bytes(payload);os.replace(temporary,destination)
assert sha(destination)==q['candidate_source_sha256']
after=before|{name:sha(destination)};compiled_after=compiled|{name:sha(destination)}
assert [n for n in before if before[n]!=after[n]]==[name]
save(app/'applied-source-inputs.json',after);save(app/'applied-compiled-inputs.json',compiled_after)
state.update(status='applied_exact544_pending_current_normal_validation',applied=True,source_manifest_sha256=sha(app/'applied-source-inputs.json'),changed_sources=[name],applied_utc=datetime.now(timezone.utc).isoformat())
save(app/'application.json',state);save(app.parent/'active-application.json',dict(path=str(app),application_sha256=sha(app/'application.json'),source_manifest_sha256=state['source_manifest_sha256']))
work='來源'+state['source_manifest_sha256'][:8]+'已套用一檔sprite精確544零參照空白標記與非忽略保護測試；候選18測試／三檔63影格之31一般＋31壓縮像素比對已保留。正常兩版尚未重建及驗證；舊89D4廣域／library／G06／map563／原資源與64session可見性只保留原版本資格。先執行本版正常兩主及解碼器相關驗證，再接34／38／45／46／77技能資源與玩法；無新施法資格。完整G06／同端點、其餘G01–G12與GUI實體輸入音效／可攜G11G12仍待；登入器唯讀且產物獨立，原時限／預算不變。'
for n in names[:5]:
    path=ROOT/n;j=read(path);assert not j.get('active_workloads')
    previous=j['current_product_identity'];assert previous['source_manifest_sha256']==old_source_sha
    j['retained_source89_product_identity']=previous
    j['current_product_identity']=dict(source_manifest_sha256=state['source_manifest_sha256'],source_files=1028,compiled_input_files=1029,source_qualification_pending=True,
        Debug_normal_sha256=previous['Debug_normal_sha256'],Release_normal_sha256=previous['Release_normal_sha256'],existing_normal_source_manifest_sha256=old_source_sha,
        Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,qualification_scope='pending_exact544_related',GUI_verified=False,whole_goal_complete=False)
    for key,value in j.items():
        if key.startswith('current_selector') and isinstance(value,dict):value.update(applies_to_current_source=False,retained_source_version_only=True)
    j['current_native544_candidate'].update(needs_application=False,applied_to_product_source=True,application=str(app),applied_source_manifest_sha256=state['source_manifest_sha256'])
    j.update(current_native544_application=dict(path=str(app),application_sha256=sha(app/'application.json'),source_manifest_sha256=state['source_manifest_sha256'],applied=True,normal_validation_pending=True),
        source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(app/'application.json'),goal_complete=False,whole_goal_complete=False)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='exact544-current-normal-validation',work=work,applies_to_current_source=True,goal_complete=False)
    j['retained_source89_goal_validation']=j.get('current_source_goal_validation')
    j['current_source_goal_validation']={key:dict(state='exact544_current_source_validation_pending',complete=False) for key in (j.get('current_source_goal_validation') or {})}
    save(path,j)
for n in names[5:]:
    path=ROOT/n;first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    path.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
assert sha(jar)==jar_sha
save(app/'application-tracker-readback.json',dict(after_files={n:sha(ROOT/n) for n in names},production_source_sha256=sha(destination),normal_products_rebuilt=False,whole_goal_complete=False))
print(json.dumps(dict(application=str(app),source_manifest_sha256=state['source_manifest_sha256'],changed_sources=1,normal_products_rebuilt=False),ensure_ascii=False))
