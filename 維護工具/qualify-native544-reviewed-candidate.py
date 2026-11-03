"""Read back exact source delta, original tests, control failure and both pixels."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native544-reviewed-candidate')
r=read(run/'receipt.json');assert r['status']=='executed_isolated_baseline_red_candidate_green_exact_marker' and r['owned_process'] is None
assert len(r['process_history'])==9
assert {j['label'] for j in r['process_history'] if j['exit_code']==101}=={'baseline-samples'}
assert all(j['exit_code']==j['expected_exit_code'] for j in r['process_history'])
pins=read(run/'compiled-inputs.json');assert len(pins)==1029
for n,h in pins.items():assert sha(ROOT/n)==h,n
p=r['preparation'];assert sha(p['source'])==p['source_sha256']==sha(run/'baseline-sprite.rs')
assert sha(p['candidate'])==p['candidate_sha256']==sha(run/'candidate-sprite.rs')
before=(run/'baseline-sprite.rs').read_text(encoding='utf-8-sig');after=(run/'candidate-sprite.rs').read_text(encoding='utf-8-sig')
marker='\n                    | (544, 384, -480, -384, u16::MAX, 0)';assert after.count(marker)==1
start=after.index('\n    #[test]\n    fn reviewed544_marker_preserves_slots_and_rejects_corruption')
end=after.rfind('\n}');assert start<end
assert (after[:start]+after[end:]).replace(marker,'')==before
new_test='modern_lito::sprite::tests::reviewed544_marker_preserves_slots_and_rejects_corruption'
body=(run/'probe_body.rs').read_text(encoding='utf-8-sig');assert sha(run/'probe_body.rs')==r['body_sha256']
def inventory(label):
    text=(run/label/'stdout.log').read_text(encoding='utf-8-sig')
    return set(re.findall(r'^(.+): test$',text,re.M))
base=inventory('baseline-inventory');candidate=inventory('candidate-inventory')
assert len(base)==19 and candidate==base|{new_test} and len(candidate)==20
assert inventory('baseline-ignored-inventory')==inventory('candidate-ignored-inventory')
ignored=inventory('candidate-ignored-inventory');assert len(ignored)==2
def rows(label):
    text=(run/label/'stdout.log').read_text(encoding='utf-8-sig')
    return text,dict(re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',text,re.M))
text,units=rows('candidate-unit');assert set(units)==candidate
assert {n for n,s in units.items() if s=='ignored'}==ignored
assert all(s=='ok' for n,s in units.items() if n not in ignored)
assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)==[('ok','18','0','2','0','0')]
for label,result,filtered in [('baseline','FAILED','18'),('candidate','ok','19')]:
    text,test=rows(label+'-samples');assert test=={'exact_owned_samples_match_independent_csharp':result if result=='FAILED' else 'ok'}
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)==[(result,'0' if result=='FAILED' else '1','1' if result=='FAILED' else '0','0','0',filtered)]
base_result=read(run/'baseline-comparison.json');actual=read(run/'candidate-comparison.json')
assert base_result['accepted_files']==0 and len(base_result['errors'])==3 and all('reversed declared frame bounds' in x for x in base_result['errors'])
assert actual==dict(accepted_files=3,compact_pixel_matches=31,empty_frames=32,errors=[],flat_pixel_matches=31,frames=63)
for name,info in r['products'].items():
    assert sha(info['path'])==info['sha256'] and sha(info['wrapper'])==info['wrapper_sha256']
    wrapper=Path(info['wrapper']).read_text(encoding='utf-8-sig');assert wrapper==body.replace('__SPRITE_PATH__',(run/(name+'-sprite.rs')).as_posix())
    with Path(info['path']).open('rb') as f:
        head=f.read(64);assert head[:2]==b'MZ';f.seek(int.from_bytes(head[60:64],'little'));pe=f.read(26)
        assert pe[:4]==b'PE\0\0' and pe[4:6]==b'\x64\x86' and pe[24:26]==b'\x0b\x02'
assert sha(r['independent_reference'])==r['independent_reference_sha256'] and sha(r['report'])==r['report_sha256']
q=read(r['independent_reference']);assert q['reviewed_null_slots']==32 and q['nonempty_reference_frames']==31
normal=read(ROOT/'工作佇列.json')['current_selector_resource_optins']['normal_main_identities']
for info in normal.values():assert sha(info['path'])==info['sha256']
ids=','.join(str(j['pid']) for j in r['process_history'])
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for old in r['process_history']:
        if row['ProcessId']==old['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(old['creation_utc'])).total_seconds())>.001
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'CIM.json').write_bytes(raw)
files={p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file() and not any(part.startswith('qualified-') for part in p.relative_to(run).parts)}
files['CIM.json']=out/'CIM.json';files['qualifier.py']=Path(__file__);files['independent-reference.json']=Path(r['independent_reference'])
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist())==set(hashes)
    for n,h in hashes.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
q=dict(status='independently_qualified_isolated_exact544_marker_candidate',source_manifest_sha256=r['source_manifest_sha256'],baseline_source_sha256=r['preparation']['source_sha256'],
    candidate_source=r['preparation']['candidate'],candidate_source_sha256=r['preparation']['candidate_sha256'],normal_mains=normal,
    baseline_control_actual_exit_code=101,candidate_unit_passed=18,candidate_unit_failed=0,unchanged_ignored=2,
    baseline_registered_tests_preserved=19,original_sprite_registered_tests_preserved=18,original_sprite_nonignored_tests_passed=17,new_synthetic_nonignored_tests_passed=1,
    explicit_source_pinned_test_passed=1,new_nonignored_test=new_test,actual_samples=3,actual_frames=63,normalized_empty_frames=32,flat_pixel_matches=31,compact_pixel_matches=31,
    recorded_owned_child_identities=9,recorded_owned_children_all_terminal=True,reference=r['independent_reference'],reference_sha256=r['independent_reference_sha256'],
    parser_runtime_delta='one exact zero-reference marker',production_source_modified=False,normal_product_built=False,new_sessions=0,new_casts=0,native_render_bank_qualified=False,GUI_verified=False,whole_goal_complete=False,
    raw_files=hashes,raw_archive_sha256=sha(out/'raw.zip'))
save(out/'qualified.json',q)
print(json.dumps(dict(qualification=str(out/'qualified.json'),unit_passed=18,frames=63,flat_pixel_matches=31,compact_pixel_matches=31,production_source_modified=False),ensure_ascii=False))
