"""Independently verify actual original selector records and code relocations."""
from pathlib import Path
from datetime import datetime,timezone
import collections,csv,hashlib,json,struct,sys,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
OUT=Path(sys.argv[1]).resolve(strict=True);assert OUT.is_relative_to(ROOT/'驗證紀錄/current/native-background-dispatch-original')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
sha=lambda p:digest(p.read_bytes())
p=read(OUT/'preparation-readback.json');r=read(OUT/'run-receipt.json');s=read(OUT/'native-summary.json')
assert r['status']=='original_selector_ran_not_yet_independently_qualified' and r['owned_process'] is None
assert r['build.log_exit_code']==r['native-summary.json_exit_code']==0
assert s['cases']==p['cases']==263423 and s['original_bytes']==389 and s['relocations']==14 and s['guards_unchanged']
assert not s['object_pixels_executed']
pins=read(OUT/'compiled-inputs.json');assert len(pins)==1015
for n,h in pins.items():assert sha(ROOT/n)==h,n
assert sha(OUT/'preparation-raw.zip')==p['archive_sha256']
with zipfile.ZipFile(OUT/'preparation-raw.zip') as z:
    for n,h in p['files'].items():assert digest(z.read(n))==sha(OUT/n)==h,n
with (OUT/'private-addresses.csv').open(encoding='ascii',newline='') as f:addresses=list(csv.DictReader(f))
assert addresses[0]['kind']=='B';base=int(addresses[0]['address']);assert len(addresses)==15
old=(OUT/'selector-original.bin').read_bytes();assert len(old)==389
expected=bytearray(old+b'\xc3')
with (OUT/'relocations.csv').open(encoding='ascii',newline='') as f:relocations=list(csv.reader(f))
assert len(relocations)==14
for (kind,offset,key),actual in zip(relocations,addresses[1:]):
    offset,key=int(offset),int(key);assert actual['kind']==kind and int(actual['key'])==key
    address=int(actual['address']);value=(address-base-offset-4)&0xffffffff if kind in ['C','J'] else address
    struct.pack_into('<I',expected,offset,value)
assert bytes(expected)==(OUT/'selector-relocated.bin').read_bytes()
exe=(OUT/'oracle.exe').read_bytes();e=struct.unpack_from('<I',exe,60)[0]
assert exe[:2]==b'MZ' and exe[e:e+4]==b'PE\0\0' and exe[e+4:e+6]==b'\x4c\x01' and exe[e+24:e+26]==b'\x0b\x01'
def scalar(flags,mapid,options,header,shade,counter):
    if header:return (0,0,0,0)
    for flag,required in [(0x1000,1),(0x2000,2),(0x4000,4),(0x8000,8)]:
        if flags&flag and not options&required:return (0,0,0,0)
    if flags&0x40:return (1,0,0,0) if options&16 else (0,0,0,0)
    if flags&0xc00:return (8,shade,0,0) if 0<=shade<32 else (0,0,0,0)
    if flags&2:return (0,0,0,0)
    if flags&8:return (2,0,0,0)
    if flags&0x101:return (1,0,0,0)
    if flags&0x80:return (3,0,0,0)
    if flags&0x10:return (4,0,0,0)
    if flags&0x20:return (5,256,abs(12-counter%24)+16,0)
    if flags&0x200:return (6,0x10203040,0,0)
    return (7,0x101 if mapid==303 else 0x100,0x31415926,0)
raw=(OUT/'records-native.bin').read_bytes();assert len(raw)==263423*16
counts=collections.Counter();plain_gate=collections.Counter()
with (OUT/'cases.csv').open(encoding='ascii',newline='') as f:
    rows=list(csv.reader(f));assert rows.pop(0)==['case','flags','map','options','header','shade','counter']
assert len(rows)==263423
for i,(text,actual) in enumerate(zip(rows,struct.iter_unpack('<IIII',raw))):
    c=[int(v) for v in text];assert c[0]==i
    if i<262144:
        assert c[1:]==[i%65536,302+(i//65536)%2,[0,31][i//131072],0,15,0]
    want=scalar(*c[1:]);assert actual==want,(i,c,actual,want)
    counts[actual[0]]+=1
    if actual[0]==7:plain_gate[(c[2],actual[1]&255)]+=1
assert set(counts)==set(range(9)) and set(plain_gate)=={(302,0),(303,1)}
save(OUT/'scalar-readback.json',{'cases':len(rows),'route_counts':dict(counts),
    'plain_map_gate_counts':[{'map':k[0],'gate_low_byte':k[1],'cases':v} for k,v in sorted(plain_gate.items())]})
(OUT/'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
files={f.name:sha(f) for f in OUT.iterdir() if f.is_file() and f.suffix!='.zip'}
with zipfile.ZipFile(OUT/'run-raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(OUT/n,n)
with zipfile.ZipFile(OUT/'run-raw.zip') as z:
    for n,h in files.items():assert digest(z.read(n))==h
q={'status':'independently_verified_original_first_pass_object_selector_all_16bit_flags',
    'observed_utc':datetime.now(timezone.utc).isoformat(),'path':str(OUT),'source_files_unchanged':1015,
    'original_bytes':389,'appended_private_return_bytes':1,'relocations':14,'actual_cases':263423,
    'exhaustive_16bit_flag_cases':262144,'targeted_header_shade_option_counter_cases':1279,
    'route_counts':dict(counts),'original_instructions_only_allowed_relocations':True,'all_private_guards_unchanged':True,
    'actual_object_pixels_executed':False,'complete_scene_gate_proven':False,'modern_scene_adapter_verified':False,
    'game_sessions':0,'whole_goal_complete':False,'raw_archive_sha256':sha(OUT/'run-raw.zip'),
    'files':files,'all_archives_readback_verified':True}
assert not (OUT/'qualified.json').exists();save(OUT/'qualified.json',q);save(APP/'background-dispatch-qualified-current.json',q)
print(json.dumps({k:q[k] for k in ['status','actual_cases','route_counts','all_archives_readback_verified']},ensure_ascii=False))
