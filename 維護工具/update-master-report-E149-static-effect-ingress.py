"""Compact ingress handoff; decode completion is explicitly not callback enqueue time."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import re,json,hashlib
from native_E149_recovery_contract import R,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'

def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_static_effect_ingress_development'];require(sha(ref['evidence'])==ref['evidence_sha256'],'checkpoint differs');ep=Path(ref['evidence']);e=read(ep);require(sha(e['compile_cohort'])==e['compile_cohort_sha256'] and e['compiler_exit_codes']==[0]*20 and e['decoded_clock_forwarding_source_bound'] and e['decode_receipt_is_not_enqueue_clock'] and e['constructor_dispatch_and_fresh_enqueue_sample_pending'] and e['tests_executed']==0 and not e['S37_scene_history_and_retirement_queue_integrated'],'source scope differs');c=read(e['compile_cohort']);check_pins(c['normal_product_pins']);rb=read(ep.with_name('readback.json'));require(all(sha(R/n)==h for n,h in rb['after_files'].items()),'checkpoints differ');old=REPORT.read_bytes();body=old.decode('utf-8-sig')
 if 'worker解碼收據已接到World的靜態2941 cue' in body:print(json.dumps(dict(already_updated=True,master_sha256=sha(REPORT)),ensure_ascii=False));return
 require(sha(REPORT)=='1B9301C3FF85E3312D47C2299492CB03C0FFEBD0E5268F560F4CA857052341D6','master predecessor differs');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')+'（Asia/Taipei）',body,count=1)
 a=body.index('- 最新隔離候選仍42檔：');b=body.index('\n\n## 已完成',a);body=body[:a]+'- 最新隔離候選43檔：29 Runtime／11 Rust QA／3 Java，371module與原六nested；未套用正常來源／選registry／連結新PE。[解碼收據來源與20metadata](<'+ep.as_posix()+'>)；[最新八檔 checkpoint](<'+ep.with_name('readback.json').as_posix()+'>)；[工作佇列](<'+(R/'工作佇列.json').as_posix()+'>)。無已登記活動批次／session，來源未凍結。'+body[b:]
 a=body.index('## 已完成的來源開發');b=body.index('## G06與尚缺前提',a)
 section='''## 已完成的來源開發與目前界線

**最新 worker解碼收據已接到World的靜態2941 cue**。IngressReceipt在完整封包解密後、channel backpressure前取Instant；Event保留原值，LOGIN先核對目前connection epoch與該封包rx_counter，再由World為有效S37/2941建立generation／sequence／target incarnation／position綁定。ring淘汰與world退役同步清除，無時間的accept不造clock，JSON略過本機時鐘。這是來源接線，尚未執行TCP或GAME驗證。

**解碼完成時間不是callback排入佇列的時間**。原4D65F0取fresh clock再加delay；仍須接controller dispatch／constructor及實際enqueue sample，不能直接拿decode receipt、worker載入或paint時間當零delay期限。Scene／History／全域queue與退役尚未接入，97仍未完成，目前2941仍unsupported／omitted；不假設無像素、固定一幀或32秒刪除。原clock初始化／rollover、兩個重入繪圖suffix上游與4857B0巢狀drain交錯仍待。

三Runtime差分（native_connection／native_login／native_world），候選增至43檔／29 Runtime／11 Rust QA／3 Java，371module及原六nested不變。兩版library／library-test／七skill加network共20metadata均0（原01060f actual0）：原Debug兩成功保留，修正extern庫檔名後接續18項，沒有重跑兩成功項。新增4純QA＋前459共**463案全未跑**，不是完整library inventory；請求來源入口88／剩92不變，不代表88技能完整。

原metadata工具把庫檔命名成Debug-library.rmeta，第一個整合目標無法載入extern，原0／0／1與62a862工具exit1完整保留。只將同一成功庫內容複製為lib前綴alias並核對SHA，再接續；QA與產品來源未改，不將連帶型別錯誤誤當產品缺陷。前輪兩版library-only與原18metadata皆保留，不移記為新runtime資格。

#97靜態bank與frame0／Over提交橋已在來源圖內，沿同root shard→base resolver，核對list及2941／1120的size／SHA／單frame／metadata／原depth與atlas；只有MissingEntry走1120後備，不造Animation／Clip／FPS／WAV。普通精確list初始化推論flags0、direction6 alias=-1、shadow=-1；沒有任意後續globals或原生獨立像素oracle資格，loader／SPR解碼與提交橋均未執行。前輪條件生命週期核心仍未進371module，不能當已綁全域queue。

97請求／S34權威及98請求／原2942有限特效沿用：selector-only三byte、reuse9990／0；成本、材料、隱身／中毒、32／320秒與S08／現身由server掌管。98九幀Sound1505 source1／1496ms與既有History音效來源已接；NativeCastTimeline實際材料supplier caller鏈已核對，但不是whole GAME原子資格。#80共用selector_only、90／91 Hit續播、88／89與279、87持續4183、69同socket／server原子授權、direction initializer、constructor clock skew與LT-relative hurt等既有未驗界線保留。

正常1049、既有PE、正式JAR、registry及G06 binding／640次不變；未套用／選入、未連結新PE，沒有測試、session、GUI、播放或資源解碼。早期native四段漏ret、IDA endp後inline code、timeline與defaults修正及工具parser失敗的原始記錄均保留；完整來源鏈與前輪狀態見[此前完整報告與歷史證據](<'''+(out/'before.md').as_posix()+'''>)。

'''
 body=body[:a]+section+body[b:];a=body.index('1. 暫緩期間繼續開發：');b=body.index('2. 使用者明確恢復測試後：',a);body=body[:a]+'1. 暫緩期間繼續開發：先接97的controller dispatch／constructor與實際fresh enqueue clock，再接零delay queue／Scene／History／owner退役，串起static bank／Over橋與條件核心。不能把decode receipt、worker載入或paint時間當4D65F0 sample；保留同deadline順序、重入繪圖與巢狀drain，不固定一幀或借32秒期限。97完成後續99／100及剩92、4183與G06物品／source/account／timer、LT hurt/base。測試與桌面仍待使用者明確恢復。\n'+body[b:]
 for label in ['此前完整報告與歷史證據','舊映像與原始記錄']:body=re.sub(r'\['+label+r'\]\(<[^>]+>\)',lambda _:'['+label+'](<'+(out/'before.md').as_posix()+'>)',body)
 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report link missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in rb['after_files'].items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());entry.write_text('"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n',encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master differs');result=dict(before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),latest_evidence=str(ep),latest_evidence_sha256=sha(ep),lines=len(body.splitlines()),metadata_commands=20,retained_original_debug_compiles=2,new_metadata_compilers=18,tests_executed=0,hide97_renderer_integrated=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(dict(master_readback=str(out/'readback.json'),**result),ensure_ascii=False))

if __name__=='__main__':main()
