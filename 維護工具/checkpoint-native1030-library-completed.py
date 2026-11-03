"""Publish only the observed library closure; retain every earlier scope."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native1030-library-default')
observed=run/'completion-observation.json';o=read(observed);c=read(run/'cohort.json')
assert o['status']=='independently_qualified_current1030_library_both_profiles_and_terminal_processes' and o['recorded_original_owned_processes_all_terminal']
assert o['observed_terminal_exit_code']==0 and o['cohort_sha256']==sha(run/'cohort.json')
assert c['status']=='executed_both_current_full_library_default_selections'
for profile,info in o['profiles'].items():
    path=Path(info['qualification']);q=read(path)
    assert sha(path)==info['qualification_sha256'] and q['source_manifest_sha256']==o['source_manifest_sha256']
    assert q['passed']==1943 and q['failed']==0 and q['ignored']==483 and q['restored_historical_reports']==1719
record=dict(run=str(run),original_session_id=o['original_session_id'],completion=str(observed),completion_sha256=sha(observed),source_manifest_sha256=o['source_manifest_sha256'],profiles=o['profiles'],
    full_library_qualification_pending=False,Release_qualification_pending=False,full_library_both_profiles_qualified=True,recorded_terminal_identities=o['recorded_original_owned_identities'],
    ignored_bodies_executed=False,GUI_verified=False,physical_audio_verified=False,full_G06_verified=False,whole_goal_complete=False)
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json',
    '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
out=run/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();before={name:sha(ROOT/name) for name in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in names:z.write(ROOT/name,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in before.items())
work='來源E15907AC正常兩版與1031編譯輸入維持釘選。default兩版Debug510／Release506、各0敗21忽略、各112映像與460記錄身分已核對。完整library原93199已actual exit0，兩版各1943過0敗483忽略，逐名／來源／實際映像與歷史報告還原已獨立核對，13組記錄身分終止；未執行忽略本文或推論GUI資格。保留原49342稽核473常數失敗與Debug映像續接證據。技能34隔離快照對C02共8sessions、2次成功各18MP及Sound34一次／1次MP不足拒絕，正常產品未註冊。最新11模組候選含Model來源身分與同圖號世代亮度重設，續驗明示本地S21夜間重播與退役preflight；產品仍凍結。正常GUI／實體輸入音效、必要資源、同端點及完整G06／G01–G12仍待；8／60秒、G11及Release設定維持，來源只讀與獨立產物邊界不變。'
for name in names[:5]:
    path=ROOT/name;j=read(path);assert j['current_product_identity']['source_manifest_sha256']==o['source_manifest_sha256']
    j.update(current_native1030_library=record,next_task=work,next_focus=work,source_and_corresponding_QA_frozen=True,last_goal_turn_classification='progress',last_goal_turn_evidence=str(observed),goal_complete=False,whole_goal_complete=False)
    j.get('active_workloads',{}).pop('native1030-library-default',None);j.get('active_workload_sessions',{}).pop('native1030-library-default',None)
    j.setdefault('current_source_goal_validation',{})['G03-current-regression']=dict(state='E159_default111_and_both_full_library_defaults_qualified_remaining_resource_GUI_TCP_scope_pending',complete=False,library=record)
    for item in j.get('items',[]):
        if item['id']=='G03-current-regression':
            item.update(state='E159_default111_and_both_library_defaults_qualified_remaining_resource_GUI_TCP_scope_pending',complete=False)
            evidence=observed.relative_to(ROOT).as_posix();item.setdefault('evidence',[])
            if evidence not in item['evidence']:item['evidence'].append(evidence)
    if name==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='G04-canonical34-lifecycle-candidate',work=work,applies_to_current_source=False,normal_product_registered=False,goal_complete=False)
    save(path,j)
for name in names[5:]:
    path=ROOT/name;first,separator,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and separator
    path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(record=record,before_files=before,after_files={name:sha(ROOT/name) for name in names}))
print(json.dumps(dict(checkpoint=str(out),both_library_profiles_qualified=True,whole_goal_complete=False),ensure_ascii=False))
