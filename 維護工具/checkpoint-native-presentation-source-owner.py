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
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-presentation-source-owner')
c=read(run/'cohort.json');pins=read(run/'compiled-inputs.json')
assert len(pins)==1025 and all(sha(ROOT/n)==h for n,h in pins.items())
assert sha(run/'source-inputs.json')==c['source_manifest_sha256']
app=Path(c['application']);application=read(app/'application.json')
assert application['source_manifest_sha256']==c['source_manifest_sha256']
done=(run/'completion-observation.json').exists();completion=read(run/'completion-observation.json') if done else None
active=c['status'] not in ['failed_preserved','executed_presentation_source_owner_related_tests_and_unique_normals']
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
note=(f'來源{c["source_manifest_sha256"][:8]}已接正常Presentation之來源持有生命週期與資源目錄隔離；切場景仍退役Frame／Model／舊輸入，'
    '僅保留根目錄綁定之只讀封存檔來源，明確取消禁止晚到工作重新持有。'
    f'原{a.session_id}之Debug／Release各10相關實際測試映像及唯一入口正常建置，當前{c["status"]}。'
    '原F73兩版各13映像84／80過0敗3既有忽略與32記錄子程序終止維持原版資格；新來源不移記。'
    '原66470已在F73確認不同目錄仍沿用Model之紅測試，Rust101／探針wrapper0、raw479DB8B2已封存，修後同斷言須重驗。'
    '八秒移動／六十秒等待／G11預算未變；最終版本G06、廣域／library／map563、正常GUI／實體輸入／音效／可攜交付與完整G01–G12仍待。'
    '登入器及伺服器來源只讀，產物與重建版獨立。')
if application.get('context_contract_version',1)>=2:
    regression=app/'independent-regression-readback.json'
    record['retained_parent_Release_failure']=dict(path=str(regression),sha256=sha(regression),failed_source_manifest_sha256=application['base_source_manifest_sha256'])
    note=(f'來源{c["source_manifest_sha256"][:8]}補繪圖工作的獨立發布身分：場景／來源變更及取消會換新身分，晚到成功／錯誤皆不可重新發布；來源路徑也另核對。'
        '同世界的環境更新仍保留既有模型／一次性音效接續，同GAME且同來源才接續原選擇器歷史。'
        f'原{a.session_id}之Debug／Release各10實際相關映像及唯一入口正常建置，當前{c["status"]}。'
        '原30212之00C0來源Debug57過0敗2忽略；Release照明映像9過1敗，抓到換來源後舊工作重發畫面，wrapper1／Rust101與全來源／映像已封存6C724608；該批未通過，不移記新來源。'
        '新加同場景立即恢復案例，原目錄隔離斷言不變；八秒／六十秒與G11預算未改。'
        '最終版本G06、廣域／library／map563、正常GUI／實體輸入／音效／可攜交付與完整G01–G12仍待，登入器／伺服器來源只讀且產物獨立。')
if application.get('test_only_correction'):
    correction=application['test_only_correction']
    record['immediate_qualified_parent_source_manifest_sha256']=application['parent_qualified_source_manifest_sha256']
    record['test_only_expected_list_correction']=correction
    record['runtime_source_changed_since_parent']=False
    record['retained_broad_failure']=dict(path=str(app/'independent-holy-broad-readback.json'),sha256=sha(app/'independent-holy-broad-readback.json'))
    note=(f'來源{c["source_manifest_sha256"][:8]}僅將神聖武器整合測試之完整效果期待清單由11項補為含偵測術749之12項；'
        '神聖武器影格／sound804／變造拒絕等所有原斷言不變，Runtime來源未改。'
        '原C72D廣域81809已完整Debug111映像499過1敗21既有忽略，wrapper1／唯一神聖武器Rust101與228記錄身分退出及原始來源／映像封存3F557F88；Release未啟動。'
        '同庫候選三案3過0敗0忽略已獨立核對，未代替整批。'
        f'原{a.session_id}之兩版各10實際相關映像與唯一入口正常建置，當前{c["status"]}。'
        'C72D原G06兩版96sessions維持原來源資格，不移記新全來源版本。新版本G06／廣域／library／map563／正常雙玩家可見性仍待。'
        '8秒／60秒／G11預算不變；登入器及伺服器來源只讀且產物獨立，GUI／實體輸入／音效／可攜交付與完整G01–G12未完成。')
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
    if application.get('test_only_correction'):
        old_default=v.get('current_presentation_source_owner_default110')
        if old_default and old_default.get('source_manifest_sha256')==application['parent_qualified_source_manifest_sha256']:
            old_default.update(status='failed_preserved',qualified=False,source_and_corresponding_QA_frozen=False,
                failure_readback=application['test_only_correction']['original_broad_failure'],
                failure_archive_sha256=application['test_only_correction']['original_broad_failure_archive_sha256'])
    if 'retained_identity_before_presentation_source_owner' not in v:v['retained_identity_before_presentation_source_owner']=v.get('current_product_identity')
    for key,val in v.items():
        if isinstance(val,dict) and (key.startswith('current_') and key!='current_presentation_source_owner_application'):
            val['applies_to_current_shared_archive_source']=False
    for key in list(v):
        if key.startswith('source_freeze_for_') or key in ['source_freeze_while_Cargo']:v[key]=False
    v.update(current_presentation_source_owner_application=record,
        current_product_identity=dict(source_manifest_sha256=c['source_manifest_sha256'],source_files=1024,compiled_input_files=1025,
            source_qualification_pending=not done,Debug_normal_sha256=profiles.get('Debug',{}).get('normal',{}).get('sha256') if profiles.get('Debug',{}).get('normal') else None,
            Release_normal_sha256=profiles.get('Release',{}).get('normal',{}).get('sha256') if profiles.get('Release',{}).get('normal') else None,
            Debug_normal_qualified_for_current_source=profiles.get('Debug',{}).get('status')=='independently_qualified_related_and_normal',
            Release_normal_qualified_for_current_source=profiles.get('Release',{}).get('status')=='independently_qualified_related_and_normal',GUI_verified=False,whole_goal_complete=False),
        current_product_version_scope='source'+c['source_manifest_sha256'][:8]+'_presentation_source_owner_related_'+('qualified_other_scopes_pending' if done else 'pending'),
        current_working_tree_verified=False,working_tree_modified_since_current_normal_qualification=not done,
        source_freeze_for_presentation_source_owner_validation=active,source_freeze_while_Cargo=active and c['status'] in ['compiling_presentation_source_owner_related_targets','building_unique_entry_normal'],
        active_workload_sessions={'presentation_source_owner_related':a.session_id} if active else {},next_task=note,
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
