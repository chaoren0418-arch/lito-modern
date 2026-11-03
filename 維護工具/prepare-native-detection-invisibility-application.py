"""Retain the qualified 6D baseline, then stage one existing-API red test."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
CANDIDATE=ROOT/'維護工具/candidates/native-detection-invisibility'
BASE=ROOT/'驗證紀錄/current/native-retained-geometry/validation-20261017-004225-579828'
LIB=ROOT/'驗證紀錄/current/native-startup-library-default/20261017-012714-666943'
PROBE=ROOT/'驗證紀錄/current/native-detection-invisibility-candidate/20261017-015322-825372'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def retain(path,files):
    hashes={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(path,'x',zipfile.ZIP_DEFLATED) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist())==len(hashes) and set(z.namelist())==set(hashes)
        for n,h in hashes.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    return dict(path=str(path),sha256=sha(path),files=hashes)
lib=read(LIB/'completion-observation.json');q=read(PROBE/'independent-readback.json')
assert lib['original_session_id']==80951 and lib['observed_terminal_exit_code']==0 and lib['recorded_owned_processes_all_terminal']
assert q['original_session_id']==98810 and q['observed_terminal_exit_code']==0 and q['skill_ids']==[13,60]
assert q['full_candidate_effect_loader_compiled'] is False and q['actual_casts']==0
pins=read(BASE/'source-inputs.json');inputs=read(BASE/'compiled-inputs.json')
assert len(pins)==1024 and len(inputs)==1025 and sha(BASE/'source-inputs.json')==lib['source_manifest_sha256']==q['source_manifest_sha256']
for n,h in inputs.items():assert sha(ROOT/n)==h,n
meta=read(CANDIDATE/'metadata.json')
files=['LiTo381-Modern/src/modern_lito/'+n for n in meta['files']]
files+=['LiTo381-Modern/tests/native_skills.rs','LiTo381-Modern/tests/oracle/NativePeerHarness.java']
assert len(files)==5 and len(set(files))==5
for n,info in meta['files'].items():
    assert sha(PROJECT/'src/modern_lito'/n)==info['base_sha256'] and sha(CANDIDATE/n)==info['candidate_sha256']
assert sha(PROJECT/'tests/oracle/NativePeerHarness.java')==meta['fixture_proposal']['base_sha256']
for n,h in q['candidate_files'].items():assert sha(CANDIDATE/n)==h
out=ROOT/'驗證紀錄/current/native-detection-invisibility'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir(parents=True);(out/'prepare-tool.py').write_bytes(Path(__file__).read_bytes())
before=retain(out/'before.zip',{n:ROOT/n for n in files})
candidate=retain(out/'proposal.zip',{n:CANDIDATE/n for n in q['candidate_files']})
(out/'base-source-inputs.json').write_bytes((BASE/'source-inputs.json').read_bytes())
(out/'base-compiled-inputs.json').write_bytes((BASE/'compiled-inputs.json').read_bytes())
state=dict(status='baseline_retained_before_detection_invisibility_application',output=str(out),before=before,proposal=candidate,
    baseline_source_manifest_sha256=sha(BASE/'source-inputs.json'),baseline_compiled_source_archive=read(BASE/'cohort.json')['compiled_source_archive'],
    baseline_library_completion=str(LIB/'completion-observation.json'),baseline_library_completion_sha256=sha(LIB/'completion-observation.json'),
    isolated_probe_qualification=str(PROBE/'independent-readback.json'),isolated_probe_qualification_sha256=sha(PROBE/'independent-readback.json'),
    normal_mains_before={p:dict(path=str(PROJECT/'target/current/x86_64-pc-windows-msvc'/p.lower()/'LiToModern.exe'),sha256=h) for p,h in lib['normal_mains'].items()},
    candidate_applied_to_product=False,actual_new_casts=0,GUI_verified=False,whole_goal_complete=False)
save(out/'application.json',state)
fragment=(CANDIDATE/'test-additions.rs').read_text(encoding='utf-8-sig')
start=fragment.index('#[test]');end=fragment.index('#[test]',start+1)
red=fragment[start:end].strip()+'\n'
test=PROJECT/'tests/native_skills.rs';original=test.read_bytes()
assert b'fn detection_selector_menu_guards_and_cooldown_leave_authority_unchanged(' not in original
test.write_bytes(original+b'\n'+red.encode('utf-8'))
redpins={n:sha(ROOT/n) for n in inputs};assert {n for n in inputs if inputs[n]!=redpins[n]}=={'LiTo381-Modern/tests/native_skills.rs'}
save(out/'red-compiled-inputs.json',redpins)
(out/'red-native_skills.rs').write_bytes(test.read_bytes())
state.update(status='red_detection_request_test_staged_only',red_test_sha256=sha(test),
    red_test_name='detection_selector_menu_guards_and_cooldown_leave_authority_unchanged',
    red_expected_reason='此技能的請求格式尚未接入')
save(out/'application.json',state)
print(json.dumps(dict(application=str(out),status=state['status'],product_runtime_sources_changed=False,test_source_staged=True),ensure_ascii=False))
