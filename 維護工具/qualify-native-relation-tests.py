"""Independently read retained sources, EXEs, and actual harness logs."""
from pathlib import Path
import hashlib,json,re,sys,zipfile
ROOT=Path(__file__).resolve().parents[1]
run=Path(sys.argv[1]).resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-lighting-candidate')
expected=int(sys.argv[2]); require_current='--require-current' in sys.argv[3:]
labels='--relation-labels-only' in sys.argv[3:]
water='--water-producer' in sys.argv[3:]
port='--water-port' in sys.argv[3:]
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest().upper()
receipt=read(run/'receipt.json'); pins=read(run/'compiled-inputs.json'); products=read(run/'products.json')
assert receipt['status']=='actual_applied_lighting_and_related_targets_passed'
assert receipt['owned_process'] is None and receipt['compile_exit_code']==0
assert receipt['compiled_inputs']==len(pins)==(1002 if port else 999) and receipt['source_inputs_unchanged']
assert set(products)==set(receipt['suites']) and len(products)==(5 if labels else 13 if port else 12)
if labels:assert set(products)=={'native_world_metadata','native_label_updates','native_world_labels','native_clan_markers','native_emblems'}
with zipfile.ZipFile(run/'compiled-source.zip') as z:
    assert set(z.namelist())==set(pins)
    for name,value in pins.items():
        assert sha(z.read(name))==value,name
        if require_current:assert sha((ROOT/name).read_bytes())==value,name
total=0; logs={}
with zipfile.ZipFile(run/'test-products.zip') as z:
    assert set(z.namelist())=={name+'.exe' for name in products}
    for name,p in products.items():
        data=z.read(name+'.exe'); assert sha(data)==p['sha256']
        pe=int.from_bytes(data[60:64],'little')
        assert data[:2]==b'MZ' and data[pe:pe+4]==b'PE\0\0'
        assert data[pe+4:pe+6]==b'\x64\x86' and data[pe+24:pe+26]==b'\x0b\x02'
        log=(run/name/'stdout.log').read_bytes()
        found=re.findall(rb'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)
        assert len(found)==1 and found[0][0]==b'ok'
        values=list(map(int,found[0][1:])); assert values[1:]==[0,0,0,0]
        recorded=receipt['results'][name]
        assert recorded==dict(exit_code=0,passed=values[0],failed=0,ignored=0)
        total+=values[0]
        logs[name]=dict(passed=values[0],stdout_sha256=sha(log),stderr_sha256=sha((run/name/'stderr.log').read_bytes()),exe_sha256=p['sha256'])
assert total==receipt['passed']==expected and receipt['failed']==receipt['ignored']==0
record={'status':'independently_readback_verified_'+receipt['profile']+str(total)+('_relation_labels_scope' if labels else '_water_port_scope' if port else '_water_producer_scope' if water else '_actor_relation_scope'),
 'profile':receipt['profile'],'passed':total,'failed':0,'ignored':0,'actual_test_executables':len(products),
 'source_inputs':len(pins),'current_worktree_checked':require_current,
 'source_manifest_sha256':sha((run/'compiled-inputs.json').read_bytes()),
 'source_zip_sha256':sha((run/'compiled-source.zip').read_bytes()),
 'test_products_zip_sha256':sha((run/'test-products.zip').read_bytes()),'results':logs,
 'normal_main_qualified':False,'current_GUI_verified':False,'actual_game_sessions':0,
 'scope':('Existing master/label/clan-marker/emblem regressions' if labels else 'Pure Rust native water clock/queue/scroll vectors, GAME and scene retirement, actual Model RGBA555 water pixels, dry shared camera accumulation and asynchronous history transfer' if port else 'Actual World/Scene water source classification from shared body action, master and event flags; exact700 range lookup stop plus earlier lighting/core regressions' if water else 'MS950 raw packet relations, stateful S36 flags and prior lighting/water core tests')+('; latest normal main/GUI/full regression remain separate' if port else '; no original S36 handler execution, no scene water shader/timer integration'),
 'whole_goal_complete':False}
(run/'qualified.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
assert read(run/'qualified.json')==record
print(json.dumps({'run':str(run),'status':record['status'],'passed':total}))
