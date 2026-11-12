# RESOURCE-MATRIX — 資源命名空間與格式矩陣（G07.1）

本檔由 `維護工具/resource-matrix/resource_matrix.py` 產生（工具版本 1，#336／T057），**不要手改**；要更新就重跑工具。內容只取決於客戶端資源與 `LiTo381-Modern/src` 的現況，沒有時間戳，同一份資源重跑會得到相同位元組。

- 資源集合雜湊（下表所有來源的路徑／大小／SHA-256 串接後再取 SHA-256）：`74DCB59C7C1235A2A4F7A3F2EFA74F3EBF4EE9E2010B35D4D194C6F2DCC9F951`
- 列舉成員總數：**322,822**（封存記錄＋納入雜湊的散檔），每一個都落在 §2 的某一列；工具以加總核對。
- 逐成員清單（含每個成員的 SHA-256，約 48 MB）不入庫，重跑後在 `scratch/resource-matrix/members.tsv`；其 SHA-256 應為 `288B79D8CC0DC60E7CE5D9FFF893024EFF000C29ADD563970379970AE572A76C`。
- 例外清單（被關卡拒絕、未知格式、非資源、同名重複、分片路由不符等）：`docs/resource-matrix/exceptions.tsv`（22,844 筆）。
- 機器可讀版：`docs/resource-matrix/matrix.json`。

## 0. 怎麼讀這份矩陣

| 欄位 | 意思 |
|---|---|
| 命名空間 | `SHARED`＝天堂共用資源（客戶端根目錄）；`NATIVE_LITO`＝LiTo 專屬（`LiTo/`）；與 `native_resources.rs` 的 `ResourceNamespace` 對應。兩者之間**沒有**自動 fallback。 |
| 狀態 `supported_container_ok` | Rust 有這個格式的通用解碼器，而且這個成員通過「容器層關卡」。 |
| 狀態 `rejected_by_decoder_gate` | Rust 有解碼器，但這個成員會被關卡拒絕；判定欄是原因碼（G07.4 診斷要用的來源）。 |
| 狀態 `selected_members_only` | Rust 只消費指名的成員（例如特定 tbl／html），沒有通用解析器。 |
| 狀態 `no_decoder` | Rust 沒有這個格式的解碼器。 |
| 狀態 `not_game_resource` | 開發殘留、版控檔、壓縮檔等，不是遊戲資源。 |
| 字面引用 | 成員檔名以字面值出現在 `src` 非測試程式碼的個數；`format!` 組出來的名稱（例如 `{id}.spr`）抓不到，所以這是下限。 |

**界線**：判定是把 Rust 解碼器在解像素前的檢查照抄成 Python 逐成員跑的結果（已驗證的是「Python 關卡的結果」）。SPR／TIL 的 RLE 像素本體、影格尺寸與總像素上限沒有在這裡驗；「Rust 解碼器實際逐成員跑過」仍是待驗證，由 `LiTo381-Modern/tests/resource_matrix_exhaustive.rs`（`#[ignore]`）負責。

## 1. 來源綁定（SHA-256）

| 來源 | 大小（bytes） | SHA-256 |
|---|---:|---|
| `Sprite.idx` | 2,585,972 | `327C06372D87F6CF6D5176DA51DA5EF1F94DBF388D1588B735C5965B96B6A52B` |
| `Sprite.pak` | 1,904,012,586 | `4DD4E5348FBAD2E69BAE9CC388BE23D7BECEBE8423AD9F79F393C1457A69EE62` |
| `Sprite00.idx` | 285,800 | `15B46AE6995B5B3D64A27A509C78847F0329BA55360A200961A7ECB73B13C96C` |
| `Sprite00.pak` | 450,577,933 | `64741119E853880E0E31009CF06145A5F21520B5264D07ECE5541BA701A01C9D` |
| `Sprite01.idx` | 286,500 | `1E4C819DEBE37449691142025692AC1FF1D1F82C148830C76338E7FDFF8CDD8F` |
| `Sprite01.pak` | 422,714,941 | `A46A5A85E56C03CC6B1D7ED7CDDF3D59E8887612313842CDE9822B76932B5AC5` |
| `Sprite02.idx` | 288,964 | `A0208077CFA7E76280641F1A2BF7F7AEACC43B6F9A16E21771EBD732E791AE5C` |
| `Sprite02.pak` | 412,811,463 | `88858F8EAF57854D50EA21F2D84DD3D3E8C1D3FB30C5C583AF2175FC465CDE0C` |
| `Sprite03.idx` | 292,828 | `BB26979CF21F7655B3F5D57E2C0D152F8CA7E755AD66F1801A1EDB455245CD72` |
| `Sprite03.pak` | 422,963,990 | `3A1D8C976C70AD908F9B9F5DE16F65CBCFA1CAD47E5DE7F84E615E18C8363F8D` |
| `Sprite04.idx` | 296,608 | `7C801F623F68BDCA50271DBDDDC2FEFBCAC990E9E9EF60776F48DAA73F9CD1F5` |
| `Sprite04.pak` | 396,506,828 | `8F4BBB0C296208C3D6A7BA6FC666D313A25621650D455557E709EBBBABFD14DD` |
| `Sprite05.idx` | 299,604 | `659B4CD84092E9DE5B76B0FFA4793B562213B42BCF0A9771EB0E85E6AA5E379B` |
| `Sprite05.pak` | 370,799,127 | `9BB611E401B2F48E4F0A89F7EEB79165D4B182CA082E6C09B36B3A46D9F7F93E` |
| `Sprite06.idx` | 303,636 | `45AA43F0EC1A1480E82C27379BD25B433F960F27160B87BF5E0DFE9F67D831E4` |
| `Sprite06.pak` | 399,499,710 | `4514513774F4E591E1140EFC801FE390231F405F4D030C2F0701AAD6FF5E01B3` |
| `Sprite07.idx` | 306,576 | `C49468722B1265864CF66F169C09D1318F0DECD4817AF9F15CD8EBB67F6C98AB` |
| `Sprite07.pak` | 429,001,722 | `7E4E7797E16471ED9BF8ADBCE6724E21C992A7A860BB10832214D3E9A94663AE` |
| `Sprite08.idx` | 307,220 | `4EB34A830BBAE9D3C3CE112D83A2CC3DD96D0312BFA798ED59542F403924C63C` |
| `Sprite08.pak` | 405,698,718 | `4FD3A06362673F2E5A3CB1434777366D32C6EDAD1A330E400D092CE62FD32E04` |
| `Sprite09.idx` | 307,164 | `CC8B83FE4010EE5A3FA470466A414931BD2E61308972D76C610ADA8EA1C04896` |
| `Sprite09.pak` | 409,918,765 | `06B8A0A98BBBF25871957C06E767247D92BE49C9745AEC4FC8593BBFAB91E466` |
| `Sprite10.idx` | 304,140 | `7B4688CF2B01C7889A5E6EF8F06309C52C8AAD781537BFF92D23ED543E08FB8D` |
| `Sprite10.pak` | 401,370,637 | `BD19155DCCFC3A7C7DFC5ADAFA63CE63D65E6640A4A6D43F363093B3B3F245C2` |
| `Sprite11.idx` | 300,052 | `A190E3E9E54753D89B2AE1209910AD053E6BE1D857E895F01631E6DA9C7DD88E` |
| `Sprite11.pak` | 408,635,715 | `F2D47FF75C12E25A49EF1018BC74284535FB452770434BA17FC44D7637424A78` |
| `Sprite12.idx` | 294,620 | `2C271EF696788BA1F44ACAF0A8FE93E4ABAE5C88A57D6103E3CCB84C4B319541` |
| `Sprite12.pak` | 392,475,964 | `CF1FA59D39A9C3A4AD4AFCD2EF11382C48946F88BD5B29CF1A627F1AEE5A5A46` |
| `Sprite13.idx` | 290,000 | `333D0670A4E2FE47C51B62F4F60DE2A579B4B7BC5B9F0910BB2A7F79502C671B` |
| `Sprite13.pak` | 404,079,327 | `CE206C743F0F7744BB7153791BE27B6FB84E5C34F91B8CB92F2C6A520294C946` |
| `Sprite14.idx` | 287,144 | `51802133F6BE2BB7A8E727C58F5DA299B9C67C707506E2931FF63B1A4D4633CF` |
| `Sprite14.pak` | 442,917,643 | `55A73C6B6F3315F5E5EA81E7B4D9C7B120F61E41864297F01B4C1F306F65C6D4` |
| `Sprite15.idx` | 285,156 | `874B53AFE762C43415E25204F4E8FCF405F36ADFACA6B9258BD84C1EB1D4C441` |
| `Sprite15.pak` | 484,407,291 | `864800D2AFBD8B38ED9BFCC08A0079B27FBF53F018DE562F4CBA6FE54DCCABD2` |
| `Tile.idx` | 152,492 | `9291FD800AC8CAE6DC355C1D8BBB965880561DE169FBC7EA22D174FFF9AD24DB` |
| `Tile.pak` | 593,320,968 | `82E52F47317F9A55EF775BBFCE3C5F26F42B31A98748FFAA84CE5E2C3ED60D4A` |
| `Text.idx` | 1,049,948 | `43A342AD3929B541096630165C04794192C48359ECBF116B538EAFFC49741F6F` |
| `Text.pak` | 26,267,704 | `9516A372C43B269CF1AF7ADC0AC00ABD4163E1235DCD846E34294849616A37FF` |
| `LiTo/LTicon.idx` | 6,360 | `2EA2E48A61C246A5D4CB2820C4C3431F487705C67459290A998D9DBE9017BAA7` |
| `LiTo/LTicon.pak` | 274,258 | `52FEC7B547B63B75BE1178D66E4C0E17EA5B48F089B47E1836EDD120C79C015D` |
| `LiTo/LTsprite.idx` | 13,136 | `73536646591E04BFB62B55320E075F0C6D5F8DC17FE29BC012C7925EED670BD8` |
| `LiTo/LTsprite.pak` | 63,004,112 | `7029D561D8EF1420128D318E827BAE07D552E5316839CDDB362D9223F2FB3962` |
| `LiTo/LTsurf.idx` | 45,448 | `C699CCF22C5F59EB37E8936D939D2632C15AC381980A6FE3F26F2565185193BF` |
| `LiTo/LTsurf.pak` | 92,788,879 | `42E676C255B248C295A71097A53B1B67BF73A0C2C3DC0FE11D12BDB9EE79621E` |
| `LiTo/LTtext.idx` | 9,888 | `B8C0BB8C43AFFE59A90EBD2D26182269B6D60AF388C80963EAAEEDC8D8AE23C8` |
| `LiTo/LTtext.pak` | 292,553 | `517D853E3FE91B504953FF4B9B2EED682A3612D063F2D18E75E39FD5580F342D` |
| `LiTo/LTtile.idx` | 33,884 | `EFD13705C355BAAE6E1C95C88E24FDCDB2948D17F8936167486F97C8C0DE330E` |
| `LiTo/LTtile.pak` | 148,485,306 | `6E80F46CA19D32BFA53FB360D2398755F4BBBA7383A6B4CAD811BCB79F910D22` |
| `[tree] LAUNCHER（1 檔）` | 1,406,534 | `933396032FB8FBAD62C5DA5E916D9091D657AC6CA2F49816BD850A7883C5E79B` |
| `[tree] NATIVE_LITO/LTSurf（1 檔）` | 614,408 | `8E42893A6654DDD927BB1F877B6EDDF0CC7FFB52F61339ABE72118554CA24DDA` |
| `[tree] NATIVE_LITO/LTmap（497 檔）` | 52,285,895 | `99FCEBADD92456E2D20E321144023F805C8B200037DB2520440710E256A8AF21` |
| `[tree] NATIVE_LITO/LTsound（49 檔）` | 40,643,868 | `A6A8D765713AA381EE68D23A950B6EDCCE4416029D493E9DBB78A6E5660BC744` |
| `[tree] NATIVE_LITO/loose（25 檔）` | 703,012 | `AF28CC29E6801D92DEAE428CD19E12D2EBE1B118834DA3EE59C300954D2A2167` |
| `[tree] SHARED/Font（11 檔）` | 519,940 | `8391B47DB3726D778EB590C8EAAFE8BF96DFCBF47C5D159D2FFBB8B74B3D5362` |
| `[tree] SHARED/Sound（5,732 檔）` | 736,079,636 | `7673B2753DFC8DD40C13107F9961A633644714CFB24ECB1491C571490765DAED` |
| `[tree] SHARED/map（8,180 檔）` | 922,580,645 | `B0A6888C925586416E09981AAF84123D37246AF8CCA0105D919C07734212E641` |
| `[tree] SHARED/root（3 檔）` | 6,464,485 | `F1A2B32AA1E003D0D5C196E36CCBDD5124354DB8008D7131CFCFCB2C1764BA45` |

`[tree]` 列是散檔命名空間：把每個檔的「相對路徑（小寫）／大小／SHA-256」排序後串接再取 SHA-256。

## 2. 格式矩陣（命名空間 × 容器 × 格式 × 狀態）

共用 Sprite 分片（Sprite00–15）在這裡合併成一列，逐分片的數字在 `matrix.json`；地圖目錄合併為 `map/*`、`LiTo/LTmap/*`，逐目錄見 §4.4。

| 命名空間 | 容器 | 格式 | 狀態 | 成員數 | bytes | 字面引用 | 判定（成員數） |
|---|---|---|---|---:|---:|---:|---|
| LAUNCHER | `launcher` | loose_other | no_decoder | 1 | 1,406,534 | 1 | `not_inspected` 1 |
| NATIVE_LITO | `LiTo/LTicon` | ico_list | selected_members_only | 1 | 5 | 1 | `not_inspected` 1 |
| NATIVE_LITO | `LiTo/LTicon` | lbt | supported_container_ok | 226 | 274,253 | 0 | `tbt.ok` 226 |
| NATIVE_LITO | `LiTo/LTsprite` | spr | rejected_by_decoder_gate | 26 | 12,668,570 | 0 | `spr.unlisted_empty_frame_marker` 26 |
| NATIVE_LITO | `LiTo/LTsprite` | spr | supported_container_ok | 443 | 50,335,542 | 0 | `spr.container_ok_rgb555` 443 |
| NATIVE_LITO | `LiTo/LTsurf` | img | rejected_by_decoder_gate | 2 | 4,718,608 | 0 | `img.decode_budget` 2 |
| NATIVE_LITO | `LiTo/LTsurf` | img | supported_container_ok | 1,611 | 84,977,494 | 1 | `img.ok_flag0` 967；`img.ok_flag1` 644 |
| NATIVE_LITO | `LiTo/LTsurf` | spr | rejected_by_decoder_gate | 3 | 1,634,767 | 0 | `spr.unlisted_empty_frame_marker` 3 |
| NATIVE_LITO | `LiTo/LTsurf` | spr | supported_container_ok | 7 | 1,458,010 | 0 | `spr.container_ok_rgb555` 7 |
| NATIVE_LITO | `LiTo/LTtext` | html | selected_members_only | 256 | 129,502 | 37 | `text.ascii_head` 251；`text.non_ascii_head` 4；`text.utf8_bom` 1 |
| NATIVE_LITO | `LiTo/LTtext` | list_spr_text | selected_members_only | 1 | 39,135 | 1 | `text.ascii_head` 1 |
| NATIVE_LITO | `LiTo/LTtext` | tbl | selected_members_only | 88 | 88,432 | 4 | `text.non_ascii_head` 78；`text.ascii_head` 10 |
| NATIVE_LITO | `LiTo/LTtext` | text_misc | no_decoder | 8 | 35,484 | 0 | `text.ascii_head` 6；`text.non_ascii_head` 2 |
| NATIVE_LITO | `LiTo/LTtile` | junk | not_game_resource | 2 | 841,243 | 0 | `not_inspected` 2 |
| NATIVE_LITO | `LiTo/LTtile` | til | rejected_by_decoder_gate | 14 | 13,420 | 0 | `til.empty_slots_not_terminal_suffix` 13；`til.limit_tile_count` 1 |
| NATIVE_LITO | `LiTo/LTtile` | til | supported_container_ok | 1,191 | 138,266,003 | 0 | `til.container_ok` 1,179；`til.container_ok_empty_tail` 12 |
| NATIVE_LITO | `LiTo/LTtile` | unknown | no_decoder | 3 | 9,364,640 | 0 | `not_inspected` 3 |
| NATIVE_LITO/LTSurf | `LiTo/LTSurf` | img | supported_container_ok | 1 | 614,408 | 0 | `img.ok_flag0` 1 |
| NATIVE_LITO/LTmap | `LiTo/LTmap/*` | loose_other | no_decoder | 2 | 36 | 0 | `file.zero_size` 1；`not_inspected` 1 |
| NATIVE_LITO/LTmap | `LiTo/LTmap/*` | s32 | supported_container_ok | 495 | 52,285,859 | 0 | `s32.ok_layers1to8` 495 |
| NATIVE_LITO/LTsound | `LiTo/LTsound` | mp3 | no_decoder | 18 | 33,976,228 | 0 | `mp3.id3` 17；`mp3.frame_sync` 1 |
| NATIVE_LITO/LTsound | `LiTo/LTsound` | wav | supported_container_ok | 31 | 6,667,640 | 1 | `wav.ok` 31 |
| NATIVE_LITO/loose | `LiTo` | bmp | no_decoder | 17 | 669,472 | 0 | `bmp.signature_ok` 17 |
| NATIVE_LITO/loose | `LiTo` | junk | not_game_resource | 1 | 33,280 | 0 | `not_inspected` 1 |
| NATIVE_LITO/loose | `LiTo` | loose_other | no_decoder | 7 | 260 | 0 | `not_inspected` 7 |
| SHARED | `Sprite` | ico_list | selected_members_only | 1 | 6 | 1 | `not_inspected` 1 |
| SHARED | `Sprite` | img | rejected_by_decoder_gate | 5 | 12,759,340 | 0 | `img.decode_budget` 5 |
| SHARED | `Sprite` | img | supported_container_ok | 1,857 | 85,226,776 | 0 | `img.ok_flag0` 973；`img.ok_flag1` 884 |
| SHARED | `Sprite` | junk | not_game_resource | 1 | 16,640 | 0 | `not_inspected` 1 |
| SHARED | `Sprite` | png | no_decoder | 803 | 9,006,294 | 0 | `png.signature_ok` 802；`png.bad_signature_89504e58` 1 |
| SHARED | `Sprite` | spr | rejected_by_decoder_gate | 1,392 | 139,632,115 | 0 | `spr.unlisted_empty_frame_marker` 1,386；`spr.reversed_frame_bounds_with_blocks` 5；`spr.zero_frame_count` 1 |
| SHARED | `Sprite` | spr | supported_container_ok | 85,952 | 1,648,949,827 | 23 | `spr.container_ok_palette` 83,684；`spr.container_ok_rgb555` 2,268 |
| SHARED | `Sprite` | tbl | selected_members_only | 3 | 30,156 | 0 | `not_inspected` 3 |
| SHARED | `Sprite` | tbt | supported_container_ok | 2,334 | 2,603,435 | 2 | `tbt.ok` 2,334 |
| SHARED | `Sprite` | unknown | no_decoder | 8 | 324,959 | 2 | `not_inspected` 8 |
| SHARED | `Sprite00–15` | img | rejected_by_decoder_gate | 2 | 5,160,316 | 0 | `img.decode_budget` 2 |
| SHARED | `Sprite00–15` | img | supported_container_ok | 5,715 | 112,033,488 | 10 | `img.ok_flag0` 3,776；`img.ok_flag1` 1,939 |
| SHARED | `Sprite00–15` | junk | not_game_resource | 3 | 157,494 | 0 | `not_inspected` 3 |
| SHARED | `Sprite00–15` | png | no_decoder | 1,435 | 278,158,123 | 0 | `png.signature_ok` 1,417；`png.bad_signature_89504e58` 18 |
| SHARED | `Sprite00–15` | spr | rejected_by_decoder_gate | 10,877 | 1,150,760,112 | 0 | `spr.unlisted_empty_frame_marker` 10,806；`spr.reversed_frame_bounds_with_blocks` 40；`spr.limit_block_count` 27；`spr.truncated` 3；`spr.zero_frame_count` 1 |
| SHARED | `Sprite00–15` | spr | supported_container_ok | 147,983 | 5,041,157,952 | 6 | `spr.container_ok_palette` 108,372；`spr.container_ok_rgb555` 39,611 |
| SHARED | `Sprite00–15` | tbt | rejected_by_decoder_gate | 6 | 4,932 | 0 | `tbt.truncated` 6 |
| SHARED | `Sprite00–15` | tbt | supported_container_ok | 3,116 | 3,968,644 | 1 | `tbt.ok` 3,116 |
| SHARED | `Sprite00–15` | unknown | no_decoder | 3 | 52,861 | 0 | `not_inspected` 3 |
| SHARED | `Sprite00–15` | ico_list | selected_members_only | 1 | 7 | 1 | `not_inspected` 1 |
| SHARED | `Text` | html | selected_members_only | 37,130 | 15,003,719 | 4 | `text.ascii_head` 34,119；`text.non_ascii_head` 2,829；`text.utf16le_bom` 176；`text.utf8_bom` 6 |
| SHARED | `Text` | junk | not_game_resource | 120 | 172,026 | 0 | `text.ascii_head` 111；`text.non_ascii_head` 9 |
| SHARED | `Text` | list_spr_text | selected_members_only | 1 | 4,067,417 | 1 | `text.ascii_head` 1 |
| SHARED | `Text` | tbl | selected_members_only | 217 | 1,930,942 | 6 | `text.non_ascii_head` 133；`text.ascii_head` 84 |
| SHARED | `Text` | text_misc | no_decoder | 13 | 222,746 | 0 | `text.ascii_head` 7；`text.non_ascii_head` 6 |
| SHARED | `Text` | unknown | no_decoder | 17 | 4,870,854 | 0 | `text.ascii_head` 16；`text.non_ascii_head` 1 |
| SHARED | `Tile` | spr | supported_container_ok | 8 | 131,844 | 0 | `spr.container_ok_rgb555` 8 |
| SHARED | `Tile` | til | rejected_by_decoder_gate | 20 | 19,613 | 0 | `til.empty_slots_not_terminal_suffix` 19；`til.limit_tile_count` 1 |
| SHARED | `Tile` | til | supported_container_ok | 5,344 | 590,520,943 | 1 | `til.container_ok` 5,328；`til.container_ok_empty_tail` 16 |
| SHARED | `Tile` | unknown | no_decoder | 9 | 1,348,986 | 0 | `not_inspected` 9 |
| SHARED | `Tile` | xml_tile | no_decoder | 65 | 1,299,582 | 1 | `magic.583f786d` 65 |
| SHARED/Font | `Font` | fnt | no_decoder | 11 | 519,940 | 0 | `not_inspected` 11 |
| SHARED/Sound | `Sound` | junk | not_game_resource | 1 | 2,656 | 0 | `not_inspected` 1 |
| SHARED/Sound | `Sound` | mid | no_decoder | 1 | 25,916 | 0 | `mid.mthd` 1 |
| SHARED/Sound | `Sound` | mp3 | no_decoder | 163 | 278,616,608 | 0 | `mp3.id3` 105；`mp3.frame_sync` 58 |
| SHARED/Sound | `Sound` | wav | rejected_by_decoder_gate | 617 | 30,407,649 | 0 | `wav.unsupported_encoding_17` 351；`wav.fmt_size_invalid` 266 |
| SHARED/Sound | `Sound` | wav | supported_container_ok | 4,949 | 426,294,472 | 2 | `wav.ok` 4,947；`wav.ok_terminal_unpadded_data` 2 |
| SHARED/Sound | `Sound/voice` | mp3 | no_decoder | 1 | 732,335 | 0 | `mp3.id3` 1 |
| SHARED/map | `map/*` | junk | not_game_resource | 19 | 14,864 | 0 | `not_inspected` 19 |
| SHARED/map | `map/*` | loose_other | no_decoder | 24 | 360,448 | 0 | `not_inspected` 22；`file.zero_size` 2 |
| SHARED/map | `map/*` | s32 | supported_container_ok | 7,807 | 898,035,022 | 2 | `s32.ok_layers1to8` 6,821；`s32.ok_layers1to8_unknown_trailing` 512；`s32.ok_layers1to7` 474 |
| SHARED/map | `map/*` | seg | no_decoder | 330 | 24,170,311 | 0 | `not_inspected` 330 |
| SHARED/root | `(root)` | loose_other | no_decoder | 3 | 6,464,485 | 2 | `not_inspected` 3 |

### 2.1 依狀態加總

| 狀態 | 成員數 |
|---|---:|
| no_decoder | 2,942 |
| not_game_resource | 147 |
| rejected_by_decoder_gate | 12,964 |
| selected_members_only | 37,699 |
| supported_container_ok | 269,070 |
| 合計 | 322,822 |

### 2.2 被解碼器關卡拒絕的原因碼

| 格式 | 原因碼 | 成員數 |
|---|---|---:|
| img | `img.decode_budget` | 9 |
| spr | `spr.unlisted_empty_frame_marker` | 12,221 |
| spr | `spr.reversed_frame_bounds_with_blocks` | 45 |
| spr | `spr.limit_block_count` | 27 |
| spr | `spr.truncated` | 3 |
| spr | `spr.zero_frame_count` | 2 |
| tbt | `tbt.truncated` | 6 |
| til | `til.empty_slots_not_terminal_suffix` | 32 |
| til | `til.limit_tile_count` | 2 |
| wav | `wav.unsupported_encoding_17` | 351 |
| wav | `wav.fmt_size_invalid` | 266 |

逐成員名稱見 `docs/resource-matrix/exceptions.tsv`。

- `til.container_ok_empty_tail` 列在「通過」，但只有 `decode_classic_til_selected_with_empty_tail` 這個明確選用的入口會接受；一般的 `decode_classic_til` 會拒絕。
- `til.limit_tile_count` 的兩個成員是 `list.til`（4～5 bytes 的文字），不是地磚。
- `spr.unlisted_empty_frame_marker`：影格宣告的邊界是反的（right < left 或 bottom < top）而且沒有區塊，也就是「不畫的空影格」；`sprite.rs` 的 `empty_sentinel` 目前只認得 5 組數值，其餘一律當成格式錯誤。資源裡第一個遇到的未列數值共有 **256 種**，前 12 種如下（全部在 `matrix.json` 的 `spr_unlisted_empty_frame_markers`）：

| left | top | right | bottom | unknown1 | unknown2 | 成員數 |
|---:|---:|---:|---:|---|---|---:|
| 192 | 120 | -128 | -120 | 0000 | 0000 | 3,998 |
| 352 | 240 | -288 | -240 | FFFF | 0000 | 2,050 |
| 192 | 96 | -128 | -144 | 0000 | 0000 | 562 |
| 432 | 300 | -368 | -300 | FFFF | 0000 | 490 |
| 192 | 96 | -128 | -144 | FFFF | 0000 | 409 |
| 544 | 312 | -480 | -456 | FFFF | 0000 | 400 |
| 352 | 240 | -288 | -240 | 0000 | 0000 | 332 |
| 232 | 150 | -168 | -150 | FE00 | 0005 | 328 |
| 186 | 113 | -134 | -127 | FFFF | 0000 | 312 |
| 352 | 192 | -288 | -288 | FFFF | 0000 | 257 |
| 352 | 240 | -288 | -240 | FE00 | 0005 | 240 |
| 180 | 116 | -140 | -124 | 0000 | 0000 | 208 |

## 3. 格式支援表

| 格式 | 支援方式 | Rust 模組（`src/modern_lito/`） | 說明 |
|---|---|---|---|
| spr | decoder | `sprite.rs` | SPR 影格圖（RGB555 或調色盤，24×24 RLE 區塊） |
| til | decoder | `tile.rs`、`tile_reader.rs` | TIL 地磚（型別 0–3、6–9、16–19、22–23、34–35） |
| tbt | decoder | `inventory_icons.rs` | TBT 圖示（列 span、RGB555） |
| lbt | decoder | `native_inventory_icons.rs`、`inventory_icons.rs` | LBT 圖示（與 TBT 同一個解碼器） |
| img | decoder | `native_mode_menu_skin.rs`、`battle_hud.rs`、`native_skill_layers.rs` | IMG 介面圖（8 byte 表頭＋RGB555） |
| wav | decoder | `audio.rs` | RIFF/WAVE PCM8／PCM16 單聲道或立體聲 |
| s32 | decoder | `map.rs`、`world_map.rs` | S32 地圖區塊（Layer1–8） |
| list_spr_text | selected | `sprite_actions.rs`、`shared_action_catalog.rs` | list.spr 動作目錄（文字） |
| ico_list | selected | `inventory_icons.rs` | list.ico 圖示範圍表（文字） |
| tbl | selected | `descriptions.rs`、`native_inventory_names.rs` | TBL 文字表 |
| html | selected | `native_npc_document.rs` | NPC 對話 HTML |
| text_misc | none | — | 其他文字檔（txt／def／mcf） |
| png | none | — | PNG 圖（png crate 只用在輸出截圖，沒有資源消費端） |
| xml_tile | none | — | Tile.pak 內的 XML（首 4 bytes 為 58 3F 78 6D，非明文 `<?xm`） |
| mp3 | none | — | MP3 音樂（audio.rs 明確拒絕） |
| mid | none | — | MIDI（audio.rs 明確拒絕） |
| seg | none | — | SEG 地圖區塊（map.rs 明確不解） |
| bmp | none | — | BMP（原登入器外框圖） |
| fnt | none | — | FNT 點陣字型 |
| unknown | none | — | 未知格式（副檔名不在表內） |
| junk | junk | — | 非遊戲資源（開發殘留／版控／壓縮檔） |
| loose_other | none | — | 其他散檔 |

## 4. 命名空間事實

### 4.1 索引層

| 容器 | 記錄數 | 不重複名稱 | 同名重複記錄 | 名稱滿 20 bytes（疑似截斷） | 非常規字元名稱 | 分片路由相符 | 分片路由不符 |
|---|---:|---:|---:|---:|---:|---:|---:|
| `Sprite` | 92,356 | 92,356 | 0 | 0 | 0 | — | — |
| `Sprite00` | 10,207 | 10,207 | 0 | 0 | 0 | 10,207 | 0 |
| `Sprite01` | 10,232 | 10,232 | 0 | 0 | 0 | 10,232 | 0 |
| `Sprite02` | 10,320 | 10,320 | 0 | 0 | 0 | 10,320 | 0 |
| `Sprite03` | 10,458 | 10,458 | 0 | 0 | 0 | 10,458 | 0 |
| `Sprite04` | 10,593 | 10,593 | 0 | 0 | 0 | 10,593 | 0 |
| `Sprite05` | 10,700 | 10,700 | 0 | 0 | 0 | 10,700 | 0 |
| `Sprite06` | 10,844 | 10,844 | 0 | 0 | 0 | 10,844 | 0 |
| `Sprite07` | 10,949 | 10,949 | 0 | 0 | 0 | 10,949 | 0 |
| `Sprite08` | 10,972 | 10,972 | 0 | 0 | 0 | 10,972 | 0 |
| `Sprite09` | 10,970 | 10,970 | 0 | 0 | 0 | 10,970 | 0 |
| `Sprite10` | 10,862 | 10,862 | 0 | 0 | 0 | 10,862 | 0 |
| `Sprite11` | 10,716 | 10,716 | 0 | 0 | 0 | 10,716 | 0 |
| `Sprite12` | 10,522 | 10,522 | 0 | 0 | 0 | 10,522 | 0 |
| `Sprite13` | 10,357 | 10,357 | 0 | 0 | 0 | 10,357 | 0 |
| `Sprite14` | 10,255 | 10,255 | 0 | 0 | 0 | 10,255 | 0 |
| `Sprite15` | 10,184 | 10,184 | 0 | 0 | 0 | 10,184 | 0 |
| `Tile` | 5,446 | 5,446 | 0 | 0 | 0 | — | — |
| `Text` | 37,498 | 28,362 | 9,246 | 6,261 | 3 | — | — |
| `LiTo/LTicon` | 227 | 227 | 0 | 0 | 0 | — | — |
| `LiTo/LTsprite` | 469 | 469 | 0 | 0 | 0 | — | — |
| `LiTo/LTsurf` | 1,623 | 1,623 | 0 | 0 | 0 | — | — |
| `LiTo/LTtext` | 353 | 353 | 0 | 0 | 0 | — | — |
| `LiTo/LTtile` | 1,210 | 1,210 | 0 | 0 | 0 | — | — |

- 索引是 old28（4 bytes 筆數＋每筆 `offset u32｜name 20 bytes｜size u32`）；`Text`、`LTtext` 的索引本體與成員內容另有 L1 保護（`resource_cipher.rs`）。
- 「分片路由」是檔名 ASCII bytes 加總後取低 4 bits（`inventory_icons.rs` 引用的 `sub_43FFC0`）；這裡對 Sprite00–15 的**每一個**成員都算一次。
- 名稱欄位只有 20 bytes：剛好 20 bytes 且副檔名是 `.html`／`.spr` 前綴（`.htm`、`.ht`、`.h`、`.`、`.sp`）的成員，矩陣依截斷前的格式歸類並加旗標 `name_truncated_20`。查這些成員要用截斷後的名稱。
- `Text` 有同名重複記錄：多個較長的原始檔名被截成同一個 20 bytes 名稱（例如 `mercenaryarrange-j.h`）。`resources.rs` 的 `SharedArchive::entry` 對這些名稱回 `AmbiguousEntry`，不會自動挑一筆。

### 4.2 跨封存同名

| 比較 | 同名成員 | 其中內容相同 |
|---|---:|---:|
| `Sprite` 對 `Sprite00–15` | 3,119 | 188 |
| `LTsprite` 對共用 Sprite 全家 | 295 | 0 |
| `LTsurf` 對共用 Sprite 全家 | 714 | 112 |
| `LTtile` 對 `Tile` | 1,205 | 0 |
| `LTtext` 對 `Text` | 12 | 0 |

共用 Sprite 全家（17 個封存）裡出現在一個以上封存的名稱：

| 副檔名 | 內容 | 名稱數 |
|---|---|---:|
| .ico | different | 1 |
| .img | different | 46 |
| .img | identical | 7 |
| .scc | different | 1 |
| .spr | different | 2,817 |
| .spr | identical | 103 |
| .tbt | different | 66 |
| .tbt | identical | 78 |

同名不同內容代表「讀哪一個封存」會改變畫面；`actor_assets.rs` 的規則是必須指名封存或全家唯一，不挑贏家。`Text` 的比較用的是保護狀態下的原始 bytes。

### 4.3 TIL 區塊型別（每個不重複區塊偏移算一次）

| 容器 | 型別 0 | 型別 1 | 型別 2 | 型別 3 | 型別 6 | 型別 7 | 型別 8 | 型別 9 | 型別 16 | 型別 17 | 型別 18 | 型別 19 | 型別 22 | 型別 23 | 型別 34 | 型別 35 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `LiTo/LTtile` | 7,301 | 7,302 | 123,285 | 123,253 | 18,836 | 19,166 | 1 | 1 | 0 | 0 | 840 | 851 | 5 | 5 | 373 | 379 |
| `Tile` | 73,818 | 73,808 | 504,481 | 504,495 | 89,459 | 90,692 | 2 | 2 | 9 | 9 | 9,696 | 9,700 | 22 | 21 | 3,736 | 3,737 |

只統計通過偏移表檢查的 TIL；型別取自區塊第一個 byte。

### 4.4 地圖目錄

- `map/`（SHARED）：717 個地圖目錄，S32 7,807 個、SEG 330 個。逐目錄數字在 `matrix.json` 的 `map_directories`。
- 只有 SEG、沒有 S32 的共用地圖目錄：1 個（`map/18`）。
- `LiTo/LTmap/`（NATIVE_LITO）：62 個地圖目錄，以 `parse_native_s32` 的 portal 尾碼規則檢查：

| 目錄 | S32 數 | bytes | 判定 |
|---|---:|---:|---|
| `LiTo/LTmap/16384` | 1 | 59,284 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/16896` | 1 | 62,728 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/17408` | 1 | 56,469 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/500` | 18 | 2,289,125 | `s32.ok_layers1to8` 18 |
| `LiTo/LTmap/501` | 1 | 61,595 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/502` | 1 | 60,893 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/503` | 1 | 64,418 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/504` | 7 | 780,293 | `s32.ok_layers1to8` 7 |
| `LiTo/LTmap/505` | 1 | 71,870 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/506` | 6 | 525,204 | `s32.ok_layers1to8` 6 |
| `LiTo/LTmap/507` | 1 | 66,846 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/508` | 1 | 87,415 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/509` | 1 | 86,253 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/510` | 9 | 773,884 | `s32.ok_layers1to8` 9 |
| `LiTo/LTmap/512` | 9 | 796,977 | `s32.ok_layers1to8` 9 |
| `LiTo/LTmap/513` | 14 | 1,576,593 | `s32.ok_layers1to8` 14 |
| `LiTo/LTmap/514` | 1 | 76,380 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/516` | 1 | 134,052 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/517` | 9 | 707,415 | `s32.ok_layers1to8` 9 |
| `LiTo/LTmap/518` | 9 | 1,092,801 | `s32.ok_layers1to8` 9 |
| `LiTo/LTmap/519` | 8 | 1,043,627 | `s32.ok_layers1to8` 8 |
| `LiTo/LTmap/520` | 8 | 1,043,627 | `s32.ok_layers1to8` 8 |
| `LiTo/LTmap/521` | 6 | 486,478 | `s32.ok_layers1to8` 6 |
| `LiTo/LTmap/522` | 6 | 486,506 | `s32.ok_layers1to8` 6 |
| `LiTo/LTmap/524` | 8 | 484,761 | `s32.ok_layers1to8` 8 |
| `LiTo/LTmap/525` | 11 | 906,688 | `s32.ok_layers1to8` 11 |
| `LiTo/LTmap/526` | 4 | 281,158 | `s32.ok_layers1to8` 4 |
| `LiTo/LTmap/527` | 18 | 2,287,462 | `s32.ok_layers1to8` 18 |
| `LiTo/LTmap/528` | 5 | 324,421 | `s32.ok_layers1to8` 5 |
| `LiTo/LTmap/529` | 4 | 287,228 | `s32.ok_layers1to8` 4 |
| `LiTo/LTmap/530` | 9 | 578,106 | `s32.ok_layers1to8` 9 |
| `LiTo/LTmap/531` | 5 | 305,105 | `s32.ok_layers1to8` 5 |
| `LiTo/LTmap/532` | 13 | 1,414,213 | `s32.ok_layers1to8` 13 |
| `LiTo/LTmap/534` | 16 | 2,265,823 | `s32.ok_layers1to8` 16 |
| `LiTo/LTmap/535` | 25 | 3,656,097 | `s32.ok_layers1to8` 25 |
| `LiTo/LTmap/540` | 13 | 1,215,525 | `s32.ok_layers1to8` 13 |
| `LiTo/LTmap/541` | 13 | 1,215,525 | `s32.ok_layers1to8` 13 |
| `LiTo/LTmap/542` | 13 | 1,215,525 | `s32.ok_layers1to8` 13 |
| `LiTo/LTmap/543` | 6 | 393,721 | `s32.ok_layers1to8` 6 |
| `LiTo/LTmap/545` | 25 | 2,642,749 | `s32.ok_layers1to8` 25 |
| `LiTo/LTmap/546` | 24 | 1,822,888 | `s32.ok_layers1to8` 24 |
| `LiTo/LTmap/547` | 12 | 1,437,842 | `s32.ok_layers1to8` 12 |
| `LiTo/LTmap/548` | 13 | 1,532,135 | `s32.ok_layers1to8` 13 |
| `LiTo/LTmap/549` | 16 | 2,007,512 | `s32.ok_layers1to8` 16 |
| `LiTo/LTmap/550` | 16 | 1,999,252 | `s32.ok_layers1to8` 16 |
| `LiTo/LTmap/551` | 3 | 208,719 | `s32.ok_layers1to8` 3 |
| `LiTo/LTmap/552` | 4 | 294,517 | `s32.ok_layers1to8` 4 |
| `LiTo/LTmap/553` | 15 | 1,115,549 | `s32.ok_layers1to8` 15 |
| `LiTo/LTmap/554` | 6 | 781,517 | `s32.ok_layers1to8` 6 |
| `LiTo/LTmap/555` | 1 | 101,356 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/560` | 11 | 1,003,580 | `s32.ok_layers1to8` 11 |
| `LiTo/LTmap/562` | 19 | 1,970,067 | `s32.ok_layers1to8` 19 |
| `LiTo/LTmap/563` | 19 | 1,970,067 | `s32.ok_layers1to8` 19 |
| `LiTo/LTmap/564` | 1 | 128,028 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/565` | 4 | 555,057 | `s32.ok_layers1to8` 4 |
| `LiTo/LTmap/566` | 4 | 625,287 | `s32.ok_layers1to8` 4 |
| `LiTo/LTmap/567` | 1 | 108,958 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/568` | 1 | 104,948 | `s32.ok_layers1to8` 1 |
| `LiTo/LTmap/569` | 4 | 638,852 | `s32.ok_layers1to8` 4 |
| `LiTo/LTmap/570` | 4 | 627,810 | `s32.ok_layers1to8` 4 |
| `LiTo/LTmap/573` | 4 | 655,523 | `s32.ok_layers1to8` 4 |
| `LiTo/LTmap/574` | 4 | 631,555 | `s32.ok_layers1to8` 4 |

### 4.5 空目錄與未納入雜湊的檔案

- 空目錄：`LiTo/LTtext/`、`Sprite/`、`Surf/`、`Tile/`、`icon/`、`skins/default/`、`text/`。
- 下列檔案不是資源命名空間的一部分（執行檔、設定、紀錄、工具包、登入器），只列數量、不納入資源集合雜湊。根目錄只納入 `shade.dat`、`Emblem769.dat`、`TW13081901.txt`，`launcher/` 只納入 `TW13081901.pak`：

| 位置 | 副檔名 | 檔數 |
|---|---|---:|
| `(root)` | .asi | 1 |
| `(root)` | .bin | 3 |
| `(root)` | .cfg | 1 |
| `(root)` | .dat | 5 |
| `(root)` | .des | 1 |
| `(root)` | .dll | 14 |
| `(root)` | .eat | 2 |
| `(root)` | .exe | 2 |
| `(root)` | .ini | 1 |
| `(root)` | .loc | 6 |
| `(root)` | .log | 1 |
| `(root)` | .sys | 5 |
| `(root)` | .txt | 2 |
| `(root)` | .vxd | 2 |
| `Capture` | .txt | 1 |
| `Microsoft.VC90.CRT` | .dll | 3 |
| `Microsoft.VC90.CRT` | .manifest | 1 |
| `launcher` | .dll | 2 |
| `launcher` | .html | 2 |
| `launcher` | .ini | 3 |
| `launcher` | .jpg | 1 |
| `launcher` | .json | 1 |
| `launcher` | .updated | 1 |
| `log` | .txt | 18 |
| `skins` | .html | 1 |
| `skins` | .jpg | 1 |
| `工具包` | .bat | 1 |
| `工具包` | .cfg | 2 |
| `工具包` | .db | 1 |
| `工具包` | .exe | 3 |
| `工具包` | .jpe | 1 |
| `工具包` | .rar | 5 |
| `工具包` | .txt | 1 |
| `工具包` | .vbs | 3 |

## 5. 重跑與核對

```powershell
python -X utf8 維護工具/resource-matrix/resource_matrix.py           # 重產
python -X utf8 維護工具/resource-matrix/resource_matrix.py --check   # 重算並與入庫版本比對，不同則 exit 1
python -X utf8 維護工具/resource-matrix/resource_matrix.py --self-test
```

`--check` 失敗代表客戶端資源、`src` 的字面引用或工具規則其中之一變了；重產後在 PR 說明差異來源。
