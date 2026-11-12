# Issue #341 — T059（M3 2/2）：音效裝置生命週期（G09）— 無裝置、裝置變更、同時多聲、切換場景與退出回收

- 階段：code；執行者：claude-local；分支：`task/341`
- 本張範圍：無裝置、裝置變更、同時多聲、切換場景與退出回收；裝置失敗不崩潰。
- 不在本張：原始 WAV 與影格事件（#340 已合併）；實際聽覺與實機拔插驗收（T030，`docs/GUI-ACCEPTANCE.md` A01–A12）。

## 結論

裝置失敗原本就不會崩潰（錯誤以 `Result` 回傳，沒有 panic 路徑），但有三個生命週期缺口，這次補上：

1. **無裝置被當成永久故障**。沒有輸出裝置時第一個音效就讓整個視窗的音效永久停用，之後接上裝置也不會恢復。現在無裝置不算故障：該音效丟棄，2 秒後的下一個音效再試一次開啟，裝置出現後自動恢復。
2. **裝置故障後永遠不恢復**。播放中拔掉裝置等情況會讓 voice pool 永久停用到關閉視窗。現在驅動放掉所有緩衝、且 5 秒內沒有新的錯誤後，pool 自己重新啟用，每個 pool 最多 8 次，超過就維持永久停用。
3. **關閉視窗後還可能開始新的聲音**。按下關閉到視窗真正銷毀之間，計時器仍在跑，新的畫面影格可以再送出音效。現在關閉中不再送出。

同時多聲、切換場景與退出回收的行為原本就符合要求，這次只補測試固定下來，沒有改行為。

本機只做了編譯檢查（`cargo check --all-targets`）；新測試沒有在本機執行，要看 PR 的 CI。實機的無裝置與拔插沒有測，列在「待驗證」。

## 做了什麼

### 1. 無裝置（`audio_output.rs`、`combat_audio.rs`）

- `AudioOutputError::device_unavailable()`：WinMM 回傳 `MMSYSERR_BADDEVICEID`(2)、`MMSYSERR_NOTENABLED`(3)、`MMSYSERR_ALLOCATED`(4)、`MMSYSERR_NODRIVER`(6) 而且沒有附帶清理失敗時為真。
- `PcmVoice::is_idle()`：驅動沒有持有這個 voice 的任何 header、PCM 或裝置 handle。不呼叫驅動。
- `Pool::play_prepared`：start 失敗時，若錯誤是「裝置不可用」而且 voice 是 idle（沒有任何東西交給驅動），就釋放該 slot、不進入故障，回傳新的 `PlayOutcome::DeviceUnavailable`，並記下 `DEVICE_RETRY_MILLIS`（2,000 ms）後才再嘗試開啟。期限內的音效直接丟棄，不呼叫驅動，所以沒有喇叭的機器上一批 256 個音效只會有一次失敗的 `waveOutOpen`。
- 已經在播的 voice 不受影響；丟棄的音效不排隊、不補播（送出端的 once-only latch 已經消耗）。
- `PoolStatus.device_unavailable`：最近一次開啟找不到裝置；下一次成功 start 時清除。`CombatVoicePool::unavailable_cues()` 是丟棄計數，只供診斷。
- 錯誤碼是「裝置不可用」但 voice 仍被驅動持有（例如 `waveOutWrite` 失敗後清理也失敗）時，仍然走故障路徑。

### 2. 裝置變更與故障恢復（`combat_audio.rs`）

- 故障時的行為不變：停掉所有 voice、每個 tick 重試清理、不配置新的 voice、期間所有音效回傳錯誤。
- 新增有上限的恢復：`poll` 在「所有 slot 都已釋放」「距離最後一次裝置錯誤（含清理失敗）已過 `FAULT_RECOVERY_MILLIS`（5,000 ms）」「恢復次數未達 `MAX_FAULT_RECOVERIES`（8）」三個條件都成立時清除故障。恢復只發生在 `poll`；`play` 與 `stop_all` 不會恢復。恢復本身不重播任何聲音。`CombatVoicePool::fault_recoveries()` 是恢復次數。
- 預設裝置切換：每個音效各自用 `WAVE_MAPPER` 開一個 handle，這點沒有改，所以新的音效會由 Windows 決定送到當下的預設裝置。

### 3. 一般入口視窗接線（`native_login_window.rs`）

- 新增 `audio_device_fault`：`audio_error` 是 pool 回報的裝置故障時為真。`poll` 回到 `Ok` 時清除這個錯誤，音效恢復送出。場景綁定或 bank 的錯誤（非裝置）維持原本的行為：停用到關閉視窗，而且不會被裝置錯誤覆蓋後又被清掉。
- 狀態列：無裝置時顯示「音效：找不到可用的音效輸出裝置，音效暫停；接上裝置後會自動恢復。」；裝置故障時維持原本的「音效：battle sound disabled after device failure: …」。
- 關閉中（`closing`）不再送出音效，只消耗事件。

### 4. 同時多聲、切換場景、退出回收（只加測試）

- 16 個 voice（`MAX_VOICES`）同時播放互不影響；第 17 個音效丟棄，不搶佔、不重開正在播的 voice；播完的 slot 才會被重用。
- 連續 50 次切換場景（`stop_all`）：正在播的 voice 各停一次，沒在播的 voice 不被碰，固定的 voice 可以重複使用。
- 退出時還在播：Drop 對每個 voice 停一次。裝置故障時退出：WM_CLOSE、關閉中的 tick、Drop 各嘗試釋放一次，不迴圈、不 panic。
- `PcmVoice` 層：無裝置時連續 start 都保持 idle，裝置出現後同一個 voice 正常播一次；播放中裝置消失（`unprepare`／`reset`／`close` 任一個回錯誤）時回傳錯誤、不釋放驅動仍可能讀取的記憶體、不能在上面排新的聲音，驅動恢復後可以再用。

### 5. 驗收清單（`docs/GUI-ACCEPTANCE.md`）

A07–A09 的「預期」改成這次的設計行為（狀態列文字、約 2 秒與約 5 秒的恢復時間、不補播），其餘不動。

## 怎麼驗證

### 已驗證

- 編譯：`cargo check --locked --offline -j1 --all-targets --target-dir target/current` 通過（exit 0）；本張改動的三個檔案沒有新警告。
- 查過 `docs/REUSE-MAP.md` 第 157 行與 tw-bin-engine `origin/main`（`41b0fe2`）`docs/reference/sound.md`：原版用 Miles Sound System（`AIL_quick_*`），文件沒有裝置失敗或拔插的處理可以照做。這次的設計只針對重建版自己的 WinMM 輸出，沒有引用 TW 的程式碼。

### 推測

- 新增與既有的單元測試會在 CI 通過。本機依規定沒有執行任何編譯出來的測試程式，以 PR 的 CI（`cargo test --locked --lib`）為準。新增的測試：
  - `audio_output::tests::missing_device_leaves_the_voice_idle_and_a_later_start_plays_normally`
  - `audio_output::tests::only_clean_no_device_codes_are_classified_as_unavailable`
  - `audio_output::tests::device_removed_while_playing_is_reported_and_never_frees_owned_memory`
  - `combat_audio::output::tests::no_output_device_drops_cues_without_fault_and_opens_once_per_retry_interval`
  - `combat_audio::output::tests::device_lost_for_new_cues_leaves_already_playing_voices_alone`
  - `combat_audio::output::tests::no_device_code_with_a_buffer_still_owned_by_the_driver_is_a_fault`
  - `combat_audio::output::tests::device_fault_recovers_only_after_quiet_time_with_nothing_owned_and_only_a_bounded_number_of_times`
  - `combat_audio::output::tests::sixteen_simultaneous_voices_are_independent_and_a_seventeenth_cue_is_dropped`
  - `combat_audio::output::tests::scene_switches_and_exit_release_every_voice_exactly_once`
  - `combat_audio::output::tests::exit_on_a_failing_device_attempts_one_release_per_voice_and_does_not_panic`
- 沒有輸出裝置時 `waveOutOpen(WAVE_MAPPER)` 回傳的是上面四個錯誤碼之一。依據是 WinMM 的錯誤碼定義，沒有在無裝置的機器上實測。若實際回傳其他錯誤碼，行為會退回「故障後約 5 秒恢復、最多 8 次」，仍然不會崩潰。
- 所有測試都用 fake backend，驗證的是重建版自己的狀態機，不是 Windows 驅動的實際行為。

### 待驗證

- **實機無裝置**（A07）：停用所有輸出裝置後啟動並戰鬥，確認不崩潰、狀態列出現無裝置提示；重新啟用裝置後下一個音效恢復。
- **實機拔插與切換預設裝置**（A08–A10）：播放中拔掉裝置時 WinMM 實際回傳什麼（錯誤、不再前進、或自動轉到新裝置）沒有測。三種情況分別對應故障恢復、#340 的逾時重設、不需處理。
- **`waveOutReset` 在卡住的裝置上會不會自己卡住**：所有 WinMM 呼叫都在 UI 執行緒上同步執行，驅動卡住時視窗會跟著卡住。這次沒有改這個架構（#340 已列）。
- **播放中關閉程式**（A11）與長時間播放（A12）。
- **既有的忽略測試** `native_audio_device`（需要實際裝置）沒有執行；它的報告欄位 `device_unplug_or_no_device_verified` 仍是 `false`。
- 狀態列的無裝置提示在裝置接回後，要等下一個音效成功開始才會消失（沒有另外查詢裝置清單）。

## 未驗證／沒做

- 沒有執行任何測試程式或客戶端；沒有開音效裝置。
- **舊入口視窗（`login_window.rs` 的 `WorldAudioOutput`、`BattleAudioOutput`）沒有接故障恢復**：它們自己的 `failed` 旗標仍是一次故障就停用到關閉視窗。無裝置的處理在 pool 內，所以這兩條路徑也不再因為無裝置而停用。一般入口（`native_login_window`）已接恢復。
- `scene_preview`（開發用預覽）直接用單一 `PcmVoice`，無裝置時會以錯誤訊息結束預覽，不是崩潰；沒有改。
- 沒有監聽 `WM_DEVICECHANGE`：恢復靠固定的時間間隔，不靠裝置通知。
- 沒有改 `docs/G01-G12.md`、`docs/G01-G12-AUDIT.md` 的 G09 狀態（G09 還需要 T030 的實際裝置證據）。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/audio_output.rs`
- `LiTo381-Modern/src/modern_lito/combat_audio.rs`
- `LiTo381-Modern/src/modern_lito/native_login_window.rs`
- `docs/GUI-ACCEPTANCE.md`（A07–A09 的預期）
- `docs/reports/issue-341.md`

## 給下一棒的提示

- **T030／實機測試**：A07–A10 請記下狀態列「音效：…」的原文與恢復所花的時間。若無裝置時看到的是「battle sound disabled after device failure: waveOutOpen failed (MMRESULT n)」而不是無裝置提示，代表實際錯誤碼不在 `audio_output.rs` 的 `DEVICE_UNAVAILABLE_CODES` 裡，把 n 回報即可補上。
- **T037（稽核）**：`docs/G01-G12-AUDIT.md` 第 142 行的 G09 列寫「無裝置、拔插、切換場景與退出回收還沒做（T059）」，本張合併後可以改成只剩 T030 的實際裝置證據。
- 若要讓舊入口視窗也能恢復，改 `world_audio.rs` 的 `WorldAudioOutput::failed` 與 `battle_playback.rs` 的 `Delivery::failed`：裝置故障改看 `pool.status().faulted`，非裝置錯誤才保留旗標。
- 調整 `DEVICE_RETRY_MILLIS`、`FAULT_RECOVERY_MILLIS`、`MAX_FAULT_RECOVERIES` 只需要改 `combat_audio.rs` 開頭的常數；測試裡的時間點由這些常數算出。
