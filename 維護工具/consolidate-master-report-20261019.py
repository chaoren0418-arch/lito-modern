"""Consolidate the explicitly requested master report; retain its exact original."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT.parent / 'LiTo381-總開發報告.md'
V = ROOT / '驗證紀錄/current'
SOURCE = '2DAF4DE08EF396B1DE5B2264820E689859BFCD837015D71A6D74643A2DDEE9DE'

def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()

def link(label, path):
    return f'[{label}](<{path.as_posix()}>)'

broad = V / 'native-catalog-default116/20261019-005048-222068'
observation = read(broad / 'completion-observation.json')
queue = read(ROOT / '工作佇列.json')
checkpoint = Path(queue['last_goal_turn_evidence']).parent
readback = read(checkpoint / 'readback.json')
assert checkpoint.parent == broad and checkpoint.name.startswith('checkpoint-completed-')
assert len(readback['after_files']) == 8
assert all(sha(ROOT / name) == digest for name, digest in readback['after_files'].items())
assert queue['current_product_identity']['source_manifest_sha256'] == SOURCE
assert not queue['active_workload_sessions'] and not queue['source_and_corresponding_QA_frozen']
assert not queue['goal_complete'] and not observation['whole_goal_complete']
assert observation['source_manifest_sha256'] == SOURCE
assert observation['original_session_id'] == 45847 and observation['terminal_tool_chunk'] == '74870e'
assert observation['observed_terminal_exit_code'] == 0
assert observation['actual_test_executables'] == 234
assert observation['recorded_original_owned_processes_all_terminal']
assert observation['recorded_original_owned_identities'] == 481
for profile, passed in [('Debug', 544), ('Release', 540)]:
    assert observation['profiles'][profile]['counts'] == dict(passed=passed, failed=0, ignored=23)

normal_path = V / 'native-same-endpoint-catalog-normal/validation-20261018-232351-306737/qualified/qualified.json'
controller_path = V / 'native-catalog-controller/20261019-000332-603879/qualified/qualified.json'
handoff_path = V / 'native-catalog-handoff/completed-20261019-000156-153029/qualified.json'
audio_path = V / 'native-catalog-audio-cohort/20261019-002450-475847/qualified/qualified.json'
hotel_path = V / 'native-catalog-hotel/20261019-004008-982025/qualified.json'
floor_path = V / 'native-catalog-hotel-floor-provenance/20261019-005858-400246/readback.json'
evidence = [normal_path, controller_path, handoff_path, audio_path, hotel_path, floor_path,
            broad / 'completion-observation.json', checkpoint / 'evidence.json', checkpoint / 'readback.json']
normal, controller, handoff, audio, hotel = [read(p) for p in evidence[:5]]
assert all(item['source_manifest_sha256'] == SOURCE for item in [normal, controller, handoff, audio, hotel])
assert controller['actual_formal_sessions'] == 96
assert handoff['actual_sessions'] == 12
assert audio['actual_sessions'] == 96
assert hotel['actual_formal_sessions'] == 32
assert handoff['Debug_original_wrapper_failure_retained']
assert not hotel['natural_expiry_verified'] and not hotel['manual_guest_exit_verified']
for profile in ['Debug', 'Release']:
    result = normal['profiles'][profile]
    assert (result['passed'], result['failed'], result['ignored']) == (16, 0, 0)
    assert sha(Path(result['normal']['path'])) == result['normal']['sha256']

stamp = datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out = V / 'master-report-consolidation' / stamp
out.mkdir(parents=True, exist_ok=False)
old = REPORT.read_bytes()
assert len(old) == 2461990 and old.decode('utf-8-sig').startswith('# LiTo 3.81C 總開發報告與接手指南')
archive = out / 'LiTo381-總開發報告.整理前完整原文.md'
archive.write_bytes(old)
assert archive.read_bytes() == old
old_sha = sha(archive)
updated = datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M（Asia/Taipei）')

body = f'''# LiTo381 總開發報告與接續指南

更新：{updated}。本報告以可核對的當版成果為準；完整遊戲重建仍在進行，G01 基準已建立，G02–G12 尚未整體驗收。

## 1. 專案與目前停點

- **重建版**：開發真正的新遊戲核心，Windows x64 原生客戶端優先。工作目錄為 {link('lito重建版', ROOT)}，唯一現行入口為 {link('啟動現代客戶端.ps1', ROOT / '啟動現代客戶端.ps1')}。
- **登入器／伺服器**：與重建版各自獨立。可只讀參考其協定、資源與行為線索；先前替換登入器 JAR 的提案已撤回，不作為重建版完成依賴。
- 舊總報告最後記錄為 2026-09-07 Server 0.7.862／Launcher 0.5.54；這是歷史版本紀錄，本次未重新驗收其現況。歷史「總目標完成」限當時登入器／伺服器範圍，不代表重建版完成。
- **本次停點：當版 Debug／Release 廣域回歸已完成並獨立核對；完整 library 工具已準備、尚未啟動；當版正常 GUI 自動測試尚未執行。** 工作佇列已解除本批建置保留，沒有登記中的產品測試工作。

## 2. 當版基準與已完成範圍

當版來源識別為來源清單檔 SHA-256（1,047 個來源檔、1,048 個編譯輸入）：

```text
Source  {SOURCE}
Debug   {normal['profiles']['Debug']['normal']['sha256']}
Release {normal['profiles']['Release']['normal']['sha256']}
JAR     75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371
Catalog B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01
```

正式 JAR 只讀來源為 {link('登入器 app/server 的 candidate.jar', ROOT.parent / 'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar')}；測試使用重建版內固定副本。正常 EXE 路徑見 {link('正常兩版資格紀錄', normal_path)}。

最近正常程式修正：同端點傳送後，伺服器只在 LOGIN 發送的租裝服務目錄會遺失。現已將可接續的同端點房間路由綁定原擁有者、角色與精確端點，僅在正確的新 GAME 連線接續一次；新目錄優先，取消／失敗時清除，舊場景與操作權威仍須退休。變更為 native_teleport.rs、native_login.rs 及對應整合測試。

| 當版驗證 | 實際結果 | 資格限制／證據 |
|---|---|---|
| 正常入口建置與相關測試 | Debug／Release 各 16 通過、0 失敗、0 忽略（services 2＋teleport 14） | {link('資格紀錄', normal_path)}；僅相關範圍 |
| 廣域預設回歸，含 Release | Debug 544／Release 540 通過；各 0 失敗、23 忽略 | 每版 116 個整合目標＋main，共 234 個實際測試 EXE；{link('整批完成核對', broad / 'completion-observation.json')} |
| 跨服務控制器 | 兩版六分支、12 案、96 LOGIN／GAME sessions | 跨端點、逾時、拒絕、錯服務、取消重試、取消到期；{link('資格紀錄', controller_path)} |
| 同端點租裝接續 | 兩版合計 12 sessions | 租裝目錄、物品接續、舊場景拒絕及繼續移動；{link('資格紀錄', handoff_path)} |
| Presentation／音效診斷 | 兩版六分支、12 案、96 sessions | 舊事件拒絕、新事件一次；正常來源本體＋隔離診斷 API，未驗正常 GUI／音效裝置；{link('資格紀錄', audio_path)} |
| 正常 EXE 旅館服務 | 兩版八案、32 sessions | 房型／樓層、租退鑰匙、共享與房主退租驅離；{link('資格紀錄', hotel_path)} |

以上四組服務驗證共 **236 sessions**，是隔離測試連線數，不能換算為完整遊戲、玩家人數或 GUI 驗收。相關正常建置與廣域測試數也不相加為不重複測試總數。

廣域原始 session 45847 已實際 exit 0（終止紀錄 74870e），481 組記錄程序身分均已終止；{link('完成 checkpoint', checkpoint / 'readback.json')} 已同步八份接續檔。23 個忽略案例本文未執行，廣域通過不涵蓋完整 library。

渲染補充：map501 六個灰色取樣點已核對原 S32／TIL 的實際像素寫入，不能直接判為未繪製畫布；{link('像素來源核對', floor_path)} 僅證明這些取樣，不代表整張地圖、相機或正常 GUI 已驗收。

## 3. 剩餘開發與驗收缺口

| 項目 | 接續工作 |
|---|---|
| G02／G04 雙玩家與多人世界 | 在最終來源驗證雙場戰鬥、結算、同步、碰撞與生命週期；舊版本證據保留原資格 |
| G03 完整回歸 | 執行當版 Debug／Release 完整 library；盤點忽略案例，必要案例另行具備資源並實跑 |
| G05 玩法完整性 | 補物品、裝備、技能及狀態矩陣；歷史技能書 74 項中辨識 37／缺 37，辨識不等於施法成功；舊能力／特效結果不得轉記當版完整通過 |
| G06 服務與房間（優先 G06.3d） | 普通 CA00 同端點通道的已知失敗、非預設素質接續、旅館自然到期與訪客主動離房、其他可達服務／世界旅行及正常 GUI 操作 |
| G07 畫面與原始資源 | 全地圖、動作圖、遮擋、屋頂、排序、相機與完整 frame；診斷 PNG 與局部像素證明分開記錄 |
| G08 GUI 自動測試 | 正常 EXE 的登入→選來源／角色→入場→雙玩家→租裝／旅館→傳送／取消／恢復→登出；涵蓋實體輸入、IME、多 DPI 與視窗生命週期 |
| G09 音效 | 正常 GUI 與實際裝置播放、切換／失效／恢復；診斷 Player 通過僅保留自身範圍 |
| G10–G12 交付 | Release 全流程、乾淨主機可攜套件、效能／長時間／故障恢復，最後逐項整體稽核 |

已知限制保留原貌：同端點 Debug 原 wrapper exit 1 是舊範圍名稱斷言失敗，後續獨立讀回通過限定 runtime 範圍，沒有改寫原 exit 或重跑冒充原成功。普通 CA00 失敗及各工具讀回／checkpoint 初次失敗皆保留原始紀錄；後續修正不抹除失敗歷史。

## 4. 下一步順序與通過條件

1. **先補當版完整 library 基準**：接續已完成的廣域批次，執行 Debug＋Release 全庫；核對來源、實際 EXE、原始輸出、實際 exit、測試清單與全部自有程序終止。明列忽略項目，不能將忽略算通過。
2. **繼續完成剩餘開發**：優先 G06.3d 的普通同端點、素質、旅館退出／到期等缺口，再推進 G02／G04／G05／G07／G09。每項補上可重現案例，按來源版本記資格。
3. **每次產品變更後完成 Release 回歸**：經唯一入口建置正常兩版，先跑變更相關測試，再按影響範圍跑廣域、完整 library 與服務端到端。現有 2DAF4DE0 的結果不能直接搬到新來源；來源未變時保留既有合格結果，失敗或新疑慮再重跑必要範圍。
4. **完成當版正常 GUI 全流程自動測試**：自行建立隔離帳號與本機服務、自行操作登入和測試視窗；依專案規則使用 computer-use 技能的現行 Windows 控制方式。現有私有桌面／訊息注入腳本只作參考，不能代替本版正常視窗、實體輸入與多 DPI 驗收。截圖、事件、封包與角色權威需互相核對。
5. **自動驗證完成後才交人工驗收**：再做乾淨主機 Release 套件、實際音效裝置、G11 長跑／效能／故障恢復與 G12 稽核。逐項滿足 G01–G12 才能宣告完整完成；未執行的項目明列待辦。

## 5. 新對話直接接續

先讀本報告，再讀 {link('重建版規則', ROOT / 'AGENTS.md')}、{link('工作佇列', ROOT / '工作佇列.json')}、{link('README', ROOT / 'README.md')}、{link('架構', ROOT / 'docs/ARCHITECTURE.zh-TW.md')} 與 {link('歷史', ROOT / 'docs/HISTORY.md')}。其他檔案內仍有歷史段落，現況以來源 SHA、最新完成 checkpoint 與原始資格紀錄交叉核對；舊 PASS 不沿用到新來源。

**第一個待執行動作是完整 library，尚未啟動，沒有可續讀的 library session。** 在重新核對工作佇列無其他活動工作及來源仍為 2DAF4DE0 後執行：

```powershell
python -B -X utf8 'D:/lineage開發/3-lito開發/lito重建版/維護工具/run-native-catalog-library-default.py' 'D:/lineage開發/3-lito開發/lito重建版/驗證紀錄/current/native-catalog-default116/20261019-005048-222068'
```

記錄實際回傳的 run 路徑及 session，使用 {link('library active checkpoint 工具', ROOT / '維護工具/checkpoint-native-catalog-library-active.py')} 登記；結束後用同目錄 qualify／observe／completed 工具獨立核對並同步接續狀態（參數先查 --help）。若新對話發現已有活動批次，先核對原 PID＋建立時間與原始收據再接續，不能因工具 session 無法連回就另開重複批次。

正常建置使用啟動現代客戶端.ps1 的 -BuildOnly -BuildProfile Debug 或 Release；Cargo 固定 --locked --offline -j1、x86_64-pc-windows-msvc 及 LiTo381-Modern/target/current。測試期間固定來源及對應 QA；計時中的服務測試不並行編譯。8 秒移動／就緒、60 秒 peer 階段、Release 與 G11 預算維持原設定。

開發與自動測試均在重建版內；相鄰登入器／伺服器／原始客戶端資源只讀，不碰正式玩家表、正式程序，不執行／包裝舊 LiTo.bin。由代理先完成可自動化流程，不再要求使用者登入、找測試視窗或輸入測試帳密；人工測驗安排在自測確認後。

## 6. 歷史封存

整理前完整原文已逐位元封存：{link('舊總報告（完整）', archive)}。原檔 2,461,990 bytes，SHA-256：

```text
{old_sha}
```

歷史報告用於追溯登入器／伺服器成果；當版重建測試原始紀錄仍保留在重建版驗證目錄。後續更新以修訂本報告的現況與待辦為主，避免反覆堆疊同一進度段落。
'''

# Stage and validate the entire document before replacing the explicitly authorized file.
staged = out / 'LiTo381-總開發報告.整理後.md'
staged.write_text(body, encoding='utf-8', newline='\n')
assert 5000 < len(body.encode('utf-8')) < 22000
links = re.findall(r'\]\(<([^>]+)>\)', body)
assert links and all(Path(path).exists() for path in links)
assert len(re.findall(r'^```', body, re.MULTILINE)) == 6
assert REPORT.read_bytes() == old
REPORT.write_bytes(staged.read_bytes())
assert REPORT.read_bytes() == staged.read_bytes()
assert sha(archive) == old_sha
assert all(sha(ROOT / name) == digest for name, digest in readback['after_files'].items())
receipt = dict(status='master_report_consolidated_and_read_back', report=str(REPORT),
               updated_taipei=updated, old_bytes=len(old), old_sha256=old_sha,
               archive=str(archive), new_bytes=REPORT.stat().st_size, new_lines=len(body.splitlines()),
               new_sha256=sha(REPORT), all_local_links_exist=len(links),
               current_source_manifest_sha256=SOURCE, broad_checkpoint=str(checkpoint),
               cited_evidence_sha256={str(path): sha(path) for path in evidence},
               canonical_eight_unchanged_during_report_edit=True,
               full_library_started=False, GUI_qualified=False, whole_goal_complete=False)
(out / 'readback.json').write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps(receipt, ensure_ascii=False))
