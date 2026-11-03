"""Record qualified normal mains before freezing documentation for the GUI cohort."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
NORMAL=APP/'object-gate-tests-20261016-132623-487623/normal-qualified'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
n=read(NORMAL/'qualified.json');s=read(NORMAL/'scoped-qualified.json');pins=read(NORMAL/'source-inputs.json')
assert len(pins)==1019 and s['status']=='qualified_object_gate_Debug135_Release135_and_normal_mains'
assert not (NORMAL/'GUI-current.json').exists()
for name,h in pins.items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest().upper()==h,name
lead=('2026-10-16來源1019：Debug/Release各23實際x64測試EXE、24群135過0敗6既有opt-in忽略與唯一入口正常兩主05F325C8/CD4F29A8已獨立讀回，'
    '同94643實際exit0。floor＋普通RLE物件write provenance及TIL真header已接Raster；raw物件預覽仍列unqualified fragments。'
    '當版TW14 GUI待啟動，來源與對應QA凍結；map303完整背景/viewport/clock與完整G01–G12未完成。1005 TW14保留原版資格。')
for filename in ['README.md','docs/ARCHITECTURE.zh-TW.md']:
    p=ROOT/filename;old=p.read_text(encoding='utf-8-sig');assert lead not in old
    title,body=old.split('\n',1);p.write_text(title+'\n\n'+lead+'\n'+body,encoding='utf-8')
for filename in ['docs/HISTORY.md','docs/NATIVE-FLOOR-GATE.zh-TW.md']:
    p=ROOT/filename;old=p.read_text(encoding='utf-8-sig');assert lead not in old
    p.write_text(old+'\n\n## 2026-10-16 source1019正常入口與兩版回歸\n\n'+lead+
        '\n\nRelease保持z／ThinLTO／CGU16／strip symbols，兩版135/0/6與23實際測試映像、來源及raw封存核對；六項ignored名稱回查既有來源，未執行本文。'
        '兩次正常入口BuildOnly的原輸出（含嚴格UTF8或CP950 roundtrip）及PE AMD64/PE32+、features空集合、正常EXE封存均獨立讀回。'
        '這不是最新完整default/lib/TCP或正常GUI資格；接著做當版TW＋extended14。\n\n'+
        '[正常兩主資格](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-132623-487623/normal-qualified/qualified.json)；'+
        '[限定Rust範圍](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-132623-487623/normal-qualified/scoped-qualified.json)。\n',encoding='utf-8')
print('Documented source1019 normal mains and both135/0/6; current GUI explicitly pending')
