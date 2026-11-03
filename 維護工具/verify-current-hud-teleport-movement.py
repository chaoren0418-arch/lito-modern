"""Match every actual movement packet to the intended own-window GUI step."""
from pathlib import Path
import argparse,json,hashlib,shutil
root=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('case',type=Path);a=ap.parse_args();case=a.case.resolve(strict=True);assert case.is_relative_to(root/'驗證紀錄/current/native-teleport-gui')
r=read(case/'receipt.json');q=read(case/'HUD-source-qualification.json');assert r['status'].startswith('verified_native_teleport_GUI_') and r['inputs_unchanged'] and r['reference_is_current'];assert q['receipt_sha256']==sha(case/'receipt.json') and q['all_published_inputs_and_current_sources_match']
steps=[s for s in r['steps'] if s['step']=='GUI-key-walk'];assert len(steps)==(14 if r['scenario']=='cross-endpoint' else 16)
heading={0x68:0,0x26:1,0x66:2,0x27:3,0x62:4,0x28:5,0x64:6,0x25:7}
expected=[bytes([0x0a])+int(s['before'][0]).to_bytes(2,'little')+int(s['before'][1]).to_bytes(2,'little')+bytes([heading[s['key']]]) for s in steps]
cap=next((Path(r['runtime'])/'captures').glob('*/frames.jsonl'));frames=[json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()];actual=[bytes.fromhex(x['plain']) for x in frames if x['direction']=='client_to_server' and x['plain'].startswith('0A ')];assert actual==expected,{'actual':[b.hex() for b in actual],'expected':[b.hex() for b in expected]}
out={'status':'verified_one_C_MOVE_per_recorded_GUI_step_exact_origin_and_heading','scenario':r['scenario'],'main_sha256':r['exe_sha256'],'receipt_sha256':sha(case/'receipt.json'),'capture_sha256':sha(cap),'recorded_steps':len(steps),'actual_C_MOVE':len(actual),'extra_C_MOVE':0,'validator_sha256':sha(Path(__file__)),'full_game_complete':False};(case/'movement-packet-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');dest=case/'validation-sources'/(sha(Path(__file__))[:16]+'-'+Path(__file__).name);dest.parent.mkdir(exist_ok=True)
if not dest.exists():shutil.copy2(Path(__file__),dest)
else:assert sha(dest)==sha(Path(__file__))
print(json.dumps({k:out[k] for k in ['status','scenario','actual_C_MOVE','extra_C_MOVE']}))