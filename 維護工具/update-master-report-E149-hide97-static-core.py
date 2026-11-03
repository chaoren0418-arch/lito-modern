"""Compact continuation report for the standalone, unintegrated static core."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import re, json
from native_E149_recovery_contract import R, read, sha, encoded, require, check_pins
T=R/'維護工具'; V=R/'驗證紀錄/current'; REPORT=R.parent/'LiTo381-總開發報告.md'

def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_hide97_static_core_development'];require(sha(ref['evidence'])==ref['evidence_sha256'],'checkpoint differs');p=Path(ref['evidence']);e=read(p);require(sha(e['source_evidence'])==e['source_evidence_sha256'] and not e['integrated_into_runtime'] and e['tests_executed']==0,'source scope differs');s=read(e['source_evidence']);check_pins(s['input_pins']);check_pins(s['normal_product_pins']);rb=read(p.with_name('readback.json'));require(all(sha(R/n)==h for n,h in rb['after_files'].items()),'eight checkpoints differ');old=REPORT.read_bytes();body=old.decode('utf-8-sig')
 if '獨立生命週期核心已完成兩版metadata' in body:
  print(json.dumps(dict(already_updated=True,master_report_sha256=sha(REPORT)),ensure_ascii=False));return
 require(sha(REPORT)=='F3CFB8DE21C52E2AA1078D25911751546B33AA9BFD6293DA3D6423307E6954DC','master predecessor differs');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')+'（Asia/Taipei）',body,count=1)
 a=body.index('**97的2941靜態圖呈現與退役尚未接入');b=body.index('\n\n四Runtime覆寫',a)
 text='''**97的2941靜態圖呈現與退役尚未接入，不能宣告97完整**。本輪獨立生命週期核心已完成兩版metadata（Debug／Release各0），不在目前42檔候選／371module內。零delay先入列、同deadline維持順序、固定sample逐筆先pop再callback；外部callback可在退役前觀察frame0，不假設固定一幀。Drain借用本model，sample不能跨model移用；owner退役清工作且保留admission tombstones，時鐘倒退／overflow拒絕。沒有綁實際S37／Scene／資源／History或全域queue。[獨立核心與來源](<'''+p.as_posix()+'''>)。

4E2150兩個重入488860的區域suffix條件已核對（限到達4E3207後）：第一條own+13=0、834F1D=0、834EB2=0、834EB3非0及641498=0；第二條own+13非0且641498=0。這些byte尚未授予高階語意或全路徑資格。另確認S37/169自己的4857B0含相對500ms drain／draw loop，2941不直接走169分支；巢狀drain、上游條件、原clock初始化與全部callback交錯仍缺。canonical 2941一幀四refs／1120後備的原始bytes已保存，尚未解碼像素。目前S37/2941仍unsupported／omitted；不借32秒本機期限，S34／S08／現身沿server權威。'''
 body=body[:a]+text+body[b:];a=body.index('1. 暫緩期間繼續開發：');b=body.index('2. 使用者明確恢復測試後：',a)
 body=body[:a]+'1. 暫緩期間繼續開發：先接97 canonical靜態圖、S37 admission／Scene／History／owner退役；釐清4E2150兩suffix上游、4857B0巢狀drain與全域clock／callback交錯。獨立core只做條件模型，不能當實際排程／像素資格，不設固定一幀或32秒刪除。97完成後續99／100與剩92請求，保留selector_only及coverage parser；再續4183 clock／S0B、G06物品／source/account／timer與LT hurt/base。測試與桌面仍待使用者明確恢復。\n'+body[b:]
 for label in ['此前完整報告與歷史證據','舊映像與原始記錄']:
  body=re.sub(r'\['+label+r'\]\(<[^>]+>\)',lambda _: '['+label+'](<'+(out/'before.md').as_posix()+'>)',body)
 require(all(Path(n).is_file()for n in re.findall(r'\]\(<([^>]+)>\)',body)),'report link missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in rb['after_files'].items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());entry.write_text('"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n',encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs');result=dict(before_sha256=__import__('hashlib').sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),source_evidence=e['source_evidence'],checkpoint=str(p.with_name('readback.json')),lines=len(body.splitlines()),metadata_commands_this_round=2,tests_executed=0,hide97_renderer_integrated=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(dict(master_readback=str(out/'readback.json'),**result),ensure_ascii=False))

if __name__=='__main__':main()
