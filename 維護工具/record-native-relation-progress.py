"""Record completed, source-bound Debug relation evidence, without qualification transfer."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
CORE=APP/'actual-tests-20261016-070104-441481'
LABELS=APP/'actual-tests-20261016-070345-699280'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
rel=lambda p:p.relative_to(ROOT).as_posix()
c=read(CORE/'qualified.json');l=read(LABELS/'qualified.json')
assert c['passed']==60 and l['passed']==16
assert c['current_worktree_checked'] and l['current_worktree_checked']
pins=read(CORE/'compiled-inputs.json'); assert pins==read(LABELS/'compiled-inputs.json') and len(pins)==999
for name,value in pins.items():assert sha(ROOT/name)==value,name
for p in [CORE,LABELS]:assert read(p/'receipt.json')['owned_process'] is None
formal=Path(r'D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar')
assert sha(formal)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
stamp=datetime.now(timezone.utc).isoformat()
evidence={'status':'Debug76_raw_actor_relations_and_existing_label_regressions_verified',
 'updated_utc':stamp,'compiled_inputs':999,'passed':76,'failed':0,'ignored':0,'actual_test_executables':17,
 'core':rel(CORE/'qualified.json'),'labels':rel(LABELS/'qualified.json'),
 'source_manifest_sha256':sha(CORE/'compiled-inputs.json'),
 'terminal_sessions':[{'session':99075,'exit_code':0},{'session':61235,'exit_code':0}],
 'earlier_S4F_S69_predecessor_58_not_counted':rel(APP/'actual-tests-20261016-065650-770780/qualified.json'),
 'source_frozen_for_active_job':False,'water_product_scene_integrated':False,
 'current_Release_verified':False,'normal_main_qualified':False,'current_GUI_verified':False,
 'actual_game_sessions':0,'formal_JAR_readonly_hash_rechecked':True,'formal_JAR_sha256':sha(formal),
 'whole_goal_complete':False}
save(APP/'actor-relations-current.json',evidence)
next_task=('當前999輸入：Debug12targets60/0/0加5個既有主人/label/clan-marker/emblem回歸16/0/0，共17實際EXE76/0/0，兩批完全相同來源與ZIP獨立讀回；session99075/61235實際exit0，無live job。S0B/S4F/S69保留raw MS950名稱，S36重複敵盟、選擇性清1F、事件graphics歷史與GAME開始/取消/失敗/傳送退役已實作；S69沒有S0B自有寵物例外，空clan只清20。比較採明示MS950，不宣稱原ACP初始化已核實。下一步把關係資料、同Pose action與分類接入水下producer，再接純Rust水下額外pass及GAME-owned phase/scroll：原S28首次50ms、487510續100ms、4D65F0無callback去重、S28false取消不直接重置phase；按maploader/cancel與共用camera鏈補查，不猜elapsed快轉。水下scene仍未整合；map303/special globals及完整回歸待。最新999 Release/唯一入口正常主/GUI未驗，舊9949A21/68DB兩版43/TW14保留其SHA不移記。新增relations target後default測試inventory須重新核對。全部產物只在重建版，登入器/相鄰伺服器唯讀，不替換登入器JAR；全G01–G12未完成。')
for name in ['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json']:
    p=ROOT/name; v=read(p)
    v.update(progress_updated_utc=stamp,next_task=next_task,current_working_tree_verified=False,
             working_tree_modified_since_current_normal_qualification=True,
             native_actor_relations_current_work=evidence,last_goal_turn_classification='progress',
             last_goal_turn_evidence=[rel(APP/'actor-relations-current.json'),'docs/NATIVE-ACTOR-RELATIONS.zh-TW.md'])
    a=v.get('working_tree_native_lighting_application',{})
    a.update(status=evidence['status'],compiled_inputs=999,current_test_session=61235,current_test_terminal_exit_code=0,
             source_frozen_for_current_tests=False,normal_mains_rebuilt_for_repair=False,water_product_scene_integrated=False,
             current_Debug_qualification=rel(APP/'actor-relations-current.json'),actor_relations_qualification=rel(APP/'actor-relations-current.json'))
    v['working_tree_native_lighting_application']=a
    if isinstance(v.get('current_product'),dict):
        v['current_product']['status']='last_994_normal_scoped43_GUI14_baseline_working_tree999_Debug76_relations_Release_normal_GUI_pending'
    save(p,v); assert read(p)==v
notice='2026-10-16工作樹：999輸入，Debug十二實際測試EXE60過，加五個既有主人/label/clan-marker/emblem回歸16過，合17EXE76過0敗0忽略，完全相同來源與ZIP獨立讀回。新增raw MS950名稱、S36事件旗標、S4F/S69更新與GAME關係退役；水下producer/phase/scroll仍待接Scene。最新Release/唯一入口正常主/GUI尚未驗；以下997Debug50與994正常43/TW14保留原內容資格，不移記。見[關係狀態限定說明](NATIVE-ACTOR-RELATIONS.zh-TW.md)。'
for filename in ['README.md','docs/ARCHITECTURE.zh-TW.md']:
    p=ROOT/filename; text=p.read_text(encoding='utf-8-sig'); first=text.find('\n')
    link='docs/NATIVE-ACTOR-RELATIONS.zh-TW.md' if filename=='README.md' else 'NATIVE-ACTOR-RELATIONS.zh-TW.md'
    line=notice.replace('NATIVE-ACTOR-RELATIONS.zh-TW.md',link)
    p.write_text(text[:first+1]+'\n'+line+'\n'+text[first+1:],encoding='utf-8')
p=ROOT/'docs/HISTORY.md'
with p.open('a',encoding='utf-8') as f:
    f.write('\n\n## 2026-10-16 999原始MS950關係與S36生命週期Debug76\n\n'+
        '新增native_actor_relations並接S0B/S36、GAME begin及取消/失敗/F4退役；同GAME換圖/portal保留context。保留原始MS950名稱，不把同Unicode不同byte視為同一關係；1F/20依事件更新，盟戰結束捕獲當時gfx後按class11/6/12清旗標，重複敵盟僅移除第一筆。靜態補查S4F只換master、S69同clan setter無自有pet例外且空clan只清20，因此同步補raw更新與兩測試。\n\n'+
        '首輪999十二target58/0/0來源與原EXE保留；S4F/S69修後新999同一完整manifest，session99075十二target60/0/0、61235五個既有master/label/clan-marker/emblem target16/0/0均實際exit0，合17實際AMD64 EXE76/0/0，來源/test ZIP逐份讀回。沒有新增GAME sessions、原S36 handler執行或最新Release/正常主/GUI資格；同為999的首輪58不移記，舊99750與99443/TW14仍按其原來源保留。產品水下scene/phase/scroll仍未整合；原ACP比較表未動態核實。所有寫入限重建版，正式只讀C02 JAR hash再核對，無登入器替換。詳NATIVE-ACTOR-RELATIONS.zh-TW.md。\n')
print(json.dumps({'status':evidence['status'],'source_inputs':len(pins),'formal_JAR_unchanged':True}))
