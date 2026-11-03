"""Document verified Debug scope while the specific Release/normal pipeline runs."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
work=read(APP/'object-gate-product-current.json');assert work['tool_session']==94643 and work['profile']=='Release'
qpath=Path(work['Debug_qualification']);q=read(qpath)
assert q['source_files']==1019 and (q['passed'],q['failed'],q['ignored'],q['actual_test_executables'])==(135,0,6,23)
pins=read(qpath.parent/'compiled-inputs.json')
for n,h in pins.items():assert hashlib.sha256((ROOT/n).read_bytes()).hexdigest().upper()==h,n
lead=('2026-10-16來源1019進行中：純Rust物件selector已對263423原route/參數/gate數值核對，Raster已接floor＋普通RLE物件寫入來源，保留TIL真header。'
    '修正版Debug23實際x64 EXE/24群135過0敗6既有opt-in忽略、source/test/raw獨立讀回，同58895實際exit0。'
    'Release及唯一入口正常兩主正沿同94643執行，來源/對應QA凍結；尚未當作通過。'
    'raw物件預覽仍列unqualified fragments，完整map303背景/viewport/clock未接，當版正常GUI未驗；1005 TW14僅保留原版資格。完整G01–G12未完成。')
detail=('初輪同94991實際exit1：22整合104過，library Raster10過1 Hidden tone錯誤，world_scene群未執行；23實際測試EXE/source/原log與failed-scope-raw已封存讀回，未當合格。'
    '另依原first-pass的signed四byte header守衛覆核，raw半片的預覽rectangle不能代替原RLE覆蓋。'
    '修正Hidden在SPR提交時略過、Normal轉普通覆蓋，從原TIL資料表保存prefix並核對RLE解析header；raw物件只記未具資格，不產生假的完整gate。'
    '新增raw來源完整性案例，保留原畫面/命中、黑色覆蓋、屋頂weight31與special混色斷言。改後12項Raster與19項world_scene均過，六個既有opt-in仍未執行。')
for n in ['README.md','docs/ARCHITECTURE.zh-TW.md']:
    p=ROOT/n;s=p.read_text(encoding='utf-8-sig');assert lead not in s
    title,body=s.split('\n',1);p.write_text(title+'\n\n'+lead+'\n'+body,encoding='utf-8')
for n in ['docs/HISTORY.md','docs/NATIVE-FLOOR-GATE.zh-TW.md']:
    p=ROOT/n;s=p.read_text(encoding='utf-8-sig');assert lead not in s
    p.write_text(s+'\n\n## 2026-10-16 物件分派與RLE覆蓋來源\n\n'+lead+'\n\n'+detail+
        '\n\n[修正版Debug限定資格](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-131624-113375/profile-qualified.json)；'+
        '[初輪失敗保留](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-130245-955826/failed-scope-readback.json)。\n',encoding='utf-8')
assert all(hashlib.sha256((ROOT/n).read_bytes()).hexdigest().upper()==h for n,h in pins.items())
print('Recorded Debug135 qualified scope and explicit live Release/normal pipeline94643')
