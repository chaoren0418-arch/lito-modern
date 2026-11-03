"""Publish the resource-ready source checkpoint without claiming runtime acceptance."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import hashlib, json, re
from native_E149_recovery_contract import R, read, sha, encoded, require, check_pins

T = R / '維護工具'
V = R / '驗證紀錄/current'
REPORT = R.parent / 'LiTo381-總開發報告.md'
KEY = 'current_E149_static2941_ready_development'

def main():
    w = read(R / '工作佇列.json')
    ref = w[KEY]
    ep = Path(ref['evidence'])
    require(sha(ep) == ref['evidence_sha256'], 'evidence differs')
    e = read(ep)
    require(e['compiler_exit_codes'] == [0] * 20 and e['tests_executed'] == 0
            and e['source_ready_precedes_enqueue_source_bound']
            and e['Scene_and_History_source_bound']
            and e['cached_late_and_paint_token_checks_source_bound']
            and not e['normal_apply_eligible']
            and not e['all_native_global_callbacks_qualified'], 'source boundary differs')
    require(sha(e['compile_cohort']) == e['compile_cohort_sha256'], 'cohort differs')
    c = read(e['compile_cohort'])
    check_pins(c['normal_product_pins'])
    rb = read(ep.with_name('readback.json'))
    require(all(sha(R / n) == h for n, h in rb['after_files'].items()), 'checkpoint differs')
    old = REPORT.read_bytes()
    body = old.decode('utf-8-sig')
    marker = '**最新資源就緒 gate 與 Scene／History 接線已加入隔離候選**'
    if marker in body:
        print(json.dumps(dict(already_updated=True, master_sha256=sha(REPORT)), ensure_ascii=False))
        return
    require(sha(REPORT) == '423B8455A575E819AFE1AFFFDECEF96EF89E7E68D5C030F6D6A44ADAD9B5F4EF', 'master predecessor differs')
    out = V / 'master-report-current-work' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
    out.mkdir(parents=True)
    (out / 'before.md').write_bytes(old)
    entry = T / 'update-master-report-E149-current.py'
    (out / 'entry.before.py').write_bytes(entry.read_bytes())
    stamp = datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')
    body = re.sub(r'更新：[^（]+（Asia/Taipei）', '更新：' + stamp + '（Asia/Taipei）', body, count=1)
    a = body.index('- 最新隔離候選44檔：')
    b = body.index('\n\n## 已完成', a)
    body = body[:a] + '- 最新隔離候選44檔：30 Runtime／11 Rust QA／3 Java，372 module／原六 nested；資源就緒與畫面失效守衛已接，仍不可套用。正常來源／registry／既有PE未改。[來源與20項編譯紀錄](<' + ep.as_posix() + '>)；[最新八檔 checkpoint](<' + ep.with_name('readback.json').as_posix() + '>)；[工作佇列](<' + (R / '工作佇列.json').as_posix() + '>)。來源未凍結，無活動批次。' + body[b:]
    a = body.index('## 已完成的來源開發')
    b = body.index('## G06與尚缺前提', a)
    section = '''## 已完成的來源開發與目前界線

**最新資源就緒 gate 與 Scene／History 接線已加入隔離候選**。#97 的流程改為收包／dispatch → 等待實際 SPR bank → 資源就緒證明 → fresh enqueue；尚未就緒不建立 deadline。證明持有嚴格核對過的 bank，不能用布林值或假時間代替。Scene 背景載入與原 fragments／Over 提交、FiniteHistory、view 交付及 controller queue 已接；cached／late-worker／paint／input 共用失效 token，退役後不能靠舊快照或 root ABA 復活。載入錯誤在同來源範圍保留。

**97 尚未完成，候選仍不可套用。** 原共享 callback、重入繪圖與固定 sample 的交錯尚未補齊；原生 clock 初始化／rollover、實際資源與像素均未驗。現代流程會在資源證明交付後排程並 drain，不能據此宣稱原生可見時機等價，也不能假定固定一幀或永遠不可見。32 秒狀態期限仍由 server 掌管。

本輪六個 Runtime 覆寫，44檔／30 Runtime／11 Rust QA／3 Java、372 module／原六 nested 不變。Debug／Release 各 library、library-test、七技能與 network 共 **20 項 metadata 編譯全部退出0**；只產生 metadata，未連結 PE、未執行測試。原8個 queue 案改寫，新增4案，共12案（8純／4需原資源）；累計 **475案全未跑**，不是完整 library inventory。候選描述沿用的新增資源案數0已另註：按實際 QA 為新增1、原3重分類；不改已編譯 contract。技能請求來源88／剩92不變，並非88技能完整。

準備工具曾在讀取新增 module 的索引處失敗（9e69f3），當時尚未啟動 Rust 編譯；原工具與 partial 目錄保留，v2只修候選／正常清單取值。前輪 extern 命名失敗與所有原收據亦保留。IngressReceipt 的 decode 時刻仍在解密後、backpressure前；與 dispatch／constructor／resource-ready／enqueue 時刻分開。bank 只在 MissingEntry 使用1120後備，IO／損壞／取消不得降級。

97請求／S34、98請求／2942有限特效及既有技能來源沿用；成本／材料／隱身／中毒／自然期限皆由 server 決定。69同socket、87持續4183、constructor／方向初始化、LT hurt/base及 G06 原物品／source/account／timer 等未驗界線保留。正常1049輸入、既有PE、正式JAR、registry及G06 binding／640次計畫未改；未啟動測試、GUI、server／DB、SPR解碼或播放。

完整前輪狀態及歷史原證據見[此前完整報告與歷史證據](<HISTORY_BEFORE>)。

'''.replace('HISTORY_BEFORE', (out / 'before.md').as_posix())
    body = body[:a] + section + body[b:]
    a = body.index('1. 暫緩期間繼續開發：')
    b = body.index('2. 使用者明確恢復測試後：', a)
    body = body[:a] + '1. 暫緩期間繼續開發：補97原共享 callback／重入繪圖 producer 與固定 sample／巢狀 drain 交錯；確保合法時機可提交，callback 後不能再發佈。不固定一幀或借32秒期限；原 clock／rollover及獨立像素驗證留待測試。97完成再99／100及剩92，續4183、G06物品／source/account／timer與LT hurt/base。測試與桌面仍待使用者明確恢復。\n' + body[b:]
    body = body.replace('先四元件inventory、17原控制', '先跑當版12個 gate 案（4案需原資源），四元件inventory、17原控制')
    body = body.replace('不含最新42檔', '不含最新44檔')
    for label in ['此前完整報告與歷史證據', '舊映像與原始記錄']:
        body = re.sub(r'\[' + label + r'\]\(<[^>]+>\)', lambda _: '[' + label + '](<' + (out / 'before.md').as_posix() + '>)', body)
    require(all(Path(p).is_file() for p in re.findall(r'\]\(<([^>]+)>\)', body)), 'report link missing')
    require(REPORT.read_bytes() == old and all(sha(R / n) == h for n, h in rb['after_files'].items()), 'concurrent writer')
    (out / 'after.md').write_text(body, encoding='utf-8', newline='\n')
    (out / 'updater.py').write_bytes(Path(__file__).read_bytes())
    entry.write_text('"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("' + Path(__file__).name + '")),run_name="__main__")\n', encoding='utf-8', newline='\n')
    (out / 'entry.py').write_bytes(entry.read_bytes())
    REPORT.write_bytes((out / 'after.md').read_bytes())
    require(REPORT.read_bytes() == (out / 'after.md').read_bytes(), 'master differs')
    result = dict(before_sha256=hashlib.sha256(old).hexdigest().upper(), after_sha256=sha(REPORT), latest_evidence=str(ep), latest_evidence_sha256=sha(ep), lines=len(body.splitlines()), metadata_commands=20, tests_executed=0, normal_apply_eligible=False, whole_goal_complete=False)
    (out / 'readback.json').write_bytes(encoded(result))
    print(json.dumps(dict(master_readback=str(out / 'readback.json'), **result), ensure_ascii=False))

if __name__ == '__main__':
    main()
