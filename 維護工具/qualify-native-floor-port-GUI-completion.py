"""Independently read back current1005 GUI after its original tool handle exits."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
ap=argparse.ArgumentParser();ap.add_argument('normal',type=Path);ap.add_argument('--session-id',type=int,required=True);args=ap.parse_args()
NORMAL=args.normal.resolve(strict=True);assert NORMAL.is_relative_to(APP)
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
g=read(NORMAL/'GUI-current.json');process=read(NORMAL/'GUI-process-completion-observation.json')
assert g['status']=='qualified_current_lighting_TW14_GUI_cases' and not g['source_and_corresponding_tests_frozen']
assert process['session_id']==args.session_id and process['terminal_exit_code']==0 and process['actual_terminal_tool_observed']
assert process['original_pipeline_identity_absent'] and process['owned_workloads']==[]
assert process['pipeline_pid']==g['pipeline_pid']
assert process['expected_creation_utc']==g['pipeline_creation_utc']
cohort=Path(g['cohort']).resolve(strict=True);assert cohort.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-current')
q=read(cohort/'qualified.json');n=read(NORMAL/'qualified.json');s=read(NORMAL/'scoped-qualified.json');pins=read(NORMAL/'source-inputs.json')
assert n['status']=='qualified_unique_entry_floor_port_normal_builds'
assert s['status']=='qualified_floor_port_Debug120_Release120_and_normal_mains'
assert q['status']=='qualified_current_lighting_normal_mains14_GUI_teleport_and_recovery_cases'
assert q['source_files']==q['compiled_input_files']==len(pins)==1005
assert q['source_manifest_sha256']==n['source_manifest_sha256']==sha(NORMAL/'source-inputs.json')
for profile in ['Debug','Release']:
    assert s['counts'][profile]==dict(passed=120,failed=0,ignored=6,actual_test_executables=19)
assert q['resource_profile']=='tw13081901-v1' and q['extended_item_details'] and q['scenarios']==14
count_keys=['actual_formal_sessions','LOGIN_sessions','GAME_sessions','exact_C_MOVE','extra_C_MOVE','original_world_PNGs','original_glyph_matches','stale_GUI_notifications']
assert tuple(q[k] for k in count_keys)==(112,56,56,220,0,80,56,128)
assert q['reference_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
expected=['cross-endpoint','timeout','refused','wrong-service','cancel-retry','cancel-late','cancel-expiry']
for profile in ['Debug','Release']:
    cases=q['profiles'][profile]['cases'];assert [c['scenario'] for c in cases]==expected
    main=n['profiles'][profile];assert sha(Path(main['path']))==main['sha256']==s['normal_mains'][profile]
    assert q['profiles'][profile]['main_sha256']==main['sha256']
    for row in cases:
        folder=Path(row['case']);local=read(folder/'current-lighting-source-qualification.json')
        assert sha(folder/'current-lighting-source-qualification.json')==row['qualification_sha256']
        assert local['source_files']==1005 and local['main_sha256']==main['sha256'] and local['source_inputs_unchanged']
for name,h in pins.items():assert sha(ROOT/name)==h,name
archive=cohort/'GUI-raw-evidence.zip';assert sha(archive)==q['raw_archive_sha256'] and q['raw_archive_readback_verified']
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist())==set(q['raw_archive_files'])
    for name,h in q['raw_archive_files'].items():assert hashlib.sha256(z.read(name)).hexdigest().upper()==h,name
for key in ['physical_input_verified','multi_DPI_verified','same_endpoint_verified','full_default_regression_verified','full_library_regression_verified','full_current_TCP_regression_verified']:assert not q[key],key
observation={'status':'independently_readback_verified_current1005_TW14_GUI_and_terminal_process',
 'observed_utc':datetime.now(timezone.utc).isoformat(),'session_id':args.session_id,'terminal_exit_code':0,'source_files':1005,
 'cohort':str(cohort),'qualification_sha256':sha(cohort/'qualified.json'),'raw_files':len(q['raw_archive_files']),
 'raw_archive_sha256':sha(archive),'source_manifest_sha256':sha(NORMAL/'source-inputs.json'),'normal_mains':s['normal_mains'],
 'counts':{k:q[k] for k in ['scenarios']+count_keys},'scope':'Both current normal mains, tw13081901-v1/extended, private desktop widget messages at192DPI only',
 'physical_input_verified':False,'multi_DPI_verified':False,'full_framebuffer_oracle_verified':False,
 'full_default_library_TCP_regression_verified':False,'water_pixels_in_live_GUI_verified':False,'map303_specific_GUI_verified':False,'whole_goal_complete':False}
save(NORMAL/'GUI-completion-observation.json',observation)
assert read(NORMAL/'GUI-completion-observation.json')==observation
print(json.dumps(observation,ensure_ascii=False))
