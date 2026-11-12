# Issue #340 — T059（M3 1/2）：音效裝置生命週期（G09）— 原始 WAV 與影格事件

- 階段：code；執行者：claude-local；分支：`task/340`
- 本張範圍：原始 WAV 與影格事件；不能有重複事件或無限聲音。
- 不在本張：無裝置、裝置變更、同時多聲、切換場景與退出回收（#341，T059-2）；實際聽覺驗收（T030）。

## 結論

盤點客戶端全部原始 WAV 與 list.spr 的影格音效標記後，找到三個實際缺口並修正，另補一道「聲音不會無限佔用 voice」的保護：

1. **266 個原始 WAV 會被嚴格解析器拒絕**（`fmt ` 區塊 40 bytes、尾端全 0），其中 69 個被 list.spr 的 `<` 標記引用。現在 265 個可以解析；剩下的 `6169.wav` 另有結尾少 pad 的問題，維持拒絕。
2. **同一影格的多個 `[` 會同時全部播出**（list.spr 有 2,734 個這種影格，應該只抽一個候選）。現在每個影格只進一個音效。
3. **WAV 沒有播放長度上限**（解析器允許 1 Hz，16 MiB 可以播半年以上）。現在超過 60 秒的 WAV 不會進入 bank。
4. **裝置一直回報「播放中」時 voice 永遠不釋放**。現在 voice pool 在「PCM 長度 + 5 秒」後重設該 voice 一次，不重播。

本機只做了編譯檢查（`cargo check --all-targets`）與唯讀資源盤點；新測試沒有在本機執行，要看 PR 的 CI。

## 做了什麼

### 1. 原始 WAV 盤點（唯讀）

新增 `維護工具/audit-shared-sound-wav.py`：只讀 `<client-root>/Sound`，逐條對照 `audio::parse_pcm_wav` 與 `PreparedSound::from_wav` 的規則；加 `--list-spr` 時再統計影格音效標記。不開裝置、不播放、不寫客戶端目錄。

對 `D:\lineage開發\0-客戶端\Sound` 的結果（5,566 個數字檔名 WAV，456,702,121 bytes；檔名與大小清單 SHA-256 `303C2910…BE4ED00D`）：

| 項目 | 修正前 | 修正後 |
|---|---|---|
| 通過嚴格解析 | 4,949 | 5,214（其中 1..9999 的 5,013 個、≥10000 的 201 個） |
| 被拒：`fmt ` 40 bytes（PCM、cbSize 0、尾端 22 bytes 全 0） | 266（`2412`–`2414`、`6123`–`6493` 區間） | 0 |
| 被拒：最後一個非 data 區塊少 pad byte | 0（`6169.wav` 先被上一列擋下，算在 266 內） | 1（`6169.wav`，保留拒絕，見「待驗證」） |
| 被拒：IMA ADPCM（0x0011） | 351（`27643`–`27993`，全部 ≥10000） | 351（不屬於 shared 1..9999，維持拒絕） |
| 空 PCM | 0 | 0 |
| 超過 60 秒 | — | 0（最長 `361.wav` 22,203 ms；中位數約 1.07 秒；最短 `1.wav` 44 ms） |

格式分布（修正後通過者）：22050 Hz／16-bit／mono 4,524、22050／16／stereo 562、44100／16／mono 79、44100／16／stereo 35、8-bit mono 14（22050／11025／22000 Hz）。`fmt ` 長度：16 bytes 4,538、18 bytes 411、40 bytes 265（加上被拒的 `6169.wav` 共 266）。結尾 data 少 pad 的變體 2 個（原本就支援）。

`Sound/` 內另有 163 個 mp3（音樂）、`0.mid`、1 個 `.scc`，不是本張的 WAV 路徑。

### 2. `fmt ` 40 bytes 變體（`audio.rs`）

`parse_format` 原本只收 16 bytes，或 18 bytes 且 cbSize 為 0。現在另外接受 40 bytes，條件是第 16 byte 起全部為 0（cbSize 0，沒有宣告任何延伸內容）。尾端有任何非 0、其他長度、非 PCM 的 tag（含 0xFFFE extensible）仍然拒絕。

### 3. 影格音效：一個影格只進一個候選（`native_combat_animation.rs`）

`frame_sound` 原本對每個 `Sound(id)`（`[`）都回傳該 id，所以 `24.0:6[1537[1538` 會同時進兩個音效。`<` 群組原本就只在第一個標記抽一個。現在 `[` 與 `<` 走同一條路：先用 `classify_native_cues` 分類整個影格，只有影格第一個標記帶出抽到的候選；抽到 0 是靜音；分類失敗（超過 3 個候選、`[`／`<` 混用、音效排在非音效之後、id ≥ 10000）一律不發聲。

依據：本 repo `combat_animation::NativeAudioGroup` 的定義（候選是加權擇一，不是同時播放），以及 tw-bin-engine `docs/reference/sound.md` 2.1 節（commit `41b0fe2`，`sub_A3EF40`：候選數 1 取第 0 個，其餘 `rand() % n`，抽到 0 略過）。

list.spr（SHA-256 `523C61AD…E8B9F127`，4,067,417 bytes）的影格音效標記統計：

| 每影格候選數 | `[` | `<` | `^` |
|---|---|---|---|
| 1 | 10,588 | 12,985 | 143 |
| 2 | 2,466 | 4,536 | — |
| 3 | 264 | 2,493 | — |
| 4 | — | 52 | — |
| 5 | 4 | 45 | — |

`[`／`<` 混用的影格 0 個；音效排在 `!`／`>`／`]` 之後的影格 0 個；候選值為 0 的 0 個；id ≥ 10000 的 0 個。所以這次改動對真實 list.spr 的影響只有：2,730 個 2–3 候選的 `[` 影格從「全部播」變成「抽一個」，4 個 5 候選的 `[` 影格（第 58710 行附近）從「全部播」變成「不發聲」。

同一個函式也被 `native_world_animation.rs:403` 使用，一起修正。

### 4. 播放長度上限（`combat_audio.rs`）

新增 `MAX_PLAYBACK_MILLIS = 60_000`。`PreparedSound::from_wav` 在長度（PCM bytes ÷ byte rate，無條件進位）超過時拒絕，整個 bank 載入失敗，不發布部分可播的 bank。新增 `PreparedSound::playback_millis()`。原始 WAV 最長 22.2 秒，沒有檔案受影響。

### 5. voice 逾時重設（`combat_audio.rs`）

`CombatVoicePool` 的每個 slot 在成功 start 時記下期限＝開始時間＋`playback_millis`＋`VOICE_OVERRUN_GRACE_MILLIS`（5,000 ms）。`poll` 發現裝置仍回報播放中而且已過期限時，對該 voice 呼叫一次 `stop`（waveOutReset → unprepare → close），釋放 slot，計入 `expired_voices()`；不重播、不換新 voice。`stop` 失敗時沿用既有規則：pool 進入 fault 並保留該 slot。自然播完或 `stop_all` 的 voice 不計入。時鐘是 `Instant`，測試用可注入的時鐘。

### 6. 確認不會要求裝置循環（`audio_output.rs`，只加測試）

每次 start 只送出一個 `WAVEHDR`，`dwFlags`、`dwLoops` 都是 0，`dwBufferLength` 等於 PCM 長度。程式沒有改，只在 fake backend 記錄送出的 header 並加斷言。

## 怎麼驗證

### 已驗證

- 原始 WAV 與 list.spr 的數字：本機執行 `python -I -X utf8 維護工具\audit-shared-sound-wav.py "D:\lineage開發\0-客戶端" --json scratch\sound-audit-final.json --list-spr <已解出的 list.spr>`。list.spr 取自 `scratch\wt\task-338\scratch\issue-338\extract\list.spr`（#338 解出，SHA 與 `combat_audio.rs` 測試內的 `523c61ad…` 相同）。影格統計用獨立的正規表示式，不是 Rust 解析器。
- 266 個 40-byte `fmt ` 檔的尾端全為 0：盤點工具套用新規則後，因 `fmt ` 被拒的檔案數為 0。
- 編譯：`cargo check --locked --offline -j1 --all-targets`（`target/current`）通過，exit 0；本張改動的檔案沒有新警告。

### 推測

- 新增與既有的單元測試會在 CI 通過。本機依規定沒有執行任何編譯出來的測試程式，要以 PR 的 CI（`cargo test --locked --lib`）為準。新增的測試：
  - `audio::tests::original_forty_byte_zero_tail_pcm_fmt_is_accepted_and_nothing_else`
  - `combat_audio::tests::playback_length_is_bounded_before_a_sound_can_be_published`
  - `combat_audio::output::tests::voice_reported_playing_past_its_bounded_length_is_reset_once_and_never_restarted`
  - `combat_audio::output::tests::naturally_finished_and_explicitly_stopped_voices_are_not_counted_as_expired`
  - `combat_audio::output::tests::failed_reset_of_an_overrunning_voice_latches_the_fault_without_another_start`
  - `audio_output::tests::every_start_queues_exactly_one_unlooped_header_of_the_exact_pcm_length`
  - `native_combat_animation::tests::frame_sound_selects_one_candidate_and_only_on_the_first_cue`
  - `native_combat_animation::tests::weighted_frame_cues_enter_exactly_one_sound_per_action_and_never_replay`
- LiTo.bin（3.81C）的影格音效抽選規則與 TW 3.81 相同。依據是 TW 的反組譯結論與本 repo 既有的 `NativeAudioGroup` 定義；沒有在 LiTo.bin 上重新反組譯。

### 待驗證

- **實際聽覺**（T030）：多候選 `[` 影格現在只聽到一個音效；40-byte `fmt ` 的 266 個 WAV 能在實際裝置播出。
- **實際裝置上的逾時重設**：真的卡住的裝置上 `waveOutReset` 會不會自己卡住，本張沒有測。這屬於裝置失敗情境，建議 #341 一起看。系統睡眠後喚醒時，睡眠前還在播的 voice 會被判定逾期並重設（聲音被切掉，不會重播）。
- **既有的忽略測試**（需要客戶端資源或實際裝置，例如 `native_audio_device`、`real_shared_*`）沒有執行。
- **4–5 個候選的影格**（`<` 97 個、`[` 4 個）：`classify_native_cues` 上限是 3，這些影格現在不發聲。TW 文件說原版最多存 5 個候選、超過 5 才略過。要不要把上限改成 5 需要另外裁定，因為 `PreparedCue`、`validate_group` 與 `?` 傳播的呼叫端（例如 `native_skill_effects.rs:696`）都用同一個上限。
- **`^` 標記**（143 個影格、32 個 id）：TW 文件說原版是循環播放，直到發聲體銷毀。重建版沒有移植，分類後一律拒絕（不發聲），所以目前不會產生循環聲音；之後若要移植，必須同時做停止條件。
- **`[` 的 voice 規則**：本 repo 把 `[` 標成「同 id 播放中不重開」，一般影格音效走 `play_legacy`（允許重疊）；TW 文件說 `[` 與 `<` 行為相同、可重疊。本張沒有改這條規則。
- **`6169.wav`**：最後一個區塊 `DGDA`（923 bytes）結尾少 pad byte。解析器只對結尾的 `data` 區塊放行這種情況（既有測試明確要求其他區塊拒絕），list.spr 也沒有引用這個檔，所以維持拒絕。
- **list.spr 引用但 `Sound/` 沒有檔案的 id**：`[` 9 個（679、952、4755、6094、6095、6119–6122）、`<` 607 個。只查了 `Sound/` 目錄（重建版只從這裡讀 WAV），沒有用 PakViewer 查 pak，所以不宣稱是缺資源；重建版對這些 id 的行為是記錄 `prepared:false` 並不發聲。

## 未驗證／沒做

- 沒有執行任何測試程式或客戶端；沒有開音效裝置。
- 沒有改 `docs/G01-G12.md`、`docs/G01-G12-AUDIT.md` 的 G09 狀態（G09 仍未完成：#341 與 T030）。
- `expired_voices()` 目前只是 pool 上的計數，沒有接到 UI 或診斷輸出。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/audio.rs`
- `LiTo381-Modern/src/modern_lito/audio_output.rs`（只加測試）
- `LiTo381-Modern/src/modern_lito/combat_audio.rs`
- `LiTo381-Modern/src/modern_lito/native_combat_animation.rs`
- `LiTo381-Modern/src/modern_lito/native_combat_animation_tests.rs`
- `維護工具/audit-shared-sound-wav.py`
- `docs/reports/issue-340.md`
- 本機未追蹤：`scratch/sound-audit-final.json`（盤點完整輸出）

引用的唯讀來源：tw-bin-engine `origin/main`（`41b0fe2`）`docs/reference/sound.md` 第 2 節；`docs/REUSE-MAP.md` 第 157 行（音效觸發：list.spr 音效照做）。

## 給下一棒的提示

- **#341（T059-2）**：`CombatVoicePool::expired_voices()` 可以當「裝置卡住」的訊號；裝置拔除時 WinMM 可能回錯誤（走既有 fault 路徑），也可能只是不再前進（走這次的逾時重設）。兩條都要在無裝置／拔插情境確認。`native_login_window.rs:976` 每個 tick 都會 `poll`，逾時重設就在那裡發生。
- **T030**：聽覺核對請包含一個多候選 `[` 的動作（例如 list.spr 第 5 行 `24.0:6[1537[1538`、第 8 行 `[248[703[701`）與一個 40-byte `fmt ` 的音效（`<6123`–`<6493` 區間，69 個被引用）。
- 盤點工具可以重跑：換客戶端或 list.spr 版本時先跑一次，看「被拒」與「超過 60 秒」兩欄是否仍為預期。
