"""Apply the retained proposal only after the missing-request red is preserved."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('application',type=Path);a=ap.parse_args()
app=a.application.resolve(strict=True);assert app.is_relative_to(ROOT/'驗證紀錄/current/native-detection-invisibility')
state=read(app/'application.json');red=read(app/'red-independent-readback.json')
assert state['status']=='red_detection_request_test_staged_only'
assert red['test_exit_code']==101 and red['observed_wrapper_exit_code']==1 and red['runtime_sources_unchanged']
assert sha(app/'red-raw.zip')==red['raw_archive_sha256']
assert not (app/'apply-receipt.json').exists()
pins=read(app/'red-compiled-inputs.json')
for n,h in pins.items():assert sha(ROOT/n)==h,n
with zipfile.ZipFile(state['proposal']['path']) as z:
    assert hashlib.sha256(Path(state['proposal']['path']).read_bytes()).hexdigest().upper()==state['proposal']['sha256']
    assert set(z.namelist())==set(state['proposal']['files'])
    proposal={n:z.read(n) for n in z.namelist()}
    for n,b in proposal.items():assert hashlib.sha256(b).hexdigest().upper()==state['proposal']['files'][n]
meta=json.loads(proposal['metadata.json'].decode('utf-8-sig'))
for n,info in meta['files'].items():
    assert sha(PROJECT/'src/modern_lito'/n)==info['base_sha256']
    assert hashlib.sha256(proposal[n]).hexdigest().upper()==info['candidate_sha256']
with zipfile.ZipFile(state['before']['path']) as z:
    assert sha(state['before']['path'])==state['before']['sha256']
    base_test=z.read('LiTo381-Modern/tests/native_skills.rs')
    for n,h in state['before']['files'].items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
fragment=proposal['test-additions.rs']
def red_function(text):
    begin=text.index('fn detection_selector_menu_guards_and_cooldown_leave_authority_unchanged(')
    end=text.find('#[test]',begin)
    return text[begin:end if end!=-1 else len(text)].strip()
assert red_function((app/'red-native_skills.rs').read_text(encoding='utf-8-sig'))==red_function(fragment.decode('utf-8-sig'))
assert sha(PROJECT/'tests/oracle/NativePeerHarness.java')==meta['fixture_proposal']['base_sha256']
for n in meta['files']:(PROJECT/'src/modern_lito'/n).write_bytes(proposal[n])
(PROJECT/'tests/native_skills.rs').write_bytes(base_test+b'\n'+fragment)
(PROJECT/'tests/oracle/NativePeerHarness.java').write_bytes(proposal['NativePeerHarness.java'])
before=read(app/'base-source-inputs.json');source={n:sha(ROOT/n) for n in before}
changed=[n for n in source if source[n]!=before[n]]
assert len(source)==1024 and set(changed)==set(state['before']['files'])
compiled=source|{'LiTo381-Modern/windows/app.manifest':sha(PROJECT/'windows/app.manifest')}
assert len(compiled)==1025
save(app/'applied-source-inputs.json',source);save(app/'applied-compiled-inputs.json',compiled)
save(app/'apply-receipt.json',dict(status='applied_detection_invisibility_requests_menu_and_effect_pending_product_validation',
    changed_sources=changed,source_files=1024,compiled_input_files=1025,source_manifest_sha256=sha(app/'applied-source-inputs.json'),
    candidate_proposal_archive_sha256=state['proposal']['sha256'],red_test_product_sha256=red['test_product_sha256'],
    red_assertions_preserved=True,normal_product_validation_complete=False,actual_new_casts=0,GUI_verified=False,
    whole_goal_complete=False,applied_utc=datetime.now(timezone.utc).isoformat()))
(app/'apply-tool.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(application=str(app),changed_sources=changed,source_manifest_sha256=sha(app/'applied-source-inputs.json'),
    actual_casts=0,product_validation_pending=True),ensure_ascii=False))
