"""Publish exact source-matched normal/SPR scope; keep all other gates pending."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native544-integration')
done=run/'completion-observation.json';q=read(done);c=read(run/'cohort.json')
assert q['status']=='independently_observed_current_exact544_normals_and_sprite_both_profiles' and q['observed_terminal_exit_code']==0 and q['recorded_owned_all_terminal']
assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['source_files']==1028 and q['compiled_input_files']==1029
inputs=read(run/'compiled-inputs.json');assert len(inputs)==1029 and all(sha(ROOT/n)==h for n,h in inputs.items())
for profile,info in q['profiles'].items():
    assert sha(info['qualification'])==info['qualification_sha256'] and sha(c['profiles'][profile]['normal']['path'])==info['normal_sha256']
record=dict(run=str(run),completion_observation=str(done),completion_sha256=sha(done),original_session_id=q['original_session_id'],observed_terminal_exit_code=0,
    source_manifest_sha256=q['source_manifest_sha256'],source_files=1028,compiled_input_files=1029,profiles=q['profiles'],recorded_owned_identities=q['recorded_owned_identities'],recorded_owned_all_terminal=True,
    qualified=True,qualification_scope='normal_build_and_sprite_related_only',full_library_bodies_executed=False,full_G06_verified=False,actual_formal_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json',
    '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
for n in names[:5]:
    j=read(ROOT/n);assert j['current_product_identity']['source_manifest_sha256']==q['source_manifest_sha256']
    assert set(j['active_workloads'])=={'native544-normal'} and j['active_workloads']['native544-normal']['original_session_id']==q['original_session_id']
out=run/('completion-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='來源'+q['source_manifest_sha256'][:8]+'精確544空白修正已套產品；原'+str(q['original_session_id'])+'正常兩主經唯一入口建置／版本與PE x64核對，兩版各18解碼器測試＋1既有原資源本文通過並獨立讀回。全library清單2411、原忽略473維持，其他本文未由本批取得資格。三檔63影格／32空白及31一般＋31原生壓縮像素之候選證據依完全相同來源沿用，未重算為新正常程式施法。舊89D4各批只保留原版資格。下一步接34／38／45／46／77原生技能資源與玩法，再補最終版完整G06／廣域與必要回歸；同端點、GUI實體輸入音效、可攜G11G12及完整G01–G12仍待。登入器唯讀、產物獨立，原時限／預算不變。'
for n in names[:5]:
    path=ROOT/n;j=read(path);identity=j['current_product_identity']
    identity.update(source_qualification_pending=False,qualification_scope='normal_build_and_sprite_related_only')
    for profile,info in q['profiles'].items():
        identity[profile+'_normal_sha256']=info['normal_sha256'];identity[profile+'_normal_source_manifest_sha256']=q['source_manifest_sha256']
        identity[profile+'_normal_qualified_for_current_source']=True;identity[profile+'_normal_state']='independently_qualified_sprite_related_only'
    identity.pop('existing_normal_source_manifest_sha256',None)
    j.update(current_native544_normal=record,active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,
        next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=record,goal_complete=False,whole_goal_complete=False,
        progress_updated_utc=datetime.now(timezone.utc).isoformat())
    j['current_native544_application'].update(normal_validation_pending=False,normal_related_qualification=record)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='native-five-attack-integration',work=work,applies_to_current_source=True,goal_complete=False)
    for key,gate in (j.get('current_source_goal_validation') or {}).items():
        if key.startswith('G03'):gate.update(state='exact544_sprite_related_only_remaining_regressions_pending',complete=False)
    save(path,j)
for n in names[5:]:
    path=ROOT/n;first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    path.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(scope=record,before_files=before,before_archive_sha256=sha(out/'before.zip'),after_files={n:sha(ROOT/n) for n in names}))
save(Path(c['application'])/'normal-related-qualification.json',record)
print(json.dumps(dict(checkpoint=str(out),normal_mains={p:v['normal_sha256'] for p,v in q['profiles'].items()},whole_goal_complete=False),ensure_ascii=False))
