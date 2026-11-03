"""Track the new source separately from all previously qualified versions."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-shared-archive-owner')
c=read(run/'cohort.json');pins=read(run/'compiled-inputs.json')
assert len(pins)==1025 and all(sha(ROOT/n)==h for n,h in pins.items())
assert sha(run/'source-inputs.json')==c['source_manifest_sha256']
app=Path(c['application']);application=read(app/'application.json')
assert application['source_manifest_sha256']==c['source_manifest_sha256']
done=(run/'completion-observation.json').exists();completion=read(run/'completion-observation.json') if done else None
active=c['status'] not in ['failed_preserved','executed_shared_archive_owner_related_tests_and_unique_normals']
if done:assert completion['original_session_id']==a.session_id and completion['observed_terminal_exit_code']==0
profiles={}
for profile,info in c['profiles'].items():
    q=run/profile/'profile-qualified/qualified.json';qualified=q.exists()
    profiles[profile]=dict(status='independently_qualified_related_and_normal' if qualified else 'pending_related_or_normal_qualification',
        counts=info.get('counts'),normal=info.get('normal'),qualification=str(q) if qualified else None,
        qualification_sha256=sha(q) if qualified else None)
record=dict(status=c['status'],source_manifest_sha256=c['source_manifest_sha256'],source_files=1024,compiled_input_files=1025,
    application=str(app),application_sha256=sha(app/'application.json'),run=str(run),original_session_id=a.session_id,
    source_and_corresponding_QA_frozen=active,profiles=profiles,both_related_profiles_qualified=done,
    previous_source_manifest_sha256=application['base_source_manifest_sha256'],previous_qualifications_retained_only=True,
    completion_observation=str(run/'completion-observation.json') if done else None,
    completion_sha256=sha(run/'completion-observation.json') if done else None,
    actual_new_formal_sessions=0,current_G06_qualified=False,GUI_verified=False,whole_goal_complete=False)
if (app/'normal-host-lifetime-review.json').exists():
    record['normal_host_source_lifetime_followup']=dict(path=str(app/'normal-host-lifetime-review.json'),
        sha256=sha(app/'normal-host-lifetime-review.json'),applied=False,normal_GUI_source_reuse_proven=False)
note=(f'來源{c["source_manifest_sha256"][:8]}已接場景間共用Native資源擁有者，首次完整hash與來源寫／刪鎖保留；'
    '弱引用在最後場景退出後釋放，所有限制納入識別，借用同一互斥鎖以保護seek游標，等待可取消。'
    f'原{a.session_id}之Debug／Release各13相關實際測試映像與唯一入口正常建置，當前{c["status"]}。'
    '目前僅三檔產品來源變更；原12C3之可見性64sessions、兩批G06失敗與離線診斷各保留原版，不移記新來源。'
    '八秒移動／六十秒等待／G11預算不變；同版G06、廣域／library／map563、正常GUI／實體輸入／音效／可攜交付及全G01–G12仍待。'
    '登入器及伺服器來源只讀，與重建版產物獨立。')
if record.get('normal_host_source_lifetime_followup'):
    note+='正常Presentation在場景變更時會先丟棄Model，僅本輪重疊Model測試尚不能證明正常跨場景共用；待當前建置實際終止及讀回後補獨立的只讀來源持有生命週期，再驗證最終版本G06。'
if done:
    note=note.replace(f'當前{c["status"]}。','已實際終止0與逐映像／來源／記錄程序退出核對，僅相關範圍通過。')
elif c['status']=='failed_preserved':note+='本輪失敗須保留原始記錄後定位，未算通過。'
files=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json','README.md','docs/HISTORY.md']
out=app/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
before={n:sha(ROOT/n) for n in files}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
for n in files[:4]:
    v=read(ROOT/n)
    if 'retained_identity_before_shared_archive_owner' not in v:v['retained_identity_before_shared_archive_owner']=v.get('current_product_identity')
    for key,val in v.items():
        if isinstance(val,dict) and (key.startswith('current_') and key!='current_shared_archive_owner_application'):
            val['applies_to_current_shared_archive_source']=False
    for key in list(v):
        if key.startswith('source_freeze_for_') or key in ['source_freeze_while_Cargo']:v[key]=False
    v.update(current_shared_archive_owner_application=record,
        current_product_identity=dict(source_manifest_sha256=c['source_manifest_sha256'],source_files=1024,compiled_input_files=1025,
            source_qualification_pending=not done,Debug_normal_sha256=profiles.get('Debug',{}).get('normal',{}).get('sha256') if profiles.get('Debug',{}).get('normal') else None,
            Release_normal_sha256=profiles.get('Release',{}).get('normal',{}).get('sha256') if profiles.get('Release',{}).get('normal') else None,
            Debug_normal_qualified_for_current_source=profiles.get('Debug',{}).get('status')=='independently_qualified_related_and_normal',
            Release_normal_qualified_for_current_source=profiles.get('Release',{}).get('status')=='independently_qualified_related_and_normal',GUI_verified=False,whole_goal_complete=False),
        current_product_version_scope='source'+c['source_manifest_sha256'][:8]+'_shared_archive_owner_related_'+('qualified_other_scopes_pending' if done else 'pending'),
        current_working_tree_verified=False,working_tree_modified_since_current_normal_qualification=not done,
        source_freeze_for_shared_archive_owner_validation=active,source_freeze_while_Cargo=active and c['status'] in ['compiling_shared_archive_owner_related_targets','building_unique_entry_normal'],
        active_workload_sessions={'shared_archive_owner_related':a.session_id} if active else {},next_task=note,
        last_goal_turn_classification='progress',last_goal_turn_evidence=[str(app/'application.json'),str(run/'cohort.json')],
        progress_updated_utc=datetime.now(timezone.utc).isoformat(),goal_complete=False,whole_goal_complete=False)
    for item in v.get('items',[]):
        if item['id'] in ['G02-real-peer-battle','G03-current-regression','G05-gameplay-matrix','G06-room-service-branches','G08-player-gui','G09-audio','G10-release-package','G11-stability-performance']:
            item.update(state='source'+c['source_manifest_sha256'][:8]+'_related_validation_'+('qualified_remaining_scope_pending' if done else 'pending_previous_versions_retained'),complete=False)
    save(ROOT/n,v)
for n in files[4:]:
    p=ROOT/n;head,_,body=p.read_text(encoding='utf-8-sig').partition('\n');p.write_text(head+'\n\n2026-10-17：'+note+'\n'+body,encoding='utf-8',newline='\n')
save(out/'record.json',dict(before=before,before_archive_sha256=sha(out/'before.zip'),after={n:sha(ROOT/n) for n in files},scope=record,whole_goal_complete=False))
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(app.parent/'current-stage.json',dict(run=str(run),original_session_id=a.session_id,source_manifest_sha256=c['source_manifest_sha256'],tracker=str(out)))
print(json.dumps(dict(checkpoint=str(out),status=c['status'],source_manifest_sha256=c['source_manifest_sha256'],both_related_profiles_qualified=done),ensure_ascii=False))
