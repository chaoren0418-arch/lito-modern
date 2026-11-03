"""Archive updater and add current summon51 source/metadata scope."""
from pathlib import Path
from datetime import datetime,timezone
from native_E149_recovery_contract import R,sha,encoded,require
T=R/'維護工具';V=R/'驗證紀錄/current';p=T/'update-master-report-E149-current.py';s=p.read_text('utf-8-sig');require("s51_ref=w.get('current_E149_summon51_development')" not in s,'stage already present');out=V/'master-report-summon51-updater-source'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.py').write_bytes(p.read_bytes());(out/'before.json').write_bytes(encoded(dict(path=str(p),sha256=sha(p),archive=str(out/'before.py'),archive_sha256=sha(out/'before.py'))))
needle=" date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')";require(s.count(needle)==1,'date marker differs')
block=''' s51_ref=w.get('current_E149_summon51_development')
 if s51_ref:
  require(sha(s51_ref['qualification'])==s51_ref['qualification_sha256'],'summon51 qualification differs');s51=read(s51_ref['qualification']);check_pins(s51['source_pins']);require(s51['compiler_exit_codes']==[0]*18 and s51['tests_executed']==0 and s51['summon_selection_count']==31 and s51['summon_pending_after_cast_action_bounded_seconds']==2,'summon51 scope differs')
  area2230_note=area2230_note.replace('**本輪最新：#50冰矛圍籬與無音效class4候選已補**','**#50無音效class4來源保留**').replace('**G06 flight接續補強**','**前輪G06 flight接續補強**').replace('來源入口識別63／剩117。','該輪來源入口識別63／剩117。')
  area2230_note=('**本輪最新：#51召喚術選項與controller候選已接**。C14 `14 06 02`＋selection u32LE共七bytes；正式31個選項經summon／summonSpecial兩helper與七欄constructor展開，特殊容量adjustment7；名稱只作唯讀NPC SQL基礎label。controlAbility5時使用選項，否則由server依角色等級決定。Native Window新增獨立非modal下拉選單，僅顯式選取授權；SummonTarget核對完整inventory owner／revision、connection／generation／actor incarnation／corrections／control interruption／map。controller獨立cast_summon沿共用互斥、body完成、reuse1000基值與pending；actor／ground／item不混用。MP30／HP30／三寶石40318、lawful-10、容量／模板及1800秒皆server權威，2510／755與action不造companion ACK。'+link('正式分支、31選項與來源編譯',s51_ref['qualification'])+'；'+link('入口與邊界',T/'native-E149-summon51.md')+'。\\n\\n**跨poll拒絕修正**：353地圖／743等級或319容量拒絕可能在action之後。原初版summon observer雖已scoped，但共用controller在CastAction清pending，仍會漏下一poll；由來源檢查發現，原候選與18metadata（'+s51['original_metadata_observed_tool_chunk']+' actual exit0）保留，沒有執行反例。修正版只在召喚CastAction保留關聯至原兩秒觀測窗、權威拒絕或退役，其他技能終止規則不變，不延長deadline。四Runtime差分仍33檔／365module；controller五nested與window一rental nested重建且原六target不變。修正版兩版library／library-test／七skill整合18metadata退出0，原'+s51['observed_original_tool_chunk']+' actual exit0。新12純QA＋前214共226案全未跑；來源入口64／剩116，不是完整library inventory或GUI／玩法通過。'+link('原缺口與修正範圍',T/'native-E149-summon51-hold.md')+'。完整companion指令、控制能力／模板／材料、自然1800秒與owner／傳送退役仍待GAME／GUI。原三javap皆退出0；首次外層reader誤用CP950退出1，raw保留後改UTF-8讀同輸出，未重跑。正常1049輸入、所有PE、正式JAR、registry及G06 binding／16案640次不變；未套用／選入／連結或執行PE／Java／SQL／測試／GUI／播放。\\n\\n'+area2230_note)
  catalog_note=catalog_note.replace('本輪候選63，尚117','本輪候選64，尚116').replace('31個非技能召喚helper保留opaque','原31個召喚helper曾保留opaque，本輪已由兩helper Code展開31選項，原catalog歷史檔不改寫').replace('最新識別集合見本輪freeze50 qualification的current_source_coverage','最新識別集合見本輪summon51 qualification的current_source_coverage')
'''
s=s.replace(needle,block+needle)
needle=' # The prior report is an immutable bridge to every historical source/result.';require(s.count(needle)==1,'footer marker differs')
block=''' if s51_ref:
  body=body.replace('最新117個請求缺口','最新116個請求缺口').replace('其餘117個入口','其餘116個入口').replace('當版下一組#51／#55字面定義','當版下一組#55／#56字面定義')
  body=body.replace('#51正式summonSpellRequest另有selection欄，接續獨立召喚選項UI／Owner／controller，不能當actor ID或selector-only。','#51七byte selection、31選項UI與Owner／controller及原兩秒窗跨poll拒絕已接候選；完整companion與自然1800秒仍待正式GAME，不把actor／effect當召喚成功。')
  body=body.replace('驗T鍵與#50／','驗T鍵與#51／#50／').replace('最新33檔（再含50／無音效class4與flight接續）需另建當版映像','最新33檔（再含51選項UI／controller與跨poll拒絕）需另建當版映像')
'''
s=s.replace(needle,block+needle);p.write_text(s,encoding='utf-8',newline='\n');print(out)
