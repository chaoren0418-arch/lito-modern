"""Record current gaps and a qualified unapplied proposal without reducing gaps."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
matrix=ROOT/'驗證紀錄/current/native-current-skill-support/20261017-001620-989736/qualified.json'
candidate=ROOT/'驗證紀錄/current/native-detection-skill-candidate/20261017-002102-887104/independent-readback.json'
m,c=read(matrix),read(candidate)
assert (m['observed_unique_learned_ids'],m['request_recognized_learned_ids'],m['missing_unique_learned_requests'])==(74,21,53)
assert c['skill_id']==13 and not c['candidate_applied_to_product'] and c['actual_casts']==0
assert c['source_manifest_sha256']==m['source_manifest_sha256']
pipeline=ROOT/'驗證紀錄/current/native-default110/20261016-231456-842850'
cohort=read(pipeline/'cohort.json')
for name,value in cohort['validation_pins'].items():assert sha(ROOT/'維護工具'/name)==value
compiled=read(candidate.parent/'compiled-inputs.json')
for name,value in compiled.items():assert sha(ROOT/name)==value
files=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json','README.md','docs/HISTORY.md']
out=candidate.parent/('tracker-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
before={name:sha(ROOT/name) for name in files}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in files:z.write(ROOT/name,name)
with zipfile.ZipFile(out/'before.zip') as z:
    assert set(z.namelist())==set(files)
    for name,value in before.items():assert hashlib.sha256(z.read(name)).hexdigest().upper()==value
support=dict(status=m['status'],qualification=str(matrix),sha256=sha(matrix),source_manifest_sha256=m['source_manifest_sha256'],
    observed_unique_learned_ids=74,request_recognized_learned_ids=21,missing_unique_learned_requests=53,
    missing_ids=m['missing_ids'],missing_by_role=m['missing_by_role'],request_recognition_is_not_gameplay_acceptance=True,
    new_sessions_executed=0,new_casts_executed=0,all_formal_reachable_operations_audited=False,GUI_verified=False,whole_goal_complete=False)
detection=dict(status=c['status'],qualification=str(candidate),sha256=sha(candidate),skill_id=13,
    request_hex='140104',request_guard_rejections=6,source_frames=23,finite_effect_samples=24,sound108_entry_count=1,
    bank_RGBA_bytes=c['bank_RGBA_bytes'],candidate_applied_to_product=False,product_source_changed=False,
    actual_formal_sessions=0,actual_casts=0,invisible_target_reveal_or_cooldown_runtime_verified=False,
    generic_skill_effect_loader_and_draw_compiled=True,unrelated_private_TW_prepare_branch_omitted=True,
    full_candidate_effect_loader_compiled=False,independent_CSharp_pixel_oracle_executed=False,
    GUI_verified=False,audio_device_verified=False,whole_goal_complete=False)
note=('當版4C7DE771/C02的兩版map563捕獲與現行請求已獨立核對：兩角色共74種已學技能，21種有請求處理，53種仍缺；'
    '請求辨識不代表玩法完成，舊BBC/9719表不移記當版。偵測術13之三檔候選已隔離d9d682actualexit0，'
    '140104、選單、6拒絕、原749之23frame/24sample及sound108一次已獨立讀回；原SPR由既有classic decoder交叉比對，未跑獨立CSharp pixel oracle。'
    '候選未套產品、沒有新session或施法，53缺口不減；完整施法/隱形揭露/冷卻/GUI仍待。首次私有模組編譯失敗已封存；'
    '探針使用原樣tower/shared-catalog，無關TW prepare分支省略，未聲稱完整TW loader編譯。'
    '廣域Release沿原24147，來源/四QA固定；待實際終止與雙版讀回後，在無自有編譯負載下量測40次warm residence，再依結果修正效能與接入玩法。'
    'G11固定預算未變，完整G01–G12未完成。')
for name in files[:4]:
    p=ROOT/name;v=read(p)
    v.update(current_native_skill_support=support,current_native_detection_candidate=detection,
        next_task=note,last_goal_turn_classification='progress',last_goal_turn_evidence=[str(matrix),str(candidate),str(pipeline/'cohort.json')],
        progress_updated_utc=datetime.now(timezone.utc).isoformat(),current_working_tree_verified=False,goal_complete=False,whole_goal_complete=False)
    if 'items' in v:
        g=next(item for item in v['items'] if item['id']=='G05-gameplay-matrix')
        g['current_support_audit']=support;g['unapplied_detection_candidate']=detection
        if g.get('findings'):
            history=g.setdefault('historical_findings',[])
            for old in g['findings']:
                if old not in history:history.append(old)
        g['findings']=[note];g['complete']=False
        for check in g.get('checks',[]):
            if check['id']=='G05.2':check.update(current_request_support_audit=str(matrix),current_missing_unique_learned_requests=53,next_candidate_batch=[13],candidate_batch_applied=False)
    save(p,v)
for name in files[4:]:
    p=ROOT/name;header,_,body=p.read_text(encoding='utf-8-sig').partition('\n')
    p.write_text(header+'\n\n2026-10-17：'+note+'\n'+body,encoding='utf-8',newline='\n')
save(out/'record.json',dict(before=before,before_archive_sha256=sha(out/'before.zip'),after={name:sha(ROOT/name) for name in files},
    support=support,detection=detection,product_source_changed=False,whole_goal_complete=False))
(out/'checkpoint-tool.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(checkpoint=str(out),missing_requests_still=53,candidate_applied=False),ensure_ascii=False))
