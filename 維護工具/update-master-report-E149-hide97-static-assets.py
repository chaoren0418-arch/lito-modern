"""Keep the total report concise; distinguish library metadata from runtime qualification."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import re,json,hashlib
from native_E149_recovery_contract import R,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'

def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_hide97_static_assets_development'];require(sha(ref['evidence'])==ref['evidence_sha256'],'checkpoint differs');ep=Path(ref['evidence']);e=read(ep);require(sha(e['compile_cohort'])==e['compile_cohort_sha256'] and e['compiler_exit_codes']==[0,0] and e['tests_executed']==0 and not e['loader_invoked'] and not e['S37_scene_history_integrated'],'source scope differs');c=read(e['compile_cohort']);check_pins(c['normal_product_pins']);rb=read(ep.with_name('readback.json'));require(all(sha(R/n)==h for n,h in rb['after_files'].items()),'eight checkpoints differ');old=REPORT.read_bytes();body=old.decode('utf-8-sig')
 if '專用Static2941Bank已進入42檔候選來源圖' in body:print(json.dumps(dict(already_updated=True,master_report_sha256=sha(REPORT)),ensure_ascii=False));return
 require(sha(REPORT)=='5E602E19805F2DDF2C8B2143C5C7F09BA0FBFAFB5739D0B1CA3B4915C2F5E25D','master predecessor differs');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')+'（Asia/Taipei）',body,count=1)
 a=body.index('- 最新隔離候選42檔：');b=body.index('\n\n## 已完成',a);body=body[:a]+'- 最新隔離候選仍42檔：28 Runtime／11 Rust QA／3 Java，371module與原六nested；未套用正常來源／選registry／連結新PE。[靜態bank最新來源與兩版library metadata](<'+ep.as_posix()+'>)；[前輪請求／有限特效18metadata](<'+e['previous_qualified_18_metadata']['qualification'].replace('\\','/')+'>)；[最新八檔 checkpoint](<'+ep.with_name('readback.json').as_posix()+'>)；[工作佇列](<'+(R/'工作佇列.json').as_posix()+'>)。無已登記活動批次／session，來源未凍結。'+body[b:]
 a=body.index('**97的2941靜態圖呈現與退役尚未接入');b=body.index('\n\n#80共用selector_only',a)
 section='''**97的2941靜態圖呈現與退役尚未接入，不能宣告97完整**。本輪專用Static2941Bank已進入42檔候選來源圖：原Text list SHA、無action／type9定義、2941及1120的size／SHA、單frame／metadata／fragments皆嚴格核對，沿同root shard→base resolver，保留depth與atlas。只有MissingEntry走1120原後備；變更、損壞、IO或cancel均拒絕。不造Animation／Clip／FPS／dwell，不載WAV。兩版library metadata退出0（原e19a41 actual0），**沒有呼叫loader或解碼SPR**；本版library-test及七整合metadata未重編，不能移記前輪18。

前輪獨立生命週期核心（不在371module內）仍為條件模型：0ms先入列、同deadline穩定順序、固定sample逐筆先pop再callback，繪圖不推進時鐘，owner退役保留admission tombstones。它與bank尚未接S37／Scene／History或全域queue，實際仍走unsupported／omitted；不保證無像素或固定一幀，不借32秒期限。方向6 alias／shadow defaults、原clock初始化／巢狀drain與全部交錯仍缺。

4E2150兩個重入488860的區域suffix已核對（限到達4E3207後）：第一條own+13=0、834F1D=0、834EB2=0、834EB3非0且641498=0；第二條own+13非0且641498=0。尚未授予byte高階語意或全CFG資格。S37/169自己的4857B0含相對500ms drain／draw loop，2941不直接走169分支；上游／巢狀queue仍待。S34／S08／現身沿server權威。

本輪只覆寫tower_assets，仍42檔／28 Runtime／11 Rust QA／3 Java、371module與原六nested；零新增QA，**前459案全未跑**，不是完整library inventory。請求來源入口88／剩92不變，不代表88技能完整；下一組99／100，先補97。前輪4案首次編譯及18metadata收據保留，本輪不授予當版QA資格。'''
 body=body[:a]+section+body[b:];a=body.index('1. 暫緩期間繼續開發：');b=body.index('2. 使用者明確恢復測試後：',a);body=body[:a]+'1. 暫緩期間繼續開發：先將97靜態bank接S37 admission／Scene／History／owner退役及零delay實際queue；核對兩suffix上游、4857B0巢狀drain、clock／方向6 alias／shadow defaults。不能用一般Clip、固定一幀或32秒刪除。當版library-test及七整合metadata另待重編；97完成後續99／100及剩92請求，保留selector_only／coverage parser，再續4183與G06物品／source/account／timer、LT hurt/base。測試與桌面待使用者明確恢復。\n'+body[b:]
 for label in ['此前完整報告與歷史證據','舊映像與原始記錄']:body=re.sub(r'\['+label+r'\]\(<[^>]+>\)',lambda _:'['+label+'](<'+(out/'before.md').as_posix()+'>)',body)
 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report link missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in rb['after_files'].items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());entry.write_text('"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n',encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master differs');result=dict(before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),latest_evidence=str(ep),latest_evidence_sha256=sha(ep),lines=len(body.splitlines()),metadata_commands_this_round=2,tests_executed=0,hide97_renderer_integrated=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(dict(master_readback=str(out/'readback.json'),**result),ensure_ascii=False))

if __name__=='__main__':main()
