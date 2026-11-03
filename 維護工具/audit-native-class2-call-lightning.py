"""Read-only primary PE/ASM binding for future native class2 skill34 work."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,hashlib,json,re,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
ORIGINAL=ROOT.parent/'lito登入器/舊版分析與驗收'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
pe_path=ORIGINAL/'runtime/LiTo.unpacked-oep-current.exe';asm_path=ORIGINAL/'ida-client-oep-v2/full-disassembly.asm'
pe=pe_path.read_bytes();asm=asm_path.read_bytes()
assert hashlib.sha256(pe).hexdigest().upper()=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert hashlib.sha256(asm).hexdigest().upper()=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
e=struct.unpack_from('<I',pe,60)[0];assert pe[e:e+4]==b'PE\0\0'
count=struct.unpack_from('<H',pe,e+6)[0];sections=e+24+struct.unpack_from('<H',pe,e+20)[0];base=struct.unpack_from('<I',pe,e+24+28)[0];assert base==0x400000
def raw(address,length):
    rva=address-base
    for index in range(count):
        size,start,physical,at=struct.unpack_from('<IIII',pe,sections+index*40+8)
        if start<=rva and rva+length<=start+min(size,physical):return pe[at+rva-start:at+rva-start+length]
    raise ValueError(hex(address))
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
ranges={'class2_dispatch':(0x4a5fe0,0x4a602d),'class2_ctor':(0x44eac0,0x44eb90),'class2_advance':(0x44ebd0,0x44ed80),'class2_timer_and_draw':(0x44ed80,0x44ede0)}
decoded={name:list(md.disasm(raw(lo,hi-lo),lo)) for name,(lo,hi) in ranges.items()}
instructions={ins.address:ins for rows in decoded.values() for ins in rows}
expected={0x44eb04:'33db',0x44eb13:'885e13',0x44eb66:'894668',0x44edbb:'894e2c',0x44edc1:'895630',0x44edc7:'894e3c',0x44edcd:'895640'}
for address,code in expected.items():assert instructions[address].bytes.hex()==code
assert any(ins.mnemonic=='call' and ins.op_str=='0x44eac0' for ins in decoded['class2_dispatch'])
assert any(ins.mnemonic=='call' and ins.op_str=='0x4e5b50' for ins in decoded['class2_advance'])
assert any(ins.mnemonic=='call' and ins.op_str=='0x4d65f0' for ins in decoded['class2_advance'])
assert any('0x648efc' in ins.op_str for ins in decoded['class2_advance'])
assert any(ins.mnemonic=='call' and ins.op_str=='0x4c3e90' for ins in decoded['class2_advance'])
contract=ROOT/'驗證紀錄/current/native-current-skill-contract-audit';cq=read(contract/'qualified.json');factories=contract/'current-learned-factories.json'
assert sha(factories)==cq['file_pins'][factories.name]
entry=next(x for x in read(factories)['rows'] if x['skill_id']==34);spell=next(x for x in entry['matches'] if x['factory']=='attackSpell')['fields']
assert (spell['effectGfx'],spell['mpCost'],spell['maximumRange'],spell['reuseDelayMillis'])==(11736,18,6,10)
asset=ROOT/'驗證紀錄/current/native-attack-five-resource-inspection/20261017-144953-005315/asset-inventory.json';aq=read(asset.parent/'qualified/qualified.json')
assert sha(asset)==aq['raw_files'][asset.name]
inventory=read(asset);tw=next(x for x in inventory['explicit_TW_namespace_inspections'] if x['namespace']=='launcher/TW13081901.pak')
assert sha(Path(tw['root'])/'TW13081901.pak')==tw['pack_sha256']
row=next(x for x in tw['rows'] if x['id']==11736)
assert row['directionality']=='Undirected' and row['image_count']==1 and row['frames_per_second']==24 and len(row['frames'])==6
assert any(a['key']==109 and a['body']=='2 -1' for a in row['attributes'])
assert row['frames'][1]['events']==['Sound(34)'] and row['frames'][2]['events']==['Hit']
native=ROOT/'LiTo381-Modern/src/modern_lito/native_disintegrate.rs';text=native.read_text(encoding='utf-8-sig')
assert 'matches!(graphics, 11746 | 11748)' in text and 'position: WorldCell {' in text and 'cue.projectile.to[0]' in text
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=ROOT/'驗證紀錄/current/native-class2-call-lightning'/stamp;out.mkdir(parents=True)
for name,(lo,hi) in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo))
    (out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex():<24} {i.mnemonic} {i.op_str}' for i in decoded[name])+'\n',encoding='utf-8')
facts=dict(status='primary_PE_class2_binding_only_no_renderer_or_cast_qualification',primary_PE=dict(path=str(pe_path),sha256=sha(pe_path)),primary_ASM=dict(path=str(asm_path),sha256=sha(asm_path)),native_disintegrate=dict(path=str(native),sha256=sha(native)),formal_skill34=spell,formal_jar_sha256=cq['jar_sha256'],retained_TW_recipe_qualification=str(asset.parent/'qualified/qualified.json'),TW_pack_sha256=tw['pack_sha256'],TW_recipe=row,byte_predicates={hex(k):v for k,v in expected.items()},observed=dict(dispatch_constructor='44EAC0',node_frame13_initial=0,target_handle_field='68h',draw_resolves_target_each_time='44EDAF calls4E8160; copies actor2C/30 into effect2C/30 and3C/40 before base draw',advance='44EBD0 calls4E5B50 and schedules44ED80 via4D65F0',shared_state='648EFC counter, saved648EF8 and4C3E90 override/restore path present; full semantic ownership still unresolved'),implementation_requirements=['Do not reuse fixed-target class0 unchanged or substitute legacy gfx10 for canonical11736.','Resolve first-paint/frame0 versus first timer entry before adopting Clip::start.','Implement target incarnation and generation/source-owner retirement; retain native class2 following behavior.','Resolve shared-state override15/restoration and target visual callback semantics without local HP/MP mutation.','Qualify actual canonical frames, sound34 and full C02 positive/refused requests before registering player support.'],new_sessions=0,new_casts=0,new_renders=0,product_source_modified=False,request_coverage_incremented=False,renderer_qualified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',facts);(out/'auditor.py').write_bytes(Path(__file__).read_bytes())
pins={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in pins:z.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'archive-readback.json',dict(files=pins,sha256=sha(out/'evidence.zip'),verified=True))
print(json.dumps(dict(binding=str(out/'binding.json'),byte_predicates=len(expected),source_modified=False,new_casts=0,renderer_qualified=False),ensure_ascii=False))
