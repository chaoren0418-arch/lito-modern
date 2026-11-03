"""Archive the current report updater, then add the qualified freeze50 stage."""
from pathlib import Path
from datetime import datetime,timezone
from native_E149_recovery_contract import R,read,sha,encoded,require
T=R/'維護工具';V=R/'驗證紀錄/current';p=T/'update-master-report-E149-current.py';s=p.read_text('utf-8-sig')
require("f50_ref=w.get('current_E149_freeze50_development')" not in s,'updater already includes freeze50')
out=V/'master-report-freeze50-updater-source'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.py').write_bytes(p.read_bytes());(out/'before.json').write_bytes(encoded(dict(path=str(p),sha256=sha(p),archive=str(out/'before.py'),archive_sha256=sha(out/'before.py'))))
needle=" date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')";require(s.count(needle)==1,'date anchor differs')
block=''' f50_ref=w.get('current_E149_freeze50_development')
 if f50_ref:
  require(sha(f50_ref['qualification'])==f50_ref['qualification_sha256'],'freeze50 qualification differs');f50=read(f50_ref['qualification']);check_pins(f50['source_pins']);require(f50['compiler_exit_codes']==[0]*18 and f50['tests_executed']==0 and f50['current_source_coverage']['recognized_count']==63 and f50['freeze50_silent_flight_and_secondary'],'freeze50 source scope differs')
  area2230_note=area2230_note.replace('**本輪最新：#46烈炎術／#47弱化術與class1接續候選已補**','**#46／#47與class1接續來源保留**').replace('**G06接續補強**','**前輪G06 class1接續補強**').replace('來源入口識別62／剩118，非兩技能或G06完成','該輪來源入口識別62／剩118，非兩技能或G06完成')
  area2230_note=('**本輪最新：#50冰矛圍籬與無音效class4候選已補**。C14 `14 06 01`＋角色u32 ID／x/y共11bytes，其他可見活體／六格、reuse500／client clock1000基值及實際選單；保留Target／Owner與controller守衛。正式FreezeAttackSpell與實際Bootstrap caller已靜態核對，S23 wire0依raw756走class4→219；756原八圖折四bank、source3/0/1/2各2ticks，219原十一frame與terminal source0停留41ms。兩者無Sound／Effect／Hit；loader與clock新增嚴格無音效分支、optional group／空SoundBank，沒有97 fallback或Sound0。原MP100、16秒冰凍／機率／傷害及getter覆寫由server決定，視覺不造狀態。756 arrival僅診斷；45／46／50完整hurt相對gfx及原guard仍待。'+link('本輪正式分支、來源與編譯',f50_ref['qualification'])+'；'+link('範圍與後續',T/'native-E149-freeze50.md')+'。\\n\\n**G06 flight接續補強**：活動class4/5在raw ring淘汰後仍保留seen，原記錄再進入不建立第二條flight；重建資源準備失敗退役該flight、只記一次omitted，避免永久保留audio history。四Runtime差分仍33檔（十九Runtime／十一Rust QA／三Java）、365module與原六nested；兩版library／library-test／七skill整合18metadata退出0，原'+f50['observed_original_tool_chunk']+' actual exit0。新12（11純＋1原資源opt-in）＋前202共214案全未跑，非library inventory或通過數；來源入口識別63／剩117。原64cast／32音效／512fragments／4096keys不變；共存／G11未驗。正常1049輸入、所有PE、正式JAR、registry及G06 binding／16案640次不變；未套用／選入／連結或執行PE／Java／SQL／測試／GUI／播放。\\n\\n'+area2230_note)
  catalog_note=catalog_note.replace('本輪候選62，尚118','本輪候選63，尚117').replace('最新識別集合見本輪sun46-weak47 qualification的current_source_coverage','最新識別集合見本輪freeze50 qualification的current_source_coverage')
'''
s=s.replace(needle,block+needle)
needle=' # The prior report is an immutable bridge to every historical source/result.';require(s.count(needle)==1,'report footer differs')
block=''' if f50_ref:
  body=body.replace('最新118個請求缺口','最新117個請求缺口').replace('其餘118個入口','其餘117個入口').replace('當版下一組#50／#51字面定義','當版下一組#51／#55字面定義')
  body=body.replace('class1新History及world音效仍須恢復後驗資源／重建／退役，不把來源識別數當驗收。','50入口／無音效756及flight接續已接候選；#51正式summonSpellRequest另有selection欄，接續獨立召喚選項UI／Owner／controller，不能當actor ID或selector-only。class1與flight History須恢復後驗原資源／重建／ring去重／失敗退役；不把來源識別數當驗收。')
  body=body.replace('驗T鍵與#46／','驗T鍵與#50／#46／').replace('最新33檔（含45／46／47與class1接續）需另建當版映像','最新33檔（再含50／無音效class4與flight接續）需另建當版映像')
'''
s=s.replace(needle,block+needle);p.write_text(s,encoding='utf-8',newline='\n');print(out)
