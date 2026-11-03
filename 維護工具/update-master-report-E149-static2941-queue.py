"""Keep a compiled queue prototype distinct from correct resource ordering and GUI."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import re,json,hashlib
from native_E149_recovery_contract import R,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'

def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_static2941_queue_development'];require(sha(ref['evidence'])==ref['evidence_sha256'],'evidence differs');ep=Path(ref['evidence']);e=read(ep);require(sha(e['compile_cohort'])==e['compile_cohort_sha256'] and e['compiler_exit_codes']==[0]*20 and e['controller_queue_and_fresh_enqueue_sample_bound'] and e['lease_revocation_shared_with_snapshots'] and not e['source_ready_precedes_enqueue_qualified'] and not e['Scene_and_History_bound'] and not e['normal_apply_eligible'] and e['tests_executed']==0,'prototype boundary differs');c=read(e['compile_cohort']);check_pins(c['normal_product_pins']);rb=read(ep.with_name('readback.json'));require(all(sha(R/n)==h for n,h in rb['after_files'].items()),'checkpoints differ');old=REPORT.read_bytes();body=old.decode('utf-8-sig')
 if 'controller專用queue與observer存活token已接入隔離候選' in body:print(json.dumps(dict(already_updated=True,master_sha256=sha(REPORT)),ensure_ascii=False));return
 require(sha(REPORT)=='BB3782214013D3C60796152AB476641CA5C07F7163488B706D53B9D6841C9D08','master predecessor differs');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')+'（Asia/Taipei）',body,count=1)
 a=body.index('- 最新隔離候選43檔：');b=body.index('\n\n## 已完成',a);body=body[:a]+'- 最新隔離候選44檔：30 Runtime／11 Rust QA／3 Java，372module與原六nested；屬尚不可套用的queue原型，未改正常來源／registry／既有PE。[原型來源與20metadata](<'+ep.as_posix()+'>)；[最新八檔 checkpoint](<'+ep.with_name('readback.json').as_posix()+'>)；[工作佇列](<'+(R/'工作佇列.json').as_posix()+'>)。無已登記活動批次／session，來源未凍結。'+body[b:]
 a=body.index('## 已完成的來源開發');b=body.index('## G06與尚缺前提',a)
 section='''## 已完成的來源開發與目前界線

**最新 controller專用queue與observer存活token已接入隔離候選**。新增native_static2941 module，controller保存唯一退役權限；World／背景快照只共享Arc／Atomic存活token，不能重設為活躍。記錄decode、dispatch、候選constructor及fresh enqueue四時刻；zero-delay先排隊，同deadline維持順序，poll批次後固定sample逐筆pop再退役，paint不推時鐘。ring淘汰、connection／generation換代、五處world清除、state reset及queue drop均撤銷token；admission用單一watermark，避免無限seen累積或同receipt復活。

**仍是不可套用正常來源的中間原型，97未完成**。原4E5C00先準備靜態SPR，再由4D65F0取fresh clock排程；目前候選排程點仍早於背景資源準備，必須補資源就緒gate。專用queue尚未合併原共享callback／重入繪圖，Scene／History、cached frame與晚到worker也尚未消費token，不能因此宣告像素、正確先後或97完整。不得以「必定沒畫面」、固定一幀或32秒本機刪除代替；不得將此原型直接選入／套用。

三既有Runtime覆寫加一個queue module，增至44檔／30 Runtime／11 Rust QA／3 Java，372module／原六nested。兩版library／library-test／七skill加network共20metadata均0，本輪為新44檔來源圖完整重編，不移記前輪保留兩Debug成功的20項。新增8純QA＋前463共**471案全未跑**，不是完整library inventory；入口88／剩92不變，不代表88技能完整。前輪extern命名失敗0／0／1及62a862、同SHA alias與只補18項的原收據保留。

前輪IngressReceipt仍在完整封包解密後、channel backpressure前取Instant，LOGIN核對目前epoch與該封包rx_counter；World為有效S37/2941綁generation／sequence／target incarnation／position，ring／world退役清除，無時間的accept不造clock，JSON略過本機時鐘。decode時間與enqueue時間明確分開；現代Instant與原PE clock初始化／rollover／全部交錯尚未資格。專用bank及frame0／Over提交橋仍核對list、2941／1120成員與原fragments，只有MissingEntry走1120後備；未呼叫loader或SPR解碼，也未形成可見GUI。

97請求／S34及98請求／原2942有限特效、材料supplier caller鏈沿用，非whole GAME原子資格；成本／材料／隱身／中毒／32與320秒、S08及現身均留server。#80共用selector_only、90／91 Hit續播、88／89與279、87持續4183、69同socket、constructor skew／direction初始化與LT-relative hurt等未驗界線保留。正常1049、既有PE、正式JAR、registry及G06 binding／640次不變；未套用／選入、未連結PE，沒有測試、session、桌面、播放或資源解碼。

原五函式範圍修正、timeline完整ret、parser／base constructor的IDA inline code與普通flags0／alias=-1／shadow=-1推論及歷次工具失敗皆保留；它們不是原生獨立像素oracle。完整來源鏈與前輪狀態見[此前完整報告與歷史證據](<'''+(out/'before.md').as_posix()+'''>)。

'''
 body=body[:a]+section+body[b:];a=body.index('1. 暫緩期間繼續開發：');b=body.index('2. 使用者明確恢復測試後：',a);body=body[:a]+'1. 暫緩期間繼續開發：先把97資源就緒gate放在實際enqueue之前，符合原4E5C00先SPR、4D65F0再取fresh clock；目前queue原型不得直接套用。保留controller-owned token，接bank／Scene／History與cached／late-worker token檢查，再補共享callback／重入繪圖／巢狀drain，不固定一幀或借32秒期限。97完成再99／100及剩92、4183與G06物品／source/account／timer、LT hurt/base。測試與桌面仍待使用者明確恢復。\n'+body[b:]
 for label in ['此前完整報告與歷史證據','舊映像與原始記錄']:body=re.sub(r'\['+label+r'\]\(<[^>]+>\)',lambda _:'['+label+'](<'+(out/'before.md').as_posix()+'>)',body)
 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report link missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in rb['after_files'].items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());entry.write_text('"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n',encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master differs');result=dict(before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),latest_evidence=str(ep),latest_evidence_sha256=sha(ep),lines=len(body.splitlines()),metadata_commands=20,tests_executed=0,normal_apply_eligible=False,hide97_renderer_integrated=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(dict(master_readback=str(out/'readback.json'),**result),ensure_ascii=False))

if __name__=='__main__':main()
