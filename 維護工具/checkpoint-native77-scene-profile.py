"""Record a completed profile while the overall owned cohort stays pending."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('qualification',type=Path);ap.add_argument('--reader-tool-chunk',required=True);a=ap.parse_args()
qp=a.qualification.resolve(strict=True);assert qp.is_relative_to(ROOT/'驗證紀錄/current/native77-scene-integration');q=read(qp);p=q['profile']
assert q['status']=='independently_qualified_current_native77_normal_related_profile' and q['explicit_actual_asset_passed']==10 and q['recorded_owned_all_terminal']
assert sha(q['normal']['path'])==q['normal']['sha256'];run=Path(q['run']);inputs=read(run/'compiled-inputs.json')
for n,h in inputs.items():assert sha(ROOT/n)==h
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json',
 '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
for n in names[:5]:
    j=read(ROOT/n);assert j['current_product_identity']['source_manifest_sha256']==q['source_manifest_sha256'] and j['active_workloads']['native77-scene-normal']['original_session_id']==38394
out=qp.parent/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(qualification=str(qp),qualification_sha256=sha(qp),profile=p,source_manifest_sha256=q['source_manifest_sha256'],normal_sha256=q['normal']['sha256'],
    related_library_passed=q['passed_related_bodies'],related_library_ignored=q['ignored_related_bodies'],explicit_actual_asset_passed=10,external_integration_passed=q['external_integration_passed'],external_integration_ignored=q['external_integration_ignored'],
    reader_exit_code=0,reader_terminal_tool_chunk=a.reader_tool_chunk,recorded_jobs=len(q['recorded_jobs']),recorded_owned_all_terminal=True,
    qualified=True,qualification_scope=q['qualification_scope'],new_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
other='Release' if p=='Debug' else 'Debug'
work='來源'+q['source_manifest_sha256'][:8]+'之'+p+'正常主'+q['normal']['sha256'][:8]+'由唯一入口建置／版本0並獨立讀回；當版library相關'+str(q['passed_related_bodies'])+'過、10實體資源本文過、12外部相關映像共'+str(q['external_integration_passed'])+'過／'+str(q['external_integration_ignored'])+'原忽略，48個記錄job皆終止0。僅此範圍取得資格，原38394之'+other+'與整體程序退役仍待最終讀回，來源／QA凍結。究極光裂術場景／延後hurt與有限音效已接入；C14／ack／冷卻與實際C02施放、其他技能、最終版完整G06／G01–G12、GUI實體音效／G11G12未完成。登入器唯讀且產物獨立，原時限／預算不變。'
for n in names[:5]:
    path=ROOT/n;j=read(path);j.setdefault('current_native77_scene_normal_profiles',{})[p]=record;identity=j['current_product_identity']
    identity.update({p+'_normal_sha256':q['normal']['sha256'],p+'_normal_source_manifest_sha256':q['source_manifest_sha256'],p+'_normal_qualified_for_current_source':True,p+'_normal_state':'independently_qualified_native77_related_only'})
    identity['qualification_scope']='native77_related_'+p+'_qualified_cohort_pending'
    j.update(next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=record,goal_complete=False,whole_goal_complete=False)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='native77-current-normal-related-validation',work=work,applies_to_current_source=True,goal_complete=False)
    save(path,j)
for n in names[5:]:
    path=ROOT/n;first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    path.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
for n in names[:5]:assert read(ROOT/n)['current_native77_scene_normal_profiles'][p]==record
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(profile=record,before_files=before,after_files={n:sha(ROOT/n) for n in names}))
print(json.dumps(dict(checkpoint=str(out),profile=p,normal_sha256=q['normal']['sha256'],whole_goal_complete=False),ensure_ascii=False))
