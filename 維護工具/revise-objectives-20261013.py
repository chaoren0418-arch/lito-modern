"""Reconcile the working goal with schema38 evidence; preserve prior claims."""
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent.parent
out = root / '驗證紀錄/current/goal-revision-20261013'
out.mkdir(parents=True, exist_ok=True)
def read(p): return json.loads(p.read_text(encoding='utf-8-sig'))
def write(p, value): p.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
def sha(p): return hashlib.file_digest(p.open('rb'), 'sha256').hexdigest().upper()
queue = read(root/'工作佇列.json')
assert queue['schema'] == 2, 'One-time migration; do not overwrite subsequent work'
progress = read(root/'監製進度.json')
baseline = read(root/'驗證紀錄/current/native-emblems/summary.json')
checks = []
for key, path, expected in [
    ('formal_source', baseline['formal_reference']['source'], baseline['formal_reference']['sha256']),
    ('formal_snapshot', baseline['formal_reference']['snapshot'], baseline['formal_reference']['sha256']),
    ('debug_baseline', root/baseline['exe']['path'], baseline['exe']['sha256']),
    ('release_baseline', root/baseline['release_exe']['path'], baseline['release_exe']['sha256']),
]:
    actual = sha(Path(path))
    checks.append(dict(kind=key, path=str(path), sha256=actual, matches=actual==expected))
assert all(x['matches'] for x in checks), 'Baseline changed; investigate before revision'
for name in ['工作佇列.json','監製進度.json']:
    dest=out/name
    assert not dest.exists(), 'Archive already exists'
    dest.write_bytes((root/name).read_bytes())
now = datetime.now(timezone.utc).isoformat()
objective = ('完成以現行正式 JAR 為契約的 Windows 原生 x64 LiTo 客戶端：先完成兩名實際連線玩家的登入、房間、權威戰鬥、結算及重入；'
             '再逐項補齊多人同步、玩法、既有資源呈現、完整 GUI 與故障恢復，最後完成 Debug／Release 全流程、音效、可攜包及穩定性驗收。'
             '每項須綁定來源版本、實作與實測證據；舊版通過、展示注入和局部資源成功均不得代替當前整體完成。')
def item(id, title, deps, steps, gates, legacy=(), state='pending', evidence=()):
    return dict(id=id,title=title,state=state,depends_on=deps,work=steps,acceptance=gates,
                previous_items=list(legacy),evidence=list(evidence),complete=False)
items=[
 item('G01-current-baseline','重建現況與版本基準',[],
      ['核對來源 JAR、固定副本及已發布 EXE 雜湊','將舊版結果移出當前完成狀態；整合重複項目'],
      ['四項雜湊一致且修訂前佇列已保存','每個剩餘項目有依賴、驗收條件；未驗證不得標完成'],
      ['D-latest-formal-reference-revalidation','E-current-formal-compatibility'], 'verified_limited',
      ['驗證紀錄/current/goal-revision-20261013/revision.json']),
 item('G02-real-peer-battle','兩名實際玩家的雙場戰鬥與結算',['G01-current-baseline'],
      ['建立共用 GameHub 的兩組正式 LOGIN／GAME ClientSession 和兩個原生客戶端',
       '移除探針對訓練 bot／actor 2 的依賴，以正式角色身份追蹤對手',
       '完成移動、近戰、受傷、死亡確認、勝敗明細、返回及同連線第二場，互換勝敗'],
      ['兩端與正式核心的身份、座標、HP、結果及比分一致','傷害／HP／勝敗不由 fixture 注入',
       '兩場世代隔離、取消及全部 session／房間／頻道回收有證據'],
      ['E-real-peer-integration','E-world-events-combat','E-combat-controls-outcomes','E-combat-continuous-input'], 'in_progress'),
 item('G03-current-regression','現行協定與錯誤路徑回歸',['G02-real-peer-battle'],
      ['逐一盤點舊 54 情境，列為保留／改写／失效並說明原因','重驗分段 IO、交接、通知、選角、頻道、房間、世界及版本變更',
       '補拒絕、延遲／漏回覆、過期世代、斷線與重新登入'],
      ['每個現行必備情境均有當前 JAR／EXE 的結果','不得沿用舊版計數宣稱通過；失敗修復後重跑相關情境'],
      ['D-formal-wire-foundation','D-isolated-handshake','D-isolated-login-selection','D-game-handoff-characters','D-channel-lobby','D-room-membership']),
 item('G04-multiplayer-world','多人移動、碰撞及世界生命週期',['G02-real-peer-battle'],
      ['驗證雙向八方向移動、修正、追擊及動態角色阻擋','補加入／離開／死亡／復活／換圖及同 ID 替換的同步',
       '核對速度、移動鎖、動作中斷、狀態及持續輸入'],
      ['雙端可見世界與權威狀態一致','無穿人／卡死、舊世界動作或選取殘留；錯誤可恢復'],
      ['D-world-bootstrap-render','E-world-updates-motion','E-world-animation-movement-state','E-world-speed-collision-events']),
 item('G05-gameplay-matrix','物品、裝備、技能及狀態完整性',['G02-real-peer-battle'],
      ['從正式核心建立所有玩家可達操作的支援矩陣，標明缺失／拒絕／不適用',
       '補遠程與彈藥、已學技能、耗材／MP、buff／debuff、復活、物品使用／裝卸／丟棄／拾取',
       '用實際對手重驗成功及拒絕，連接 UI 與原始動畫／特效'],
      ['矩陣每個必備操作均有封包、權威狀態、呈現及失敗路徑證據','沒有以展示封包或本地 HP 變更冒充玩法成功'],
      ['E-combat-ranged-skills','E-gameplay-completeness']),
 item('G06-room-service-branches','房間管理與跨服務分支',['G03-current-regression'],
      ['核實建房、密碼房、設定、踢人、觀戰、隊伍及可達模式的正式契約',
       '補 rental-source C_CA、S_FB、租裝村／商店及交接；正式核心未支援者保留證據與限制'],
      ['每個正式可達分支可實際進出、拒絕與返回','不得以 Tournament 通過代替租裝或其他模式'],
      ['D-rental-service-routing','E-playable-flow']),
 item('G07-resource-rendering','既有資源與完整世界呈現',['G01-current-baseline'],
      ['盤點實際資源格式、命名空間、地圖／角色／物品／UI／特效／聲音支援及失敗來源',
       '補全體／地面名稱、徽章位置、FA／S69 状態，核對遮擋、屋頂、附件、透明及特效排序',
       '擴充到主要玩法使用的資源；未知格式明確失敗並提供來源診斷'],
      ['可重跑矩陣綁定資源 SHA 與實際呈現證據','現有切片只保留為局部成功；無靜默缺圖／錯資源或資源大量複製'],
      ['B-x64-resource-input','C-original-ui-resource-map','E-native-emblems','E-resource-coverage']),
 item('G08-player-gui','唯一入口的完整玩家操作',['G04-multiplayer-world','G05-gameplay-matrix','G06-room-service-branches','G07-resource-rendering'],
      ['逐項操作登入→來源→選角→頻道→房間→戰鬥→背包／技能→結算→返回→重入',
       '補焦點、快捷鍵、滑鼠命中、中文 IME、縮放／多 DPI、取消、錯誤提示與正常退出'],
      ['以實際 GUI 操作及畫面證據驗收，headless 探針不能代替','失焦不持續發招；無法操作或布局遮擋均修復'],
      ['E-player-ui-integration']),
 item('G09-audio','原始音效與裝置生命週期',['G07-resource-rendering'],
      ['驗證原始 WAV／影格事件、實際裝置播放與音量','補無裝置、裝置變更、同時多聲、切換場景及退出回收'],
      ['有實際裝置證據；PCM 解碼成功不能代替聽覺／播放驗收','無重複事件、無限聲音或裝置失敗導致崩潰']),
 item('G10-release-package','Release 全流程與可攜交付',['G03-current-regression','G08-player-gui','G09-audio'],
      ['Debug／Release 執行相同新版必備情境矩陣','製作唯一入口可攜包、資源路徑設定、依賴與失敗診斷'],
      ['同一候選來源與 JAR 下兩版本全流程通過','乾淨輸出位置能啟動且不依賴舊 BIN；封包含來源／EXE／資源版本清單'],
      ['F-release','F-release-integration']),
 item('G11-stability-performance','效能、長時間及故障恢復',['G10-release-package'],
      ['先保存雙玩家 Release 載入／幀時間／記憶體基準及驗收預算，避免事後改標準',
       '至少 60 分鐘、20 次完整場次循環；在登入／房間／戰鬥／返回注入斷線',
       '量測快取、worker、handle、音效、取消與退出；修復超限／持續增長'],
      ['實測達到事先記錄的預算，無崩潰、卡死、持續洩漏或錯誤權威','故障可回到可操作狀態，不能只以單次成功驗收'],
      ['E-world-performance-events']),
 item('G12-final-audit','整理與整體完成稽核',['G11-stability-performance'],
      ['保存唯一原始碼／逆向／回復證據後整理已核實冗餘產物','核對所有必備項目的驗收、已知限制、重跑指令及現行來源漂移'],
      ['G01 至 G11 驗收成立、沒有未處理必備缺口','完成聲明與可重跑收據一致；若外部來源變更先重驗受影響項目'],
      ['A-inventory-cleanup'])
]
items[0]['complete']=True
gates=['全部 G01–G12 的必備驗收成立；不得以計數或子集 PASS 代替整體完成',
       '每份結果綁定來源／固定副本／EXE／測試範圍；版本漂移使受影響驗證重新待驗',
       '未支援但正式可達的必備功能仍是缺口；不適用需來源證據與理由',
       'GUI、實際音效裝置、Release 與長時間驗收須各自有證據']
queue.update(schema=3, objective=objective, objective_revision=2, objective_updated_utc=now,
             objective_basis='使用者要求重寫目標項目，禁止直接沿用舊長期目標',
             items=items, next_task='G02-real-peer-battle', completion_gates=gates,
             integration_priority=[i['id'] for i in items if not i['complete']],
             historical_queue='驗證紀錄/current/goal-revision-20261013/工作佇列.json',
             execution_policy='逐項實作與驗證，階段完成後直接前進；簡短工作中更新，不因階段回報停止',
             last_verified_baseline={'schema':38,'evidence':'驗證紀錄/current/native-emblems/summary.json',
                                     'tests':baseline['tests'],'current_working_tree_verified':False})
queue.pop('previous_formal_reference_schema37',None)
queue['app_goal_status_note']='App 舊目標仍 blocked；介面不支援編輯或恢復未完成目標。新版執行項目在本佇列，不宣稱已改 App 目標，也不偽造完成繞過。'
progress.update(schema=3, objective=objective, objective_revision=2, next_task=queue['next_task'],
                completion_gates=gates, goal_complete=False, overall_status='in_progress',
                active_queue='工作佇列.json', results_scope='schema38 歷史已發布基準；本次修改後的工作樹待建置與驗證')
progress['app_goal_status_note']=queue['app_goal_status_note']
write(root/'工作佇列.json',queue)
write(root/'監製進度.json',progress)
write(out/'revision.json',dict(utc=now, objective_revision=2, baseline_checks=checks,
    previous_queue_sha256=sha(out/'工作佇列.json'), previous_progress_sha256=sha(out/'監製進度.json'),
    new_queue_sha256=sha(root/'工作佇列.json'), active_items=len(items),
    unrepresented_legacy_items=[x['id'] for x in read(out/'工作佇列.json')['items']
                              if not any(x['id'] in i['previous_items'] for i in items)],
    app_goal_changed=False, product_complete=False))
assert not read(out/'revision.json')['unrepresented_legacy_items']
print('Revised 12 objectives; all previous items mapped; baseline hashes match; app goal unchanged.')
