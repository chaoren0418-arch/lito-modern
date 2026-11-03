"""Fix only the source reader's generated-suffix newline accounting."""
from pathlib import Path
from datetime import datetime,timezone
from native_E149_recovery_contract import R,sha,encoded,require
T=R/'維護工具';V=R/'驗證紀錄/current';out=V/'native-E149-summon51-tooling-notes'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name in ['qualify-native-E149-summon51.py','prepare-native-E149-summon51-publication.py','update-master-report-E149-current.py']:(out/name).write_bytes((T/name).read_bytes())
p=T/'qualify-native-E149-summon51.py';s=p.read_text('utf-8-sig');a="  s+=row['generated_suffix']";b="  if row['generated_suffix']:s+='\\n'+row['generated_suffix']";require(s.count(a)==1,'suffix accounting anchor differs');s=s.replace(a,b)
needle=" for p in [Path(__file__).resolve(),T/'readback-native-E149-summon51.py',T/'readback-native-E149-eruption45-core.py']:pins[str(p)]=sha(p)";require(s.count(needle)==1,'publisher pin marker differs');s=s.replace(needle,needle+"\n pins[str(T/'fix-summon51-publication-newline.py')]=sha(T/'fix-summon51-publication-newline.py')\n pins["+repr(str(out/'receipt.json'))+"]=sha("+repr(str(out/'receipt.json'))+")")
p.write_text(s,encoding='utf-8',newline='\n')
p=T/'prepare-native-E149-summon51-publication.py';s=p.read_text('utf-8-sig');require(s.count(a)==1,'derivation source anchor differs');s=s.replace(a,b.replace('\\n','\\\\n'));p.write_text(s,encoding='utf-8',newline='\n')
p=T/'update-master-report-E149-current.py';s=p.read_text('utf-8-sig');a='未重跑。正常1049輸入';b='未重跑。發布reader漏計清單前一換行曾退出1；原工具封存，只修讀回並沿相同Rust來源／18metadata，不重編或執行測試。正常1049輸入';require(s.count(a)==1,'report note marker differs');s=s.replace(a,b);p.write_text(s,encoding='utf-8',newline='\n')
(out/'receipt.json').write_bytes(encoded(dict(original_tool_chunk='d003b2',original_tool_actual_exit_code=1,diagnosis='generated SummonSelection suffix has one leading newline appended by preparer; reader omitted it',only_native_skills_reconstruction_differed=True,Rust_sources_changed=False,compilers_rerun=False,tests_executed=0,original_sources={p.name:sha(p) for p in out.iterdir() if p.is_file()},whole_goal_complete=False)));print(out)
