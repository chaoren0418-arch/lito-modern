import json,sys,hashlib,struct
from pathlib import Path
p=Path(sys.argv[1]);r=json.loads(p.read_text(encoding='utf-8-sig'));assert r['action_profile'].startswith('rental-') and r['source_matches_snapshot']
frames=[]
for f in sorted((Path(r['runtime'])/'captures').rglob('frames.jsonl')):
    frames.extend(json.loads(s) for s in f.read_text(encoding='utf-8-sig').splitlines())
checks=[]
for role,c in r['clients'].items():
    name='TestKnight' if role=='a' else 'TestMage';w=c['end']['world'];s=w['status'];assert all(v>=1 for v in w['attribute_updates'])
    for opcode in ['0x50','0x51','0x52','0x56']:
        rows=[f for f in frames if f['direction']=='server_to_client' and f['opcode']==opcode and ('player='+name) in f['note']]
        assert rows,(role,opcode,'missing reference update');b=bytes.fromhex(rows[-1]['plain'])
        if opcode=='0x50':
            assert len(b)==8
            assert [s[k] for k in ['strength','intelligence','wisdom','dexterity','constitution','charisma','weight_gauge']]==list(b[1:])
        elif opcode=='0x51':assert len(b)==6 and [s['level'],s['experience']]==[b[1],struct.unpack_from('<I',b,2)[0]]
        elif opcode=='0x52':assert len(b)==6 and [s[k] for k in ['armor_class','fire_resistance','water_resistance','wind_resistance','earth_resistance']]==list(struct.unpack('<5b',b[1:]))
        else:assert len(b)==3 and w['magic_modifiers']==list(struct.unpack('<2b',b[1:]))
        assert int(opcode,16) not in w['unhandled_opcodes'];checks.append({'role':role,'opcode':opcode,'formal_frame':rows[-1]})
out={'status':'verified_schema0_received_attributes','receipt_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'checks':checks,'extended_numeric_schema_verified':False,'gui_verified':False}
(p.parent/'attributes-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'status':out['status'],'checks':len(checks)}))
