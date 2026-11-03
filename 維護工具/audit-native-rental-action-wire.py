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
with zipfile.ZipFile(jar) as archive:
    data=archive.read('local/lito381/server/ClientSession.class')
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
    tag,owner,name_type=pool[index];assert tag in (9,10,11)
    _,name,descriptor=pool[name_type]
    return text(pool[owner][1]),text(name),text(descriptor)
at+=6
interfaces=u2();at+=2*interfaces
for _ in range(u2()):
    at+=6
    for _ in range(u2()):
        u2();length=u4();at+=length
methods={}
for _ in range(u2()):
    access,name,descriptor=u2(),u2(),u2()
    code=None
    for _ in range(u2()):
        attribute,length=u2(),u4();start=at
        if text(attribute)=='Code':
            code_length=struct.unpack_from('>I',data,at+4)[0]
            code=data[at+8:at+8+code_length]
        at=start+length
    if text(name) in ['rentalTownGameplayOpcodeAllowed','handleBeforeForcedCombat','action']:methods[text(name)]=dict(access=access,descriptor=text(descriptor),code=code)
assert set(methods)=={'rentalTownGameplayOpcodeAllowed','handleBeforeForcedCombat','action'}
allowed=methods['rentalTownGameplayOpcodeAllowed']['code'];dispatch=methods['handleBeforeForcedCombat']['code'];action=methods['action']['code']
assert allowed[1]==0xab
pairs=struct.unpack_from('>i',allowed,8)[0];assert pairs==26
entries=dict(struct.unpack_from('>ii',allowed,12+i*8)for i in range(pairs))
assert entries[31]+1==220 and allowed[220:222]==bytes([4,0xa7]) and allowed[224]==3
assert dispatch[109]==0xaa
low,high=struct.unpack_from('>ii',dispatch,116);assert(low,high)==(0,251)
target=109+struct.unpack_from('>i',dispatch,124+31*4)[0];assert target==1392 and dispatch[1394]==0xb6
assert member(struct.unpack_from('>H',dispatch,1395)[0])==('local/lito381/server/ClientSession','action','([B)V')
assert len(action)==391
assert member(struct.unpack_from('>H',action,5)[0])[1]=='BATTLE' and member(struct.unpack_from('>H',action,15)[0])[1]=='WORLD'
assert action[131:133]==bytes([0x10,32]) and action[133]==0xb6 and member(struct.unpack_from('>H',action,134)[0])[1]=='c'
assert action[143]==0x1d and action[144]==0xb6 and member(struct.unpack_from('>H',action,145)[0])[1]=='c'
assert member(struct.unpack_from('>H',action,154)[0])[1]=='rentalTownRoomState'
assert action[160]==0xb6 and member(struct.unpack_from('>H',action,161)[0])[1]=='rentalTownMembers'
assert action[239]==0xb6 and member(struct.unpack_from('>H',action,240)[0])[1]=='sendEncrypted'
inspection=V/'native-rental-action-audio-inspection/20261018-092853-536404/report.json'
audio=read(inspection);assert audio['action']==68 and audio['events']==[dict(sound=1534,source=2,ticks=4)]
out=V/'native-rental-action-wire-binding'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'ClientSession.class').write_bytes(data)
for name,method in methods.items():(out/(name+'.code.bin')).write_bytes(method['code'])
record=dict(status='static_current_formal_rental_C1F_to_S20_wave_path_and_selected_audio_data_bound_only',formal_jar=dict(path=str(formal),sha256=sha(formal)),owned_reference=dict(path=str(jar),sha256=sha(jar)),class_sha256=hashlib.sha256(data).hexdigest().upper(),methods={name:dict(descriptor=m['descriptor'],code_bytes=len(m['code']),code_sha256=hashlib.sha256(m['code']).hexdigest().upper())for name,m in methods.items()},rental_whitelist_opcodes=sorted(entries),request_opcode=31,request_action=68,expected_request_hex='1f44',server_action_opcode=32,server_action_packet='20 + own object u32LE + action68',selected_normal_audio_data=dict(path=str(inspection),sha256=sha(inspection)),selected_sound=1534,selected_sound_source=2,actual_C1F_sent=False,actual_S20_received=False,new_sessions=0,normal_product_source_modified=False,normal_GUI_action_API_permission_inferred=False,whole_goal_complete=False)
save(out/'binding.json',record);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());pins={p.name:sha(p)for p in out.iterdir()if p.is_file()}
with zipfile.ZipFile(out/'evidence.zip','x',zipfile.ZIP_DEFLATED)as archive:
 for name in pins:archive.write(out/name,name)
with zipfile.ZipFile(out/'evidence.zip')as archive:assert all(hashlib.sha256(archive.read(n)).hexdigest().upper()==h for n,h in pins.items())
save(out/'readback.json',dict(files=pins,archive_sha256=sha(out/'evidence.zip')))
print(json.dumps(dict(binding=str(out/'binding.json'),C1F_allowed=True,actual_requests=0),ensure_ascii=False))
