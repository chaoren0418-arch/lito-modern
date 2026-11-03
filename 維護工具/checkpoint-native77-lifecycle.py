"""Record isolated native77 scope without promoting it to normal product/gameplay."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
run=ROOT/'驗證紀錄/current/native77-lifecycle/20261017-182454-076969'
qp=run/'qualified-20261017-182535-346532/qualified.json';q=read(qp)
assert sha(qp)=='0C2957E76740CDCAFA024EC7BCE1DA4696850DA6AA9D33726ED8E491B8C7531B'
assert q['status']=='independently_qualified_isolated_native77_lifecycle' and not q['production_source_modified']
assert q['recorded_owned_all_terminal'] and q['unit_passed']==5 and q['explicit_actual_asset_lifecycle_passed']==7
assert sha(q['product']['path'])==q['product']['sha256']
with Path(q['product']['path']).open('rb') as f:
    assert f.read(2)==b'MZ';f.seek(0x3c);offset=struct.unpack('<I',f.read(4))[0];f.seek(offset);assert f.read(4)==b'PE\0\0';machine=struct.unpack('<H',f.read(2))[0];assert machine==0x8664
for name,h in q['core_pins'].items():assert sha(name)==h
normal=Path(q['normal_qualification']);inputs=read(normal.parent/'compiled-inputs.json');assert len(inputs)==1029
for name,h in inputs.items():assert sha(ROOT/name)==h
cohort=read(normal.parent/'cohort.json')
for info in cohort['profiles'].values():assert sha(info['normal']['path'])==info['normal']['sha256']
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json',
 '驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
for name in names[:5]:
    j=read(ROOT/name);assert j['current_product_identity']['source_manifest_sha256']==q['source_manifest_sha256'] and not j['active_workloads']
out=qp.parent/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in names:z.write(ROOT/name,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(qualification=str(qp),qualification_sha256=sha(qp),source_manifest_sha256=q['source_manifest_sha256'],core_pins=q['core_pins'],
    original_session_id=90592,original_exit_code=0,original_terminal_tool_chunk='582eaa',independent_reader_exit_code=0,independent_reader_terminal_tool_chunk='1c31d1',
    actual_test_PE_machine=machine,unit_passed=5,unit_ignored=8,explicit_actual_asset_lifecycle_passed=7,explicit_actual_asset_lifecycle_ignored=0,
    recorded_owned_processes=5,recorded_owned_all_terminal=True,qualified=True,qualification_scope='isolated_current_TW_native77_core_only',
    applies_to_current_candidate_source=True,retained_candidate_version_only=False,production_source_modified=False,sound_payload_delivery_verified=False,native_scene_interleaving_verified=False,new_sessions=0,new_casts=0,GUI_verified=False,whole_goal_complete=False)
work='正常來源78A49D5D及兩主6A30C45C／A184FDB6不變，限定解碼器資格維持。究極光裂術原生隔離核心已5非忽略測試＋7實體TW素材生命週期本文通過並獨立讀回，5記錄程序皆終止；涵蓋同步caster!啟動戳、內部發布身分、取消／來源失敗後同身分重試拒絕舊事件、一般／暴擊有限回呼、固定封包落點、目標生命版本、來源／世代退役、64上限與延遲不追趕、Screen555後Multiply555畫布接線。候選尚未套產品；既有World立即hurt路徑、NativeScene前後景與有限音效payload去重尚待接入，再接C14權威／ack／冷卻與實際C02施放。無新sessions或施法資格；34／38／45／46、legacy及42缺漏仍待。完整G06／同端點、G01–G12與GUI實體輸入音效／可攜G11G12未完成，登入器唯讀且產物獨立，原時限／預算不變。'
prior=dict(initial_driver_prechild_failure=dict(terminal_tool_chunk='0e59bc',exit_code=1,cause='candidate copy loop shadowed cohort variable; no child started'),
    fixture_failure=dict(run=str(run.parent/'20261017-182013-540819'),terminal_tool_chunk='577329',exit_code=1,cause='synthetic world lacked MAP/START; packets were not admitted; fixture now asserts both startup and acceptance',runtime_core_repair=False),
    earlier_scoped_pass=dict(run=str(run.parent/'20261017-182038-142573'),terminal_tool_chunk='944aa0',exit_code=0,retained_source_version_only=True),
    first_reader_failure=dict(terminal_tool_chunk='15cb9b',exit_code=1,cause='ZIP reader used Windows backslashes instead of ZIP slash names; reader-only repair; runtime not rerun'),
    earlier_scoped_qualification=dict(run=str(run.parent/'20261017-182123-216732'),terminal_tool_chunk='66e8cd',exit_code=0,retained_candidate_version_only=True,qualification_sha256='A8E14EDCDFA20CCDD34C0008E243FC89BE1B8A70AC402E07C11E05F70E6D7796'),
    publication_fixture_path_failure=dict(run=str(run.parent/'20261017-182424-161829'),terminal_tool_chunk='667322',exit_code=1,cause='Python generated Windows paths contained escaped control bytes; test paths repaired to forward slashes; native core unchanged'))
save(out/'prior-runs.json',prior)
for name in names[:5]:
    path=ROOT/name;j=read(path);j.update(current_native77_lifecycle_candidate=record,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=record,
        goal_complete=False,whole_goal_complete=False,progress_updated_utc=datetime.now(timezone.utc).isoformat())
    if name==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='native77-scene-audio-authority-integration',work=work,applies_to_current_source=False,goal_complete=False)
    save(path,j)
for name in names[5:]:
    path=ROOT/name;first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    path.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
for name in names[:5]:
    j=read(ROOT/name);assert j['current_native77_lifecycle_candidate']==record and j['next_task']==work and not j['active_workloads'] and not j['whole_goal_complete']
for name in names[5:]:assert work in (ROOT/name).read_text(encoding='utf-8').split('\n\n')[0]
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(scope=record,before_files=before,before_archive_sha256=sha(out/'before.zip'),after_files={n:sha(ROOT/n) for n in names},prior_runs=str(out/'prior-runs.json')))
print(json.dumps(dict(checkpoint=str(out),qualification_sha256=sha(qp),production_source_modified=False,whole_goal_complete=False),ensure_ascii=False))
