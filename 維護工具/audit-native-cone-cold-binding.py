"""Bind retained canonical cold assets to primary PE type5 factory bytes only."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=ROOT/'驗證紀錄/current/native-class2-normal/validation-20261018-014519-231950';c=read(normal/'cohort.json');assert c['source_manifest_sha256']=='86E65C7B9D9D77FF73FF154B59F28465B75BA1B3C1D1C6EF315D4990584B45AE'
inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1035
for n,h in inputs.items():assert sha(ROOT/n)==h
original=ROOT.parent/'lito登入器/舊版分析與驗收';pe_path=original/'runtime/LiTo.unpacked-oep-current.exe';asm_path=original/'ida-client-oep-v2/full-disassembly.asm';pe=pe_path.read_bytes();asm=asm_path.read_bytes()
assert hashlib.sha256(pe).hexdigest().upper()=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert hashlib.sha256(asm).hexdigest().upper()=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
e=struct.unpack_from('<I',pe,60)[0];assert pe[e:e+4]==b'PE\0\0' and struct.unpack_from('<H',pe,e+4)[0]==0x14c
count=struct.unpack_from('<H',pe,e+6)[0];sections=e+24+struct.unpack_from('<H',pe,e+20)[0];assert struct.unpack_from('<I',pe,e+24+28)[0]==0x400000
def raw(a,length):
    for i in range(count):
        size,start,physical,at=struct.unpack_from('<IIII',pe,sections+i*40+8)
        if start<=a-0x400000 and a-0x400000+length<=start+min(size,physical):return pe[at+a-0x400000-start:at+a-0x400000-start+length]
    raise ValueError(hex(a))
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
ranges=dict(parser109=(0x43e6b2,0x43e6e2),factory_selector=(0x4a5fab,0x4a5fe0),factory_type5=(0x4a61cd,0x4a6234))
decoded={name:list(md.disasm(raw(lo,hi-lo),lo)) for name,(lo,hi) in ranges.items()}
parser=[(i.mnemonic,i.op_str) for i in decoded['parser109']]
for op in [('cmp','edi, 0x6d'),('mov','edx, dword ptr [0x647e5c]'),('mov','byte ptr [esi + edx], al'),('mov','ecx, dword ptr [0x647e50]'),('mov','word ptr [ecx + esi*2], ax')]:assert op in parser,op
selector=[(i.mnemonic,i.op_str) for i in decoded['factory_selector']]
for op in [('mov','cl, byte ptr [eax + edx]'),('dec','eax'),('jmp','dword ptr [eax*4 + 0x4a63c4]')]:assert op in selector,op
table=struct.unpack('<8I',raw(0x4a63c4,32));assert table[1]==0x4a5fe0 and table[4]==0x4a61cd
branch=[(i.mnemonic,i.op_str) for i in decoded['factory_type5']]
for op in [('push','0x160'),('mov','ecx, dword ptr [0x647e50]'),('movsx','ecx, word ptr [ecx + edx*2]'),('call','0x42f1f0')]:assert op in branch,op
asset=ROOT/'驗證紀錄/current/native-attack-five-resource-inspection/20261017-144953-005315/asset-inventory.json';aq=read(asset.parent/'qualified/qualified.json');assert sha(asset)==aq['raw_files'][asset.name]
tw=next(v for v in read(asset)['explicit_TW_namespace_inspections'] if v['namespace']=='launcher/TW13081901.pak');rows={v['graphics_id']:v for v in tw['rows'] if v['graphics_id'] in [11738,11739,11740,11741]};assert len(rows)==4
fly=rows[11738];hit=rows[11740]
assert fly['directionality']=='EightWay' and fly['image_count']==8 and fly['frames_per_second']==24
assert dict((v['key'],v['body']) for v in fly['attributes'])[109]=='5 11740'
assert dict((v['key'],v['body']) for v in fly['attributes'])[105]=='1 11739'
assert len(fly['frames'])==3 and not any(v['events'] for v in fly['frames'])
assert hit['directionality']=='Undirected' and hit['image_count']==1 and len(hit['frames'])==8 and hit['frames'][1]['events']==['Sound(715)','Hit']
assert dict((v['key'],v['body']) for v in hit['attributes'])[105]=='1 11741'
assert all('MissingAction' in rows[i]['resolution_error'] for i in [11739,11741])
contract=ROOT/'驗證紀錄/current/native-current-skill-contract-audit';cq=read(contract/'qualified.json');factories=contract/'current-learned-factories.json';assert sha(factories)==cq['file_pins'][factories.name]
spell=next(x['fields'] for v in read(factories)['rows'] if v['skill_id']==38 for x in v['matches'] if x['factory']=='attackSpell')
assert (spell['effectGfx'],spell['mpCost'],spell['maximumRange'],spell['reuseDelayMillis'])==(11738,18,6,10)
out=ROOT/'驗證紀錄/current/native-cone-cold-binding'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi) in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo));(out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in decoded[name])+'\n',encoding='utf-8')
(out/'factory-table.bin').write_bytes(raw(0x4a63c4,32));save(out/'canonical-rows.json',rows)
record=dict(status='primary_type5_cold_factory_and_retained_canonical_graph_bound_only',source_manifest_sha256=c['source_manifest_sha256'],primary_PE=dict(path=str(pe_path),sha256=sha(pe_path)),primary_ASM=dict(path=str(asm_path),sha256=sha(asm_path)),retained_asset_qualification=str(asset.parent/'qualified/qualified.json'),retained_asset_qualification_sha256=sha(asset.parent/'qualified/qualified.json'),retained_asset_inventory_sha256=sha(asset),retained_TW_pack_sha256=tw['pack_sha256'],new_resource_pack_read=False,formal_skill38=spell,formal_contract_qualification_sha256=sha(contract/'qualified.json'),effect_type=5,constructor='0x42F1F0',allocation_bytes=0x160,impact_graphics=11740,fly_overlay=11739,impact_overlay=11741,impact_sound=715,
    observed=['Parser109 stores its first scalar into byte table647E5C and second scalar into word table647E50.','Factory decrements type once and dispatches through4A63C4; type5 selects4A61CD, passes the word child ID and calls42F1F0.','Canonical11738 has eight directional images and no Sound/Hit event in its three fly frames; canonical11740 carries Sound715 and Hit.','11739/11741 lack standalone action0 definitions in the retained inventory; parent-layer sampling must be resolved rather than dropping or substituting them.'],
    pending=['Trace constructor/virtual entry, trajectory and timer cadence from PE before selecting a clock or target-follow rule.','Resolve overlay parent frame selection, blend operations, target incarnation and scene retirement.','Implement and qualify canonical native assets, actual C02 positive/refused requests, MP/cooldown and audio delivery before registering support.'],normal_product_source_modified=False,source_or_QA_override_used=False,new_sessions=0,new_casts=0,new_renders=0,request_support_registered=False,renderer_qualified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in pins:z.write(out/n,n)
with zipfile.ZipFile(out/'evidence.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
for n,h in inputs.items():assert sha(ROOT/n)==h
print(json.dumps(dict(binding=str(out/'binding.json'),effect_type=5,constructor='0x42F1F0',new_casts=0,renderer_qualified=False,whole_goal_complete=False),ensure_ascii=False))
