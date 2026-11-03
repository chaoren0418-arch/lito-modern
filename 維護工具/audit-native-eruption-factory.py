"""Read-only primary factory research; no current renderer or namespace inference."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,hashlib,json,struct,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
normal=V/'native-retired-join-normal/validation-20261018-063154-317211';inputs=read(normal/'compiled-inputs.json')
assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h
qualified=read(next(normal.glob('Debug-qualified-*/qualified.json')));assert qualified['passed']==90
primary=ROOT.parent/'lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe';pe=primary.read_bytes()
assert hashlib.sha256(pe).hexdigest().upper()=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
e=struct.unpack_from('<I',pe,60)[0];assert pe[e:e+4]==b'PE\0\0'and struct.unpack_from('<H',pe,e+4)[0]==0x14c
count=struct.unpack_from('<H',pe,e+6)[0];sections=e+24+struct.unpack_from('<H',pe,e+20)[0];assert struct.unpack_from('<I',pe,e+24+28)[0]==0x400000
def raw(a,n):
    for i in range(count):
        size,start,physical,at=struct.unpack_from('<IIII',pe,sections+i*40+8)
        if start<=a-0x400000 and a-0x400000+n<=start+min(size,physical):return pe[at+a-0x400000-start:at+a-0x400000-start+n]
    raise ValueError(hex(a))
table=struct.unpack('<8I',raw(0x4a63c4,32));assert table[6]==0x4a60ff and table[5]==0x4a60b2 and table[0]==0x4a6234
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
ranges={'type7':(0x4a60ff,0x4a6166),'type6':(0x4a60b2,0x4a60ff),'type1':(0x4a6234,0x4a62e7)}
instructions={name:list(md.disasm(raw(lo,hi-lo),lo))for name,(lo,hi)in ranges.items()}
ops={(i.mnemonic,i.op_str)for i in instructions['type7']}
assert {('push','0x160'),('movsx','ecx, word ptr [ecx + edx*2]'),('call','0x4238f0')}<=ops
asset=V/'native-attack-five-resource-inspection/20261017-144953-005315/asset-inventory.json';aq=read(asset.parent/'qualified/qualified.json');assert sha(asset)==aq['raw_files'][asset.name]
inventory=read(asset);rows={x['graphics_id']:x for x in inventory['rows']if x.get('graphics_id')in[129,130,123,124,1811]}
attrs=lambda row:{v['key']:v['body']for v in row['attributes']}
assert attrs(rows[129])[109]=='7 123'and rows[129]['image_count']==4 and rows[129]['directionality']=='EightWay'
assert attrs(rows[123])[109]=='6 -1'and attrs(rows[129])[101]=='130'and attrs(rows[123])[101]=='124'
contract=V/'native-current-skill-contract-audit';cq=read(contract/'qualified.json');factories=contract/'current-learned-factories.json';assert sha(factories)==cq['file_pins'][factories.name]
spell=next(x['fields']for row in read(factories)['rows']if row['skill_id']==45 for x in row['matches']if x['factory']=='attackSpell')
assert (spell['effectGfx'],spell['mpCost'],spell['maximumRange'],spell['reuseDelayMillis'])==(129,20,8,100)
formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar';assert sha(formal)==cq['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
out=V/'native-eruption-factory-binding'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,(lo,hi)in ranges.items():
    (out/(name+'.bin')).write_bytes(raw(lo,hi-lo))
    (out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in instructions[name])+'\n',encoding='utf-8')
(out/'factory-table.bin').write_bytes(raw(0x4a63c4,32));save(out/'retained-legacy-rows.json',rows)
record=dict(status='primary_type7_factory_and_retained_legacy_graph_bound_only',source_manifest_sha256=qualified['source_manifest_sha256'],primary_PE=dict(path=str(primary),sha256=sha(primary),reference_machine=0x14c,executed=False),
    retained_inventory=dict(path=str(asset),sha256=sha(asset),qualification=str(asset.parent/'qualified/qualified.json'),qualification_sha256=sha(asset.parent/'qualified/qualified.json')),
    retained_namespace='base legacy list.spr inventory only; current TW129 definition not bound here',formal_skill45=spell,formal_contract_qualification_sha256=sha(contract/'qualified.json'),
    factory_table_address='0x4A63C4',factory_table=[f'0x{a:08X}'for a in table],type7_branch='0x4A60FF',type7_constructor='0x4238F0',allocation_bytes=0x160,
    retained_legacy_parent=129,retained_legacy_child=123,retained_legacy_child_effect_type=6,
    branch_calls={name:[i.op_str for i in ins if i.mnemonic=='call'and i.op_str!='0x4f118c']for name,ins in instructions.items()},
    observations=['Primary type7 branch allocates0x160, reads the signed child table word and calls4238F0.','Retained base129 declares type7 child123; child123 declares type6 and its own Hit/Effect3080 timeline.','The retained four-image/EightWay discrepancy remains unresolved. No EightWay indexing or finite-class1 substitution is inferred.'],
    pending=['Bind current TW129/123/130/124 definitions and selected raw members without assuming the base namespace is canonical.','Trace4238F0 entry/timer/direction/position/retirement and type6 child behavior, including shadows and Effect3080.','Only after those proofs prepare canonical resources and qualify final-source requests, C02 positive/refused, audio and peer visibility.'],
    current_TW_graphics45_binding_qualified=False,current_source_request45_registered=False,normal_product_source_modified=False,corresponding_frozen_QA_modified=False,new_resource_pack_read=False,new_sessions=0,new_casts=0,new_renders=0,renderer_qualified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p)for p in out.iterdir()if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in pins:z.write(out/n,n)
with zipfile.ZipFile(out/'evidence.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
for n,h in inputs.items():assert sha(ROOT/n)==h
print(json.dumps(dict(binding=str(out/'binding.json'),type7_constructor=record['type7_constructor'],branch_calls=record['branch_calls'],current_TW_binding_qualified=False,new_casts=0),ensure_ascii=False))
