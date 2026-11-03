"""Read back the entire current999 GUI cohort after its original handle exited."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'actual-tests-20261016-071612-845832/normal-qualified'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
g=read(NORMAL/'GUI-current.json');process=read(NORMAL/'GUI-process-completion-observation.json')
assert g['status']=='qualified_current_lighting_TW14_GUI_cases' and not g['source_and_corresponding_tests_frozen']
assert process['session_id']==61939 and process['terminal_exit_code']==0 and process['actual_terminal_tool_observed']
assert process['original_pipeline_identity_absent'] and process['owned_workloads']==[]
assert process['pipeline_pid']==g['pipeline_pid']==93440
assert process['expected_creation_utc']==g['pipeline_creation_utc']=='2026-10-16T07:29:48.1686560Z'
cohort=Path(g['cohort']).resolve(strict=True);assert cohort.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-current')
q=read(cohort/'qualified.json');n=read(NORMAL/'qualified.json');s=read(NORMAL/'scoped-qualified.json');pins=read(NORMAL/'source-inputs.json')
assert q['status']=='qualified_current_lighting_normal_mains14_GUI_teleport_and_recovery_cases'
assert q['source_files']==q['compiled_input_files']==len(pins)==999
assert q['source_manifest_sha256']==n['source_manifest_sha256']==sha(NORMAL/'source-inputs.json')
assert s['counts']['Debug']==dict(passed=81,failed=0,ignored=0,actual_test_executables=17)
assert s['counts']['Release']==dict(passed=65,failed=0,ignored=0,actual_test_executables=12)
assert q['resource_profile']=='tw13081901-v1' and q['extended_item_details'] and q['scenarios']==14
assert (q['actual_formal_sessions'],q['LOGIN_sessions'],q['GAME_sessions'],q['exact_C_MOVE'],q['extra_C_MOVE'],
        q['original_world_PNGs'],q['original_glyph_matches'],q['stale_GUI_notifications'])==(112,56,56,220,0,80,56,128)
assert q['reference_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
expected=['cross-endpoint','timeout','refused','wrong-service','cancel-retry','cancel-late','cancel-expiry']
for profile in ['Debug','Release']:
    cases=q['profiles'][profile]['cases'];assert [c['scenario'] for c in cases]==expected
    main=n['profiles'][profile];assert sha(Path(main['path']))==main['sha256']==s['normal_mains'][profile]
    assert q['profiles'][profile]['main_sha256']==main['sha256']
    for row in cases:
        folder=Path(row['case']);local=read(folder/'current-lighting-source-qualification.json')
        assert sha(folder/'current-lighting-source-qualification.json')==row['qualification_sha256']
        assert local['source_files']==999 and local['main_sha256']==main['sha256'] and local['source_inputs_unchanged']
for name,h in pins.items():assert sha(ROOT/name)==h,name
archive=cohort/'GUI-raw-evidence.zip';assert sha(archive)==q['raw_archive_sha256'] and q['raw_archive_readback_verified']
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist())==set(q['raw_archive_files'])
    for name,h in q['raw_archive_files'].items():assert hashlib.sha256(z.read(name)).hexdigest().upper()==h,name
assert not q['physical_input_verified'] and not q['multi_DPI_verified'] and not q['same_endpoint_verified']
assert not q['full_default_regression_verified'] and not q['full_library_regression_verified'] and not q['full_current_TCP_regression_verified']
observation={'status':'independently_readback_verified_current999_TW14_GUI_and_terminal_process',
 'observed_utc':datetime.now(timezone.utc).isoformat(),'session_id':61939,'terminal_exit_code':0,'source_files':999,
 'cohort':str(cohort),'qualification_sha256':sha(cohort/'qualified.json'),'raw_files':len(q['raw_archive_files']),
 'raw_archive_sha256':sha(archive),'source_manifest_sha256':sha(NORMAL/'source-inputs.json'),
 'normal_mains':s['normal_mains'],'counts':{k:q[k] for k in ['scenarios','actual_formal_sessions','exact_C_MOVE','extra_C_MOVE','original_world_PNGs','original_glyph_matches','stale_GUI_notifications']},
 'scope':'Both current normal mains, tw13081901-v1/extended, private desktop widget messages at192DPI only',
 'physical_input_verified':False,'multi_DPI_verified':False,'full_framebuffer_oracle_verified':False,
 'full_default_library_TCP_regression_verified':False,'water_extra_effect_complete':False,'whole_goal_complete':False}
save(NORMAL/'GUI-completion-observation.json',observation)
for name in ['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json']:
    p=ROOT/name;v=read(p);v.update(source_freeze_for_active_GUI=False,current_GUI_all14_complete=True)
    v['current_product'].update(current_GUI_verified=True,current_GUI_evidence=str(cohort/'qualified.json'),current_GUI_scope=observation['scope'])
    v['current_normal_lighting_product']=v['current_product']
    v['native_water_source_current_work'].update(status='normal999_Debug81_Release65_TW14_GUI_verified',source_frozen_for_GUI=False,
        current_GUI_verified=True,GUI_terminal_exit_code=0,GUI_completion_observation=str(NORMAL/'GUI-completion-observation.json'))
    v['working_tree_native_lighting_application'].update(GUI='latest999_TW14_verified',GUI_terminal_exit_code=0,GUI_source_freeze_released=True,
        GUI_evidence=str(cohort/'qualified.json'),GUI_raw_files=len(q['raw_archive_files']),GUI_raw_archive_sha256=q['raw_archive_sha256'])
    v['next_task']='同handle61939實際exit0，93440原creation身份及owned GUI/Java/compiler/test全退已核對。當前999正常A09486F2/6E2FEDF2、Debug81/Release65與TW+extended14全部限定資格、完整raw ZIP獨立讀回；112sessions/220move零多送/80PNG/56glyph/128stale。來源可改。先執行已準備的8原時鐘/排程函式隔離oracle（water-clock-prepared-current.json），核對8656phase/9668scroll/15queue scenarios與code relocations；原4D63A0會清掉water再重排50ms，482370確保一筆10000ms aux回呼，不能沿用mapload永遠疊加舊推論。其後把真實phase/scroll與額外Rust水下像素pass接Scene/GAME owner，再當版Debug/Release與GUI重驗。完整default/lib/TCP/實體輸入/可攜/穩定性與G01–G12仍待，登入器/相鄰伺服器唯讀。'
    save(p,v)
e=read(APP/'water-source-current.json');e.update(current_GUI_verified=True,source_frozen_for_GUI=False,GUI_terminal_exit_code=0,
    GUI_evidence=str(cohort/'qualified.json'),GUI_completion_observation=str(NORMAL/'GUI-completion-observation.json'))
save(APP/'water-source-current.json',e)
for name in ['README.md','docs/ARCHITECTURE.zh-TW.md']:
    p=ROOT/name;text=p.read_text(encoding='utf-8-sig');text=text.replace('最新GUI尚未驗，額外water shader/phase/scroll仍待','同版TW14 GUI已限定核對112sessions/220move零多送/80PNG/56glyph/128stale，僅192DPI私有桌面訊息；額外water shader/phase/scroll仍待',1);p.write_text(text,encoding='utf-8')
with (ROOT/'docs/HISTORY.md').open('a',encoding='utf-8') as f:
    f.write('\n\n999正常A09486F2/6E2FEDF2的TW14 GUI已完成：同session61939實際exit0，原pwsh93440 creation07:29:48.1686560Z身份與owned GUI/Java/compiler/test全退已核對；兩版合112sessions、220move零多送、80原世界PNG、56glyph、128stale，全背包與wire details逐案成立。完整raw '+str(len(q['raw_archive_files']))+'檔ZIP '+q['raw_archive_sha256']+'再次獨立讀回，來源與正常兩主不變；解除來源凍結。僅192DPI私有桌面訊息限定，完整引擎/水下額外pass/default/lib/TCP等未完成。\n')
print(json.dumps({'status':observation['status'],'raw_files':observation['raw_files'],'counts':observation['counts']}))
