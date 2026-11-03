"""Record the source-bound water producer build checkpoint, not full water parity."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,sys
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
RUN=Path(sys.argv[1]).resolve(strict=True);assert RUN.is_relative_to(APP)
Q=RUN/'normal-qualified';read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
rel=lambda p:p.relative_to(ROOT).as_posix()
n=read(Q/'qualified.json');s=read(Q/'scoped-qualified.json');assert n['source_files']==s['source_files']==999
assert s['counts']['Debug']['passed']==81 and s['counts']['Release']['passed']==65
assert read(RUN/'receipt.json')['owned_process'] is None
pins=read(Q/'source-inputs.json')
for p,h in pins.items():assert sha(ROOT/p)==h,p
for p in n['profiles'].values():assert sha(Path(p['path']))==p['sha256']
formal=Path(r'D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar')
assert sha(formal)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
stamp=datetime.now(timezone.utc).isoformat()
e={'status':s['status'],'updated_utc':stamp,'source_inputs':999,
   'normal_qualification':rel(Q/'qualified.json'),'scoped_qualification':rel(Q/'scoped-qualified.json'),
   'normal_mains':s['normal_mains'],'source_manifest_sha256':s['source_manifest_sha256'],'counts':s['counts'],
   'actual_test_session':49076,'actual_test_terminal_exit_code':0,'owned_process':None,
   'water_source_producer_scene_integrated':True,'water_extra_effect_complete':False,
   'current_GUI_verified':False,'GUI_tools_prepared_not_executed':rel(APP/'water-GUI-adapter-preparation.json'),
   'timer_camera_static_evidence':'驗證紀錄/current/native-lighting-actor-motion-static/water-lifecycle-20261016-071847-339171/readback.json',
   'new_game_sessions':0,'formal_JAR_sha256':sha(formal),'whole_goal_complete':False}
save(APP/'water-source-current.json',e)
next_task=('999完整輸入，水下producer已用Model同Pose action/source cell/center，raw master/1F/50/low mode23與歷史gfx分類；metadata/action查詢和compose同到700停止。Debug12targets65+5既有label16=17EXE81/0/0，Release12targets65/0/0與唯一入口正常Debug/Release兩主已獨立讀回；實際session49076 exit0，無live job。正常主 '+s['normal_mains']['Debug'][:8]+'/'+s['normal_mains']['Release'][:8]+'。新版GUI仍未驗：新版adapter已解析，case/authority/recovery/window四原驗證工具與舊994raw逐byte一致，可用run-native-lighting-TW14-followup.ps1 -NormalQualification '+str(Q/'qualified.json')+' -RustEvidence '+str(Q/'scoped-qualified.json')+'跑原TW+extended14；state保存在normal-qualified/GUI-current.json，不能誤讀舊APP/GUI-current.json。先完成當版GUI再改來源。水下額外shader/phase/scroll仍未整合；原4874D0無water gate，乾地步行也累積scroll；485800/48BC80在水下另加50ms callback鏈，4D65F0無去重，4D6870刪除全部匹配，S28false不直接重設phase。下一步依這些實際鏈完成GAME-owned phase與共用鏡頭增量及額外water pass，不以單一timer/入水清零/elapsed快轉替代。map303/special globals、完整default/lib/TCP/實體輸入/可攜/穩定性仍待，完整G01–G12未完成。登入器及相鄰伺服器唯讀，無登入器JAR替換。')
for name in ['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json']:
    p=ROOT/name;v=read(p)
    if 'last_qualified_product_before_water_source_producer' not in v:v['last_qualified_product_before_water_source_producer']=v.get('current_product')
    v.update(progress_updated_utc=stamp,next_task=next_task,current_working_tree_verified=False,
        working_tree_modified_since_current_normal_qualification=False,source_freeze_while_Cargo=False,
        native_water_source_current_work=e,last_goal_turn_classification='progress',
        last_goal_turn_evidence=[rel(APP/'water-source-current.json'),'docs/NATIVE-WATER-SOURCES.zh-TW.md'],
        current_scoped_tests_verified=True,current_GUI_all14_complete=False)
    v['current_product']={'status':'normal999_water_source_producer_Debug81_Release65_qualified_GUI_pending',
        'source_files':999,'source_manifest_includes_compile_fixtures':True,
        'source_manifest_sha256':s['source_manifest_sha256'],'normal_qualification':rel(Q/'qualified.json'),
        'normal_mains':s['normal_mains'],'current_scoped_evidence':rel(Q/'scoped-qualified.json'),
        'current_full_default_regression_verified':False,'current_GUI_verified':False,'whole_goal_complete':False}
    v['current_normal_lighting_product']=v['current_product']
    a=v['working_tree_native_lighting_application'];a.update(status=s['status'],compiled_inputs=999,
        current_test_session=49076,current_test_terminal_exit_code=0,source_frozen_for_current_tests=False,
        normal_mains_rebuilt=True,normal_mains_rebuilt_for_repair=True,
        normal_qualification=rel(Q/'qualified.json'),scoped_qualification=rel(Q/'scoped-qualified.json'),
        current_Debug_qualification=rel(Q/'scoped-qualified.json'),water_source_producer_scene_integrated=True,
        water_product_scene_integrated=False,GUI='latest999_prepared_not_executed',GUI_session=None)
    save(p,v);assert read(p)==v
notice=('2026-10-16當前正常主Debug '+s['normal_mains']['Debug'][:8]+'／Release '+s['normal_mains']['Release'][:8]+
    '、999編譯輸入已接Model水下來源分類：同Pose action、raw主人/盟戰旗標與700範圍停止。Debug17實際測試EXE81過、Release12EXE65過，各0敗0忽略；來源/test/唯一入口兩主ZIP獨立讀回。最新GUI尚未驗，額外water shader/phase/scroll仍待；下列999關係76與997/994皆為變更前資格，不移記。')
for name in ['README.md','docs/ARCHITECTURE.zh-TW.md']:
    p=ROOT/name;text=p.read_text(encoding='utf-8-sig');i=text.find('\n');link='docs/NATIVE-WATER-SOURCES.zh-TW.md' if name=='README.md' else 'NATIVE-WATER-SOURCES.zh-TW.md'
    line=notice+' 見[水下來源限定紀錄]('+link+')。'
    p.write_text(text[:i+1]+'\n'+line+'\n'+text[i+1:],encoding='utf-8')
with (ROOT/'docs/HISTORY.md').open('a',encoding='utf-8') as f:
    f.write('\n\n## 2026-10-16 水下來源producer與999正常兩主\n\n'+notice+'\n\n'+
        '新增四個adapter測試與一個真資源Model測試，class5到死亡action8的class3/range14到5及body center一致、World不變已驗；其他涵蓋raw master/native50/low mode23/signed action、war end歷史gfx、缺meta/action/owner不動pixels，以及50個range14達700後不查第51個未知gfx。普通RGB mask在unblinded water仍略過，producer現在確實生成mask；額外water pass未接，未假稱水下畫面完成。source_range和compose共用同一選取規則，原864分類/光源fixture照跑。\n\n'+
        '原timer/camera補靜態證據：dry self-walk也更新4874D0；maploader485800/48BC80各會再排50ms，413A80 cleanup只取消物件414310；4D6870會移除所有同callback/owner。未原執行完整scheduler，間接bulk初始化與整條camera/map dispatch仍待。新版GUI adapter準備完成但尚未執行，四個行為/wire關鍵工具與舊994raw逐byte一致。來源與正常主保持至下一步當版TW14；正式只讀C02hash再核，沒有外部JAR替換。\n')
print(json.dumps({'status':e['status'],'normal_mains':e['normal_mains'],'GUI_verified':False}))
