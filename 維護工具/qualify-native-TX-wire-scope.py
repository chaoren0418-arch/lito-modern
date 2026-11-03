"""Retain and independently read back all corrected encrypted fault cases."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--profile',choices=['Debug','Release'],required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-wire')
d=run/a.profile;state=read(run/'cohort.json');r=state['profiles'][a.profile];product=read(d/'product-retention.json')
assert r['malformed-target']['passed']==1 and r['all-six-wire-groups']['passed']==6 and r['all-six-wire-groups']['failed']==0
for name,expected in [('malformed-target',1),('all-six-wire-groups',6)]:
    text=(d/name/'stdout.log').read_text(encoding='utf-8-sig')
    matches=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)
    assert len(matches)==1 and matches[0][:5]==('ok',str(expected),'0','0','0')
assert sha(Path(product['product']))==product['product_sha256']==r['product_sha256']
with zipfile.ZipFile(product['archive']) as z:
    with z.open('lito_modern.exe') as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==product['product_sha256']
pins=read(run/'source-inputs.json')
for name,h in pins.items():assert sha(ROOT/name)==h,name
fixture_path=ROOT/'LiTo381-Modern/tests/fixtures/native-teleport-controller-current.json';fixture=read(fixture_path)
count=len(bytes.fromhex(fixture['vectors']['availability']['hex']))
malformed={f'status-truncated-{i}' for i in range(count)}|{f'endpoint-truncated-{i}' for i in range(8)}|{'status-trailing','status-unknown-word','status-duplicate','endpoint-before-status','endpoint-wrong-slot','endpoint-invalid-index','endpoint-zero-port','endpoint-trailing'}
expected=malformed|{'retired-duplicate-burst','coalesced-old-socket-burst','new-epoch-replayed-endpoint','cancel-old-burst-fresh-login','frame-zero','frame-one','frame-partial'}
assert len(expected)==37 and len(malformed)==30
out=d/'wire-scope-qualified';out.mkdir();(out/'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
files={'independent-qualifier.py':out/'independent-qualifier.py'};connections=0
for name in sorted(expected):
    source=ROOT/'LiTo381-Modern/target/current/verification/native-teleport-wire'/a.profile/(name+'.json')
    raw=source.read_bytes();row=json.loads(raw.decode('utf-8-sig'))
    assert row['case']==name and row['quiescent'] and row['synthetic_encrypted_loopback'] and row['authenticated_formal_sessions']==0 and row['seeded_scope_from_C02']
    assert row['fixture_sha256']==sha(fixture_path);connections+=sum(row['peer_connection_counts'])
    if name in malformed:
        assert row['phase']=='failed' and row['epoch']==1 and row['peer_connection_counts']==[1] and row['client_command_opcodes']==[[0x27]]
        extra=row['extra'];assert extra['retired_TX_queue_absent'] and extra['exact_original_request_only'] and extra['quit_requests']==0 and not extra['new_epoch_started']
        assert isinstance(extra['expected_written_TX_counter'],int) and extra['expected_written_TX_counter']==extra['final_written_TX_counter']
        snap=row['snapshot'];assert snap['world']['actors']=={} and snap['inventory']['owner'] is None and snap['inventory']['items']=={} and snap['merchant']['owner'] is None and snap['joined_room'] is None and snap['selected_character'] is None and snap['services'] is None
    dest=out/(name+'.json');dest.write_bytes(raw);assert sha(dest)==sha(source);files['cases/'+dest.name]=dest
assert connections==41
for folder in ['malformed-target','all-six-wire-groups']:
    for file in (d/folder).iterdir():
        if file.is_file():files[folder+'/'+file.name]=file
for name in ['product-retention.json','receipt.json']:files[name]=d/name
for name in ['normal-invariance.json','source-inputs.json','source-delta.zip']:files[name]=run/name
archive=out/'raw.zip';hashes={name:sha(f) for name,f in files.items()}
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED) as z:
    for name,f in files.items():z.write(f,name)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist())==set(hashes)
    for name,h in hashes.items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
q=dict(status='independently_readback_verified_corrected_six_wire_groups37_cases41_synthetic_connections',profile=a.profile,passed_groups=6,failed_groups=0,ignored_groups=0,
    malformed_cases=30,scenarios=37,synthetic_TCP_connections=41,actual_formal_sessions=0,source_files=1021,source_manifest_sha256=sha(run/'source-inputs.json'),
    actual_test_sha256=product['product_sha256'],actual_test_archive_sha256=product['archive_sha256'],raw_archive_sha256=sha(archive),raw_files=hashes,
    normal_EXEs_unchanged_proof=str(run/'normal-invariance.json'),full_default_library_qualified=False,GUI_verified=False,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(status=q['status'],profile=a.profile,scenarios=37,synthetic_TCP=41,formal_sessions=0),ensure_ascii=False))
