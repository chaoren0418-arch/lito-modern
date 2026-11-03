"""Publish only the ten executed resource bodies and retain remaining gates."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser()
ap.add_argument('qualification',type=Path)
a=ap.parse_args()
qp=a.qualification.resolve(strict=True)
assert qp.is_relative_to(ROOT/'驗證紀錄/current/native-resource-optin')
q=read(qp)
assert q['status']=='independently_readback_qualified_current_ten_unique_original_resource_bodies_both_profiles'
assert q['source_files']==1028 and q['compiled_input_files']==1029
assert q['unique_previously_ignored_bodies']==10 and q['actual_test_invocations']==20
assert q['unchanged_default_library_ignored_count']==473
assert q['other_library_ignored_bodies_not_qualified_by_these_scopes']==463
assert not q['GUI_verified'] and not q['audio_device_verified'] and not q['whole_goal_complete']
assert not q['sixth_scene_test_qualified']
compiled=read(qp.parent/'compiled-inputs.json')
assert len(compiled)==1029 and all(sha(ROOT/n)==h for n,h in compiled.items())
for info in q['normal_main_identities'].values():assert sha(info['path'])==info['sha256']
assert sha(q['normal_related_qualification'])==q['normal_related_qualification_sha256']
assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
with zipfile.ZipFile(qp.parent/'raw.zip') as z:
    assert set(z.namelist())==set(q['raw_files'])
    for n,h in q['raw_files'].items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json',
    '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json',
    'README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
for n in names[:5]:
    j=read(ROOT/n)
    assert j['current_product_identity']['source_manifest_sha256']==q['source_manifest_sha256']
    assert j['current_selector_full_library']['qualified'] and j['current_selector_map563']['qualified']
    assert set(j.get('active_workloads',{}))=={'selector-resource-optin-scene'}
    assert j['active_workloads']['selector-resource-optin-scene']['original_session_id']==76378
record=dict(qualification=str(qp),qualification_sha256=sha(qp),source_manifest_sha256=q['source_manifest_sha256'],
    unique_bodies=10,actual_test_invocations=20,actual_test_executables=2,
    profiles=q['profiles'],scopes=q['scopes'],recorded_test_processes=20,
    default_library_ignored_count_unchanged=473,other_ignored_bodies_unqualified=463,
    sixth_scene_test_qualified=False,normal_main_identities=q['normal_main_identities'],
    qualified=True,actual_formal_sessions=0,GUI_verified=False,audio_device_verified=False,
    all_resources_verified=False,whole_goal_complete=False)
out=qp.parent/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==v for n,v in before.items())
work=('來源89D4B630兩版廣域504／500過、完整library各1937過0敗473原忽略、map563兩版120 LOGIN/GAME sessions均已獨立核對。'
    '同版十項原資源本文共二十次執行及兩份測試PE已通過並獨立核對；原忽略473不變，其餘463本文未因本批取得資格。'
    '下一步同版正常可見性兩版八案；輸入綁定已準備但尚未執行。技能／服務／同端點／GUI／實體輸入音效／可攜G11G12與完整G01–G12仍待。'
    '登入器來源唯讀、產物獨立且替換提案撤回；原時限／預算不變。')
for n in names[:5]:
    p=ROOT/n;j=read(p)
    j.update(current_selector_resource_optins=record,active_workloads={},active_workload_sessions={},
        source_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,
        last_goal_turn_classification='progress',last_goal_turn_evidence=record,next_task=work,next_focus=work,
        goal_complete=False,whole_goal_complete=False,progress_updated_utc=datetime.now(timezone.utc).isoformat())
    j['next_visibility_inputs']=str(ROOT/'驗證紀錄/current/native-detection-invisibility/current-normal-inputs-20261017-155415-779587')
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='source89D4-normal-visibility-pending',work=work,applies_to_current_source=True,goal_complete=False)
    for item in j.get('items') or []:
        if item['id'].startswith('G03'):item.update(state='source89D4_broad_library_map_and_ten_resource_scopes_qualified_visibility_pending',complete=False,current_resource_evidence=record)
    for key,gate in (j.get('current_source_goal_validation') or {}).items():
        if key.startswith('G03'):gate.update(state='source89D4_broad_library_map_and_ten_resource_scopes_qualified_visibility_pending',complete=False)
    save(p,j)
for n in names[5:]:
    p=ROOT/n;first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n')
    assert first.startswith('2026-10-17') and sep
    p.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json',dict(record=record,before_files=before,before_archive_sha256=sha(out/'before.zip'),after_files={n:sha(ROOT/n) for n in names}))
print(json.dumps(dict(checkpoint=str(out),unique_bodies=10,actual_invocations=20,whole_goal_complete=False),ensure_ascii=False))
