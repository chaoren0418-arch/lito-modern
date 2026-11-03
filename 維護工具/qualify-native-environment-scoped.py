"""Independently bind environment tests, original vectors and normal x64 mains."""
from pathlib import Path
import argparse, csv, hashlib, json, re, struct, zipfile

ROOT=Path(__file__).resolve().parents[1]
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);args=ap.parse_args()
run=args.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-environment')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def check_archive(v):
    p=Path(v['path']);assert sha(p)==v['sha256'] and v['readback_verified']
    with zipfile.ZipFile(p) as z:
        assert set(z.namelist())==set(v['file_pins'])
        for name,h in v['file_pins'].items():
            with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h,name
def pe64(b):
    at=struct.unpack_from('<I',b,60)[0]
    assert b[at:at+4]==b'PE\0\0' and struct.unpack_from('<H',b,at+4)[0]==0x8664
    assert struct.unpack_from('<H',b,at+24)[0]==0x20b

assert not (run/'independent-qualified.json').exists()
q=read(run/'qualified.json');receipt=read(run/'receipt.json')
assert q['status']==receipt['status']=='completed_GAME_environment_related_scoped_checks_and_unique_entry_mains'
assert not receipt['active_pid'] and receipt['phase'] is None and receipt['features']==[]
inputs=read(run/'source-inputs.json');fixtures=read(run/'fixture-inputs.json')
assert len(inputs)==q['source_files']==978 and len(fixtures)==4
assert q['source_manifest_sha256']==sha(run/'source-inputs.json')
for name,h in {**inputs,**fixtures}.items():assert sha(ROOT/name)==h,name
for item in [q['source_archive'],q['raw_archive'],q['normal_archive']]:check_archive(item)

expected={'native_environment':8,'native_game_timers':3,'native_network':9,'native_login':3,
          'native_game':4,'native_world':5,'native_light_spell':7,'native_teleport':8,'native_map_portal':1}
counts={}
assert set(q['profiles'])==set(q['normal_mains'])=={'Debug','Release'}
for profile,checks in q['profiles'].items():
    assert set(checks['tests'])==set(expected)
    check_archive(checks['products'])
    messages=[json.loads(line) for line in (run/(profile+'-compile.stdout')).read_text('utf-8-sig').splitlines() if line.startswith('{')]
    assert any(m.get('reason')=='build-finished' and m['success'] for m in messages)
    assert receipt[profile+'-compile_exit_code']==0
    totals={'passed':0,'failed':0,'ignored':0}
    with zipfile.ZipFile(checks['products']['path']) as z:
        for name,result in checks['tests'].items():
            command=read(run/(profile+'-'+name+'-command.json'))['args']
            assert command[1:]==['--test-threads=1','--nocapture']
            image=Path(command[0]);assert sha(image)==result['exe_sha256']
            retained=z.read(name+'.exe');pe64(retained)
            assert hashlib.sha256(retained).hexdigest().upper()==result['exe_sha256']
            rows=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',
                            (run/(profile+'-'+name+'.stdout')).read_text('utf-8-sig'))
            expected_counts=(expected[name],0,2 if name=='native_game_timers' else 1 if name=='native_map_portal' else 0,0,0)
            assert len(rows)==1 and tuple(map(int,rows[0]))==expected_counts,(profile,name,rows)
            assert receipt[profile+'-'+name+'_exit_code']==0
            for key in totals:totals[key]+=result[key]
    assert totals=={'passed':48,'failed':0,'ignored':3};counts[profile]=totals
    main=q['normal_mains'][profile]
    assert main['features']==[] and main['unique_entry'] and not main['GUI_revalidated']
    assert sha(Path(main['path']))==main['sha256'];pe64(Path(main['path']).read_bytes())
    assert receipt[profile+'-entry_exit_code']==0
    command=read(run/(profile+'-entry-command.json'))['args']
    assert command[3]==str(ROOT/'啟動現代客戶端.ps1') and command[-3:]==['-BuildOnly','-BuildProfile',profile]
    assert sha(run/(profile+'-entry-build.log'))==main['entry_build_log_sha256']
    assert 'Finished `' in (run/(profile+'-entry-build.log')).read_text('utf-8-sig')

# Compare the compact compiler fixtures to the retained original/reference data.
fixture_dir=ROOT/'LiTo381-Modern/tests/fixtures'
assert (fixture_dir/'native-environment-map-flag.bin').read_bytes()==(ROOT/'驗證紀錄/current/native-lighting-map-policy/map16-flags.bin').read_bytes()
assert (fixture_dir/'native-environment-map-level.bin').read_bytes()==(ROOT/'驗證紀錄/current/native-world-light-policy/native-map-levels.bin').read_bytes()
calendar=bytearray()
with (ROOT/'驗證紀錄/current/native-reference-calendar/calendar.csv').open(encoding='utf-8',newline='') as stream:
    for row in csv.DictReader(stream):
        v={k:int(value) for k,value in row.items()}
        calendar.extend(struct.pack('<IHH6B',v['raw'],v['year'],v['yday0'],v['month1'],v['day'],v['hour'],v['minute'],v['second'],v['offset']//3600))
assert len(calendar)==223729*14 and bytes(calendar)==(fixture_dir/'native-environment-calendar.bin').read_bytes()
daylight=bytearray()
for line in (ROOT/'驗證紀錄/current/native-lighting-source/boundary-vectors.jsonl').read_text('utf-8').splitlines():
    v=json.loads(line);daylight.extend(struct.pack('<HHBBHH',v['day_of_year_zero_based'],v['minute'],v['flags'],v['level'],v['sunrise'],v['sunset']))
assert len(daylight)==43920*10 and bytes(daylight)==(fixture_dir/'native-environment-daylight.bin').read_bytes()
proof={'status':'independently_qualified_GAME_environment_and_nine_related_targets',
       'run':str(run),'source_files':978,'source_manifest_sha256':q['source_manifest_sha256'],
       'fixture_inputs':fixtures,'fixture_derivation_rechecked':True,'counts':counts,'test_products_per_profile':9,
       'normal_mains':{p:v['sha256'] for p,v in q['normal_mains'].items()},
       'calendar_reference_vectors_per_profile':223729,'daylight_original_boundaries_per_profile':43920,
       'map_flag_original_vectors_per_profile':65536,'map_argument_original_vectors_per_profile':262144,
       'run_qualification_sha256':sha(run/'qualified.json'),'source_archive_sha256':q['source_archive']['sha256'],
       'raw_archive_sha256':q['raw_archive']['sha256'],'normal_archive_sha256':q['normal_archive']['sha256'],
       'all_archives_readback_verified':True,'actual_formal_sessions':0,'GUI_verified':False,
       'full_default_regression_verified':False,'full_library_regression_verified':False,
       'full_current_TCP_regression_verified':False,'night_world_rendering_integrated':False,
       'whole_goal_complete':False,'qualifier_sha256':sha(Path(__file__))}
(run/'independent-qualified.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=run/'normal';normal.mkdir()
(normal/'source-inputs.json').write_bytes((run/'source-inputs.json').read_bytes())
n={'status':'qualified_default_normal_GAME_environment_mains','source_files':978,
   'source_manifest_sha256':q['source_manifest_sha256'],'profiles':q['normal_mains'],'features':[],
   'archive':q['normal_archive'],'independent_scoped_qualification':str(run/'independent-qualified.json'),
   'night_world_rendering_integrated':False,'GUI_revalidated':False,'whole_goal_complete':False}
(normal/'qualified.json').write_text(json.dumps(n,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:proof[k] for k in ['status','counts','normal_mains','source_manifest_sha256']},ensure_ascii=False))
