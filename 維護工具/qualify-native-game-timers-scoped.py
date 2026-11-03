"""Independently read back exact mask products, native vectors and test logs."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import zipfile

ROOT=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-game-timers')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def check_archive(v):
    p=Path(v['path']);assert sha(p)==v['sha256'] and v['readback_verified']
    with zipfile.ZipFile(p) as z:
        assert set(z.namelist())==set(v['file_pins'])
        for n,h in v['file_pins'].items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
def pe64(raw):
    at=struct.unpack_from('<I',raw,60)[0]
    assert raw[at:at+4]==b'PE\0\0' and struct.unpack_from('<H',raw,at+4)[0]==0x8664 and struct.unpack_from('<H',raw,at+24)[0]==0x20b

q=read(run/'qualified.json');r=read(run/'receipt.json')
assert q['status']=='completed_GAME_timers_related_scoped_checks_and_unique_entry_mains'
entry_dir=run
recovery=None
if q.get('entry_recovery'):
    assert r['status']=='failed' and r['phase']=='Debug-entry' and 'UnicodeDecodeError' in r['error']
    assert r['Debug-entry_exit_code']==0 and not r['active_pid']
    recovered_path=Path(q['entry_recovery']);assert recovered_path.resolve().is_relative_to(run)
    assert sha(recovered_path)==q['entry_recovery_sha256']
    recovery=read(recovered_path)
    assert recovery['status']=='completed_unique_entry_mains_after_observation_encoding_recovery'
    assert recovery['original_failed_receipt_sha256']==sha(run/'receipt.json')
    assert not recovery['actual_tests_rerun'] and not recovery['active_pid'] and not recovery['active_profile']
    check_archive(recovery['raw_archive'])
    entry_dir=recovered_path.parent
else:
    assert r['status']==q['status'] and not r['active_pid'] and not r['phase']
assert r['features']==[]
assert not q['night_world_rendering_integrated'] and not q['GUI_verified'] and not q['whole_goal_complete']
assert q['source_files']==976 and q['source_manifest_sha256']==sha(run/'source-inputs.json')
inputs=read(run/'source-inputs.json');assert len(inputs)==976
for name,h in inputs.items():assert sha(ROOT/name)==h,name
check_archive(q['source_archive']);check_archive(q['raw_archive']);check_archive(q['normal_archive'])
assert set(q['profiles'])==set(q['normal_mains'])=={'Debug','Release'}
counts={}
for profile,checks in q['profiles'].items():
    check_archive(checks['products'])
    messages=[json.loads(l) for l in (run/(profile+'-compile.stdout')).read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
    assert any(m.get('reason')=='build-finished' and m['success'] for m in messages)
    assert r[profile+'-compile_exit_code']==0
    assert (recovery['profiles'][profile]['entry_exit_code'] if recovery else r[profile+'-entry_exit_code'])==0
    counts[profile]={'passed':0,'failed':0,'ignored':0}
    with zipfile.ZipFile(checks['products']['path']) as z:
        for name,c in checks['tests'].items():
            command=read(run/(profile+'-'+name+'-command.json'))
            image=Path(command['args'][0]);assert sha(image)==c['exe_sha256']
            pe64(z.read(name+'.exe'))
            assert hashlib.sha256(z.read(name+'.exe')).hexdigest().upper()==c['exe_sha256']
            text=(run/(profile+'-'+name+'.stdout')).read_text(encoding='utf-8-sig')
            rows=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
            assert len(rows)==1 and tuple(map(int,rows[0]))==({'native_game_timers':3,'native_network':9,'native_login':3,'native_game':4,'native_light_mask':5,'native_light_spell':7,'native_teleport':8}[name],0,2 if name=='native_game_timers' else 0)
            assert r[profile+'-'+name+'_exit_code']==0
            for key in counts[profile]:counts[profile][key]+=c[key]
    assert counts[profile]=={'passed':39,'failed':0,'ignored':2}
    main=q['normal_mains'][profile]
    assert main['unique_entry'] and main['features']==[] and not main['GUI_revalidated']
    assert sha(Path(main['path']))==main['sha256']
    pe64(Path(main['path']).read_bytes())
    assert sha(entry_dir/(profile+'-entry-build.log'))==main['entry_build_log_sha256']
    assert 'Finished `' in (entry_dir/(profile+'-entry-build.log')).read_text(encoding='utf-8-sig')
    command=read(entry_dir/(profile+'-entry-command.json'))
    if recovery:
        assert command['entry']==str(ROOT/'啟動現代客戶端.ps1') and command['BuildOnly'] and command['BuildProfile']==profile
        assert command['forced_console_encoding']=='utf-8'
    else:
        assert command['args'][-3:]==['-BuildOnly','-BuildProfile',profile]
        assert command['args'][3]==str(ROOT/'啟動現代客戶端.ps1')

native_dir=ROOT/'驗證紀錄/current/native-lighting-mask-compositor'
native=read(native_dir/'qualified.json')
assert native['cases']==330 and native['compositor_calls']==810 and native['clear_calls']==634
assert native['independent_python_mask_bytes_checked']==21_542_400 and native['archive_readback_verified']
assert sha(native_dir/'mask-compositor-evidence.zip')==native['archive_sha256']
vectors=ROOT/'LiTo381-Modern/tests/fixtures/native-light-mask-original.json'
assert sha(vectors)==native['vectors_sha256']
v=read(vectors);assert len(v['cases'])==330
actual=(native_dir/'actual-masks.bin').read_bytes()
for i,c in enumerate(v['cases']):
    assert hashlib.sha256(actual[i*65280:(i+1)*65280]).hexdigest().upper()==c['original_native_sha256']
assert sha(ROOT/'LiTo381-Modern/tests/fixtures/native-light-source.lmp')==native['source_lmp_sha256']
result={'status':'independently_qualified_current_x64_GAME_timers_and_related_network_selection_teleport_scoped_checks',
    'run':str(run),'source_files':976,'source_manifest_sha256':q['source_manifest_sha256'],
    'counts':counts,'test_products_per_profile':7,'normal_mains':{p:v['sha256'] for p,v in q['normal_mains'].items()},
    'original_native_mask_cases_per_profile':330,'original_native_mask_output_bytes':21_542_400,
    'source_lmp_sha256':native['source_lmp_sha256'],'native_oracle_qualification_sha256':sha(native_dir/'qualified.json'),
    'run_qualification_sha256':sha(run/'qualified.json'),'raw_archive_sha256':q['raw_archive']['sha256'],
    'source_archive_sha256':q['source_archive']['sha256'],'normal_archive_sha256':q['normal_archive']['sha256'],
    'all_archives_readback_verified':True,'entry_observation_encoding_recovery_verified':bool(recovery),
    'actual_tests_rerun_by_recovery':False,'game_timer_callbacks_integrated':True,'natural_intervals_verified_by_this_qualifier':False,'scene_light_source_selection_integrated':False,
    'night_world_rendering_integrated':False,'GUI_verified':False,'new_game_sessions':0,
    'full_default_regression_verified':False,'full_library_regression_verified':False,
    'full_current_TCP_regression_verified':False,'whole_goal_complete':False,'qualifier_sha256':sha(Path(__file__))}
with (run/'independent-qualified.json').open('x',encoding='utf-8') as f:json.dump(result,f,ensure_ascii=False,indent=2);f.write('\n')
print(json.dumps(result,ensure_ascii=False))
