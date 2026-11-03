"""Read current formal classfile Code directly; no sessions or server execution."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, struct, zipfile

ROOT=Path(__file__).resolve().parents[1]
V=ROOT/'驗證紀錄/current'

def read(p): return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as stream: return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(p,v): Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')

jar=ROOT/'LiTo381-Modern/target/current/server-reference/formal-server-C02AAAE8.jar'
formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
assert sha(jar)==sha(formal)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
contract=V/'native-current-skill-contract-audit'
qualified=read(contract/'qualified.json')
factories=contract/'current-learned-factories.json'
assert sha(factories)==qualified['file_pins'][factories.name]
spell=next(row['matches'][0]['fields'] for row in read(factories)['rows'] if row['skill_id']==46)
assert (spell['actionId'],spell['effectGfx'],spell['row'],spell['column'])==(18,1811,5,5)
with zipfile.ZipFile(jar) as archive:
    data=archive.read('local/lito381/server/Protocol.class')
assert data[:4]==b'\xca\xfe\xba\xbe'
at=8
def u1():
    global at
    result=data[at];at+=1;return result
def u2():
    global at
    result=struct.unpack_from('>H',data,at)[0];at+=2;return result
def u4():
    global at
    result=struct.unpack_from('>I',data,at)[0];at+=4;return result

pool=[None]*u2()
i=1
while i<len(pool):
    tag=u1()
    if tag==1:
        length=u2();pool[i]=(tag,data[at:at+length]);at+=length
    elif tag in (3,4):pool[i]=(tag,data[at:at+4]);at+=4
    elif tag in (5,6):pool[i]=(tag,data[at:at+8]);at+=8;i+=1
    elif tag in (7,8,16,19,20):pool[i]=(tag,u2())
    elif tag in (9,10,11,12,17,18):pool[i]=(tag,u2(),u2())
    elif tag==15:pool[i]=(tag,u1(),u2())
    else:raise AssertionError(tag)
    i+=1
def text(index):
    assert pool[index][0]==1
    return pool[index][1].decode('utf-8')
def member(index):
    tag,owner,name_type=pool[index];assert tag in (10,11)
    _,name,descriptor=pool[name_type]
    return text(pool[owner][1]),text(name),text(descriptor)
at+=6
interfaces=u2();at+=2*interfaces
for _ in range(u2()):
    at+=6
    for _ in range(u2()):
        u2();length=u4();at+=length
methods=[]
for _ in range(u2()):
    access,name,descriptor=u2(),u2(),u2()
    code=None
    for _ in range(u2()):
        attribute,length=u2(),u4();start=at
        if text(attribute)=='Code':
            code_length=struct.unpack_from('>I',data,at+4)[0]
            code=data[at+8:at+8+code_length]
        at=start+length
    if text(name)=='attackSpell':methods.append((access,text(descriptor),code))
assert len(methods)==1
access,descriptor,code=methods[0]
assert descriptor=='(IJJIIJIIIII)[B' and access&8 and len(code)==296
assert code[258]==3 and code[259]==0xb6
writer_c=member(struct.unpack_from('>H',code,260)[0])
assert writer_c==('local/lito381/server/Protocol$Writer','c','(I)Llocal/lito381/server/Protocol$Writer;')
assert code[255]==0xb6
assert member(struct.unpack_from('>H',code,256)[0])==('local/lito381/server/Protocol$Writer','h','(I)Llocal/lito381/server/Protocol$Writer;')
assert code[262:264]==bytes([0x15,10]) and code[264]==0xb6
assert member(struct.unpack_from('>H',code,265)[0])==('local/lito381/server/Protocol$Writer','h','(I)Llocal/lito381/server/Protocol$Writer;')
# Account for every writer in the successful straight-line suffix. Dwords use
# long parameters, but occupy exactly four protocol bytes in Writer.d.
writer_calls=[]
for offset in [218,222,226,230,235,240,245,255,259,264,269,274,279,283]:
    assert code[offset]==0xb6
    owner,name,desc=member(struct.unpack_from('>H',code,offset+1)[0])
    assert owner=='local/lito381/server/Protocol$Writer'
    writer_calls.append(dict(code_offset=offset,method=name,descriptor=desc))
assert [x['method'] for x in writer_calls]==['c','c','d','d','c','c','d','h','c','h','h','h','h','c']
assert sum({'c':1,'d':4,'h':2}[x['method']] for x in writer_calls[:8])==18
out=V/'native-sunburst-formal-flight-binding'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
(out/'Protocol.class').write_bytes(data)
(out/'attackSpell.code.bin').write_bytes(code)
anchor=V/'native-sunburst-anchor-binding/20261018-085729-627880/binding.json'
assert read(anchor)['graphics']==1811 and not read(anchor)['actual_formal_flight_type_captured']
record=dict(status='static_formal_attackSpell_wire18_constant_zero_bound_only',
    primary_anchor_binding=dict(path=str(anchor),sha256=sha(anchor)),
    formal_jar=dict(path=str(formal),sha256=sha(formal)),owned_reference=dict(path=str(jar),sha256=sha(jar)),
    contract=dict(path=str(factories),sha256=sha(factories)),skill46=spell,
    class_sha256=hashlib.sha256(data).hexdigest().upper(),method='Protocol.attackSpell',descriptor=descriptor,
    method_code_sha256=hashlib.sha256(code).hexdigest().upper(),writer_calls=writer_calls,
    flight_type_wire_offset=18,flight_type_constant=0,constant_code_offset=258,
    inference='A skill46 execution using the inspected attackSpell writer emits flight_type0. This supports the candidate subset; actual skill46 dispatch and S23 capture remain required.',
    actual_formal_skill46_dispatch_executed=False,actual_formal_skill46_packet_captured=False,
    old_native_ambient_gate_for_zero_disabled=True,renderer_qualified=False,normal_product_source_modified=False,
    new_sessions=0,new_casts=0,whole_goal_complete=False)
save(out/'binding.json',record)
(out/'auditor.py').write_bytes(Path(__file__).read_bytes())
pins={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED) as archive:
    for name in pins:archive.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper()==digest for name,digest in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
print(json.dumps(dict(binding=str(out/'binding.json'),wire18_constant=0,actual_casts=0),ensure_ascii=False))
