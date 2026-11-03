"""Retain semantic vectors only; no original instruction bytes enter the product."""
from pathlib import Path
import csv,hashlib,json,struct
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
out=Path(json.loads((APP/'water-clock-original-current.json').read_text())['output'])
q=json.loads((out/'qualified.json').read_text());assert q['status']=='original_clock_fragments_independently_verified'
def rows(name):
    with (out/name).open(encoding='ascii',newline='') as f:return list(csv.DictReader(f))
phase=rows('phase-vectors.csv');scroll=rows('scroll-vectors.csv')
data=bytearray(b'LWCT0001'+struct.pack('<II',len(phase),len(scroll)))
for row in phase:data.extend(struct.pack('<BBbBbBIB',*map(int,row.values())))
for row in scroll:data.extend(struct.pack('<HHhhHH',*map(int,row.values())))
footer=json.dumps({'states':rows('state-vectors.csv'),'queues':rows('queue-vectors.csv'),'original_archive_sha256':q['archive_sha256']},separators=(',',':')).encode()
data.extend(struct.pack('<I',len(footer)));data.extend(footer)
p=ROOT/'LiTo381-Modern/tests/fixtures/native-water-clock-original.bin'
assert not p.exists();p.write_bytes(data)
record={'fixture':str(p),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest().upper(),
 'phase_rows':len(phase),'scroll_rows':len(scroll),'queue_scenarios':15,'contains_original_machine_code':False,
 'native_archive_sha256':q['archive_sha256'],'format':'LWCT0001,u32 phase count,u32 scroll count,phase rows BBbBbBIB,scroll rows HHhhHH,u32 JSON footer size,JSON footer'}
(APP/'water-clock-fixture.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(record))
