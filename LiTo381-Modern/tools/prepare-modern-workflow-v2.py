"""One-time backlog-to-contract conversion; --seal only records validation/handoff."""
import hashlib
import json
import pathlib
import sys
from datetime import datetime, timezone, timedelta

P = pathlib.Path(__file__).resolve().parent.parent
G = P.parent
L = P/'logs/continuation-20261003'
Q = G/'工作佇列.json'
NOW = datetime.now(timezone(timedelta(hours=8))).isoformat()
ROOT = L/'modern-workflow-v2-handoff.json'
INDEX = pathlib.Path('D:/lineage開發/3-lito開發/LITO-監製索引.json')
SKILL = pathlib.Path('C:/Users/CHAOREN/.codex/skills/lito-supervisor')

def read(path):
    return json.loads(pathlib.Path(path).read_text(encoding='utf-8-sig'))

def pin(path):
    path = pathlib.Path(path).resolve()
    return {'path':str(path),'bytes':path.stat().st_size,
            'sha256':hashlib.sha256(path.read_bytes()).hexdigest().upper()}

def save(path, value):
    with pathlib.Path(path).open('x',encoding='utf-8',newline='\n') as f:
        json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
    return pin(path)

def seal():
    queue = read(Q)
    validation = read(L/'modern-workflow-v2-validation.stdout.json')
    assert validation['valid'] is True and validation['task_count']==len(queue['items'])
    assert queue['ready_count']==0 and queue['execution_state']=='waiting'
    assert len(queue['mode_mapping'])==40
    assert sum(x['existing_v47_flow_complete'] for x in queue['mode_mapping'])==16
    ids = {x['id'] for x in queue['items']}
    assert all(set(x['task_ids']) <= ids for x in queue['mode_mapping'])
    assert len(queue['backlog_category_mapping'])==15
    assert all(set(x['task_ids']) <= ids for x in queue['backlog_category_mapping'])
    root = save(ROOT,{'schema':'modern.workflow-v2.handoff.v1','updated_at':NOW,
        'task_id':'modern-workflow-v2','state':'review','execution_state':'waiting',
        'queue':pin(Q),'validator':validation,'ready_count':0,'eligible_task_id':None,
        'next_on_input':'modern-background-scene-binding',
        'minimal_external_input':queue['minimal_unblocking_options'],
        'task_count':len(queue['items']),'mode_mapping_count':40,'backlog_categories':15,
        'existing_program_complete_modes':16,'remaining_program_modes':24,
        'new_product_implementation':False,'new_product_acceptance':False,'overall_completed':False,
        'new_builds':0,'new_app_tests':0,'old_PASS_replays':0,'socket_Java_SQL_provider_GUI':0,
        'candidate_identity_preserved':queue['candidate_baseline'],
        'references':[pin(Q),pin(L/'modern-workflow-v2-selection.json'),
            pin(L/'modern-workflow-v2-validation.stdout.json'),pin(__file__),
            pin(SKILL/'SKILL.md'),pin(SKILL/'references/task-contract.md')]+queue['source_evidence']})
    progress_path=G/'監製進度.json';progress=read(progress_path)
    progress.update(workflow_queue_path=str(Q),workflow_task_id='modern-workflow-v2',
                    workflow_task_state='review',workflow_handoff=root,updated_at=NOW)
    progress['workflow_execution_state']='waiting';progress['workflow_ready_count']=0
    progress['workflow_next_eligible_task_id']=None
    progress['workflow_next_on_input']='modern-background-scene-binding'
    progress['overall_status']='in_progress';progress['overall_completed']=False
    done='modern-workflow-v2：一次性工作契約轉換；40模式/15類待辦映射，16歷史v47完成/24未完成保留。schema驗證通過，ready_count=0，等待精確外部來源；未新增產品實作或重跑PASS。'
    if done not in progress['completed']:progress['completed'].append(done)
    if str(ROOT) not in progress['evidence_paths']:progress['evidence_paths'].append(str(ROOT))
    with progress_path.open('w',encoding='utf-8',newline='\n') as f:
        json.dump(progress,f,ensure_ascii=False,indent=2);f.write('\n')
    print(json.dumps({'handoff':root,'tasks':len(queue['items']),'ready_count':0,
                     'workflow_task_state':'review','execution_state':'waiting'},ensure_ascii=False))

if '--seal' in sys.argv:
    seal();sys.exit(0)

index=read(INDEX)
assignment=next(x for x in index['dispatch_queue']['items'] if x['id']=='modern-workflow-v2')
assert assignment['owner']=='modern'
scope=read(L/'r540-backlog-scope-update.json')
progress=read(G/'監製進度.json')
assert len(scope['per_mode_map'])==40 and sum(x['existing_v47_flow_complete'] for x in scope['per_mode_map'])==16
assert len(scope['items'])==15 and not Q.exists()
base=[pin(L/name) for name in ('r542-repository-native-candidate-handoff.json',
     'r541-native-room-application-lifecycle-handoff.json','r541-native-room-exact-gaps.json',
     'r540-backlog-scope-update.json','r533-full-backlog-audit.json')]
assert base[0]['sha256']=='78D79A0A7431D1E78791FBF65A5F624A972F6418775FF5407518387E419E1377'
base.append(progress['r540']['full_WORLD_blocker'])
extra={}
for name in ('r458-minimal-runtime-producer-gaps.json','r456-r458-combined-handoff-corrected.json',
             'r473-resource-runtime-boundary.json'):
    if (L/name).exists():extra[name]=pin(L/name)
BASE_BOUNDARY=('只写Modern自有目录；R542/R532固定候选、default-run与既有Modern流程保持。CPU/CLI不等于live GUI，ROOM不等于WORLD，5350不能升格485F；'
 '不启动provider/Java/SQL，不消费READY/claim，不重开T73，不绕过Native工具/私有backing或server T291/T27/refund/rebid/auction/castle及full-WORLD拦截；不重跑旧PASS。')
items=[]

def gate(id, supplier, missing, consumer, ephemeral=False):
    g={'id':id,'state':'missing','evidence':None,'supplier':supplier,'missing_input':missing,'consumer':consumer}
    if ephemeral:
        g['expires_at']=NOW
        g['expiry_note']='closed unavailable gate; this timestamp does not assert human availability or a provider lease'
    return g

def task(id,title,priority,deliver,accept,missing,supplier='bin_resources',deps=(),policy=False,evidence=None,gates=None):
    gs=gates or [gate(id+'-qualified-input',supplier,missing,id)]
    resume='供應者交付來源SHA／版本／適用狀態／所有權與生命週期完整、未越過原攔截的上述缺失資料；只重新審查受影響task。'
    t={'id':id,'owner':'modern','title':title,'priority':priority,'state':'blocked',
       'next_deliverable':str(L/(id+'-handoff.json'))+'；'+deliver,
       'acceptance':accept,'dependencies':list(deps),'gates':gs,
       'blockers':[{'reason':g['missing_input'],'supplier':g['supplier'],'missing_input':g['missing_input'],
                    'resume_when':resume,'consumer':id} for g in gs if g['state']=='missing'],
       'boundary':BASE_BOUNDARY,'resume_when':resume,'policy_blocked':policy,
       'evidence':evidence or base[:5],'handoff_to':['supervisor','modern'],
       'stop_condition':'只實作來源支持的缺口及必要新驗證並交review；缺輸入即停止，不造wrapper/資料、重演已通過矩陣或改写报告维持忙碌。'}
    items.append(t);return t

items.append({'id':'modern-workflow-v2','owner':'modern','title':assignment['title'],'priority':10,'state':'review',
    'next_deliverable':str(Q)+'；40模式/15類待辦契約與'+str(ROOT)+'一次性交接',
    'acceptance':'supervise.py validate --queue 通過；依賴ID本清單且無環，40行/16已完成/24未完成完整保留；區分產品與工作流成果，沒有獨立ready時精確列解阻條件。',
    'dependencies':[],'gates':assignment['gates'],'blockers':[],
    'boundary':BASE_BOUNDARY+' 一次性轉換不修好產品，不建立週期報告工作。',
    'resume_when':'監製審查本次新佇列與schema驗證結果；後續僅證據/依賴變更可啟用具體產品task。',
    'policy_blocked':False,'evidence':base,'handoff_to':['supervisor'],
    'stop_condition':'完成佇列驗證與交接即停止；ready_count=0時產品執行waiting，不要求再次派相同清單。'})

task('modern-gui-room','R542有限ROOM玩家真人驗收',10,
 '固定R542新鮮受控真人操作收據：登入/Join/Ready/Cancel/Close、actual D1/E1與雙端退休；只驗ROOM',
 'EXE/library SHA匹配R542；操作/封包/GET與cleanup同一fresh run；焦點與密碼清除有觀察；GUI結果獨立於既有2CPU PASS，無WORLD推論。',
 '目前缺真人就緒、有效原生視窗/輸入/幾何能力及fresh監製核准485F供應。',
 gates=[gate('modern-human_ready-room','human','同一有限操作當下真人就緒及可操作時窗','modern-gui-room',True),
        gate('modern-native-input-geometry-room','supervisor/tool capability','原允許原生screen/input/座標幾何能力；現有Native CUA停用不可繞過','modern-gui-room'),
        gate('modern-provider-fresh-room','server','新READY/可信SHA/lease/PID creation-time與identity/未耗claim/清理責任全部原條件','modern-gui-room',True)])
task('modern-ui-dpi','既有UI資產／DPI與hit測量驗收',12,
 '對R542既有UI/menu/ROOM的焦點、兩DPI、點擊命中與更新閃爍收據；只在新可重現缺陷有來源時改產品',
 '兩DPI輸入與原資產顯示有同一instance有效screen/geometry；離線測量不連provider；來源未證明的座標不猜、提取RGBA不當GUI。',
 '當下真人與合法原生幾何/截圖/輸入能力未提供。',
 gates=[gate('modern-human_ready-dpi','human','有限UI觀察的真人就緒時窗','modern-ui-dpi',True),
        gate('modern-native-geometry-dpi','supervisor/tool capability','可用原生截圖／輸入／DPI座標幾何能力','modern-ui-dpi')])
task('modern-background-scene-binding','6314／7482既有RGBA接正確場景',20,
 '僅在獨立場景owner/draw位址合格後接既有RGBA到該場景的產品繪製點；封存新source及必要位置/生命週期CPU檢查',
 '沿既有RGBA/IMG decoder，不重做轉檔；scene owner、draw position、layer/clip/lifetime源SHA合格，未把未知背景硬派Directory；CPU合格不等於GUI。',
 '6314/7482的scene owner、實際draw position與layer/clip/lifetime獨立來源仍null；RGBA表示法已具備。')
task('modern-owner-ready','獨立房主資格與owner-start typed入口',21,
 '在現485F Runtime增加來源合格的owner Control/失效生命週期，接既有Sender；owner C_D2來源字段與必要新CPU checks',
 'owning資格有獨立producer而非CE/D5名稱猜測；current epoch/context gate、失效清理及精確命令有證據；完整write不當ACK。',
 'current independent owner qualification producer、version/sourceSHA、觸發/失效/命令格式未提供；現在opcode-only C_D2拒絕。',supplier='bin_resources + server')
task('modern-world-clock','適用版本的WORLD/actor時計與同步契約',22,
 '同一合格session actor/scene時計適配與來源同步邊界；精確紀錄epoch/domain/unit/起點/截止/失效與Unknown策略',
 'source時間domain有對應actor/WORLD生命週期；不能由E1 BYTE、fixture absolute nanoTime、手動callback milestone推導即時時計；只測來源支持的新適配。',
 '同session可同步時計的producer、domain/unit/起點與actor/scene生命週期證據；5350 fixture時計不能當485F live時計。',supplier='server + bin_resources')
task('modern-mode-binding-consumer','未完成模式的source-qualified綁定增量消費',25,
 '對新到的一個或多個具體mode/map delta接既有typed binding入口，附來源namespace/版本/spawn/channel/mode資格及40表受影響行增量',
 '只處理新源delta，不重核40矩陣；Native邏輯map與Shared physical map不換名；每個變動有qualified sourceSHA，未到的24行保留Unknown。',
 '至少一個剩餘24 mode/map的新合格進場/地形namespace/spawn/版本/網路資格契約；靜態40plans不構成mode admission。',supplier='server + bin_resources')
task('modern-selected-admission','人物Work/owner與selected appearance所有權接線',30,
 '既有R429–439 slice/codec接實際選中人物Work、owner epoch/revision/cutoff/cancel的typed admission adapter',
 'loaded_admission_available只在獨立合格Work/owner/current-life成立；保留stale拒收與cleanup，現有inspection不當live admission。',
 'actual Work、owner epoch/revision/observation、cutoff與cancellation source bridge未提供。',
 evidence=base[:5]+list(extra.values())[:1])
task('modern-selected-renderer','真實selected actor與renderer session銜接',35,
 '將已審查selected admission接現有renderer擁有者，附actor資產/state/scene snapshot與退場新檢查',
 'frame producer、selected identity与owned renderer相同epoch/context；scene、effects、position不從synthetic0B/isolatedRGBA拼造；CPU不當畫面驗收。',
 '實際actor/scene/frame producer及render位置/效果與其owner生命週期；完整scene尚未合格。',deps=['modern-selected-admission'])
task('modern-world-lifecycle','正式WORLD與RESULT生命週期入口',40,
 '只在原攔截合法解除且同版本full-start/selected/store證據具備後接真WORLD/RESULT session ownership，交付新有限實際流程',
 '真實store constructor全路徑與selected/bookmarks/profile migration依賴合格，不skip/borrow/fake BATTLE；ROOM RUNNING、FD HUD或component PASS不足；停止與離場可核對。',
 '原server full-start store constructor達stopped auction initializer，selected identity/bookmarks/profile migration及conditional castle仍欠資格。',supplier='server + supervisor',policy=True,
 gates=[gate('modern-world-original-restriction-resolution','supervisor/original approval authority','原server停止領域限制的合法具體解決；一般繼續或workflow授權不足','modern-world-lifecycle'),
        gate('modern-world-version-qualified-start','server','適用正式consumer版本的真selected identity/完整store/full-start/WORLD/RESULT契約及停止證據','modern-world-lifecycle')],
 evidence=base)
task('modern-movement','既有idle49／movement接actual actor/global/collision',45,
 'R452及R459–472既有finite logic接合格actor/global/collision/scheduler input與Provider effects/error cleanup，僅新增缺口適配',
 'current owner/scene/clock/table/allocation來源合格；actual C_MOVE/位置由實際authoritative結果更新；既有reducer與路徑矩陣不重跑當新成果。',
 'actual actor/global/collision results、fresh original scheduler/table/allocation、Provider effects/error cleanup來源仍缺。',
 deps=['modern-selected-admission','modern-world-lifecycle','modern-world-clock'],evidence=base[:5]+list(extra.values()))
task('modern-roof-alias-timer','Native屋頂alias/FIFO/freed callback適配',46,
 '既有有限roof logic接實際alias/flags/FIFO/midfade與freed-group callback lifetime，保留來源時計domain',
 'Native564等controlled alias有actual來源見證；Shared interpolation不推定Native共享alias；free/cancel callback不跨owner或重用clock。',
 'controlled alias witness、original freed-group callback lifetime及對應原時計未提供。',deps=['modern-selected-admission'],evidence=base[:5]+list(extra.values()))
task('modern-layer8-event','Layer8 event原runtime queue/clock接線',47,
 '既有scheduler/FirePot38/combinedscene接同instance62F328/647E90/player pointers與queue/allocation lifecycle',
 '各原clock/table/queue與owned scene同生命期；首次/重排/移除只驗新增橋接，未提供general event自然admission不假造。',
 'runtime62F328/647E90/player pointers、queue/allocation及fresh original clocks未提供。',deps=['modern-selected-admission'],evidence=base[:5]+list(extra.values()))
task('modern-campaign','Campaign resource到實際callback擁有者',48,
 '五既有campaign table接同process初始化/free/cancel/retained4C32A0原效果consumer',
 '636584/64B6DC/647980/647E18/table同instance資格；R472 pending5000及callbacks不可略過；不重extract五表。',
 '同process636584/64B6DC/647980/647E18/table与初始化/free/cancel/retained4C32A0 object效果未提供。',deps=['modern-selected-admission'],evidence=base[:5]+list(extra.values()))
task('modern-vip11101-resource','11101真實proc動畫資產消費',50,
 '只在source-qualified11101 proc frames/member/action mapping到齊後接既有asset decoder/lookup；新pin與必要視覺CPU檢查',
 'Shared11101 empty placeholder不能當完成；來源frame/member/action/count/trigger合格，gfx0fixture不充當動畫，不借其他effect。',
 '合格11101 proc frames、archive member/action與觸發mapping；現有Shared11101空actionplaceholder。')
task('modern-vip-scene','VIP/status/equipment真場景呈現',55,
 '既有VIP/status/equipment資料與合格11101/frame接同selected renderer life的產品呈現',
 '實際proc trigger/actor/effect/lifetime合格；inspection與gfx0不是scene完成；針對新增拼接測試且GUI另驗。',
 '真實proc trigger/selected actor/effect life與frame來源，scene尚未完整。',supplier='server + bin_resources',
 deps=['modern-vip11101-resource','modern-selected-renderer'])
task('modern-original-room-delta','原版房間缺陷對Modern的必要語義delta消費',60,
 '只有新原版實際room問題證據涉及Modern packet/handler時接必要consumer修正，附版本/方向/trigger/field/handlerVA與未知',
 '原版GUI失敗不當ModernGUI失敗；已知corroboration不重報/重派server；無影響Modern語義delta即停止保留既有結論，不產生假fix。',
 '新獨立qualified原版room實際觀察及對Modern有影響的具體語義delta；歷史no-response本身不授權重開原版或T73。',supplier='launcher + bin_resources')

flow_ids={}
for row in scope['per_mode_map']:
    if row['existing_v47_flow_complete']:continue
    id=f"modern-mode-{row['mode']}-map-{row['map']}-full-flow";flow_ids[(row['mode'],row['map'])]=id
    pet=row['mode']==0 and row['map'] in (566,569,570)
    task(id,f"mode{row['mode']} {row['mode_name']} map{row['map']}完整模式綁定",100+row['mode'],
      f"該mode{row['mode']}/map{row['map']}單一合格source namespace/version的實際進場、規則、renderer、RESULT返回consumer與有限新驗證；每次只閉合本行",
      '同namespace/version/selected identity之mode/map/spawn/admission合格；真WORLD/actor/movement/clock/RESULT，非靜態preview/ROOM/component。16歷史PASS不重跑；新實際流程與GUI分別計數。'+
      ('R533–540 progression/terrain/session/result元件已存在；不重做3maps矩陣，5350 fixture不升格485F。' if pet else ''),
      f"mode{row['mode']}/map{row['map']}的合格進場/版本/規則/spawn契約與真WORLD producer；現有static plan不足。",
      supplier='server + bin_resources',deps=['modern-mode-binding-consumer','modern-world-lifecycle','modern-selected-renderer','modern-world-clock'],
      gates=[gate(id+'-mode-source','server + bin_resources',f"本行(mode{row['mode']},map{row['map']})自己的version/namespace/spawn/mode admission與RESULT來源，不借其他map",id)])

mapping=[]
for row in scope['per_mode_map']:
    r=dict(row);flow=flow_ids.get((row['mode'],row['map']))
    dims={
      'mode':{'task_ids':['modern-mode-binding-consumer']+([flow] if flow else []),'producer':'server + bin_resources','historical_v47_complete':row['existing_v47_flow_complete']},
      'renderer':{'task_ids':['modern-selected-admission','modern-selected-renderer'],'producer':'bin_resources actual Work/owner + server qualified actor/scene'},
      'resource':{'task_ids':['modern-mode-binding-consumer'],'producer':'bin_resources qualified Native static/spawn/resource namespace; static plans already present','conditional_tasks':['modern-vip11101-resource','modern-background-scene-binding'],'conditional_scene_applicability':'unknown; never assign 6314/7482 or VIP to this mode without source'},
      'UI':{'task_ids':['modern-ui-dpi','modern-gui-room'],'producer':'human + permitted native geometry/input; shared ROOM scope, not per-mode WORLD proof'},
      'owner':{'task_ids':['modern-owner-ready','modern-selected-admission'],'producer':'bin_resources + server independent command/Work owner producers'},
      'clock':{'task_ids':['modern-world-clock'],'producer':'server + bin_resources same-source version/domain/lifetime; fixture nanoTime/E1 not synchronization'},
      'WORLD_RESULT':{'task_ids':['modern-world-lifecycle']+([flow] if flow else []),'producer':'server legitimate full-start/selected/store/RESULT evidence'},
      'GUI':{'task_ids':['modern-gui-room','modern-ui-dpi'],'producer':'current human availability + native tool capability + fresh supervised supply; pending'},
    }
    r['workflow_dimensions']=dims
    r['task_ids']=sorted({t for d in dims.values() for t in d['task_ids']})
    r['full_mode_task_id']=flow
    r['baseline_scope']='historical v47 program flow only; Native full binding and GUI remain false even for the 16 historical rows'
    mapping.append(r)
category_ids={
 'formal_player_GUI':['modern-gui-room'],'TW_UI_visual_and_DPI':['modern-ui-dpi'],
 '40_mode_maps':['modern-mode-binding-consumer','modern-world-lifecycle']+list(flow_ids.values()),
 'VIP_11101':['modern-vip11101-resource','modern-vip-scene'],
 'Native_selected_appearance':['modern-selected-admission','modern-selected-renderer'],
 'Native_idle49_and_movement':['modern-movement'],'Native_roof_alias_state_timer':['modern-roof-alias-timer'],
 'Native_Layer8_event':['modern-layer8-event'],'campaign_callback':['modern-campaign'],
 'Native_PetRace_566_569_570':[flow_ids[(0,m)] for m in (566,569,570)],
 'Native_background_6314_7482':['modern-background-scene-binding'],
 'Native_WORLD_and_RESULT_return':['modern-world-lifecycle'],
 'old_Native_room_no_response':['modern-original-room-delta'],
 'screenshot_and_coordinate_geometry':['modern-ui-dpi','modern-gui-room'],
 'ASCII_fixture_directory':['modern-workflow-v2']}
categories=[{**x,'task_ids':category_ids[x['item']],
    'producer':'resolved historical permission question; no new product task' if x['item']=='ASCII_fixture_directory' else
        'See task blockers/gates for exact supplier and consumer',
    'stale_blocker_removed':x['item']=='ASCII_fixture_directory'} for x in scope['items']]
assert len(flow_ids)==24 and all(t['state']!='ready' for t in items)
minimal=[
 {'task_id':'modern-background-scene-binding','supplier':'bin_resources',
  'missing_input':'6314/7482獨立合格scene owner/draw position/layer/clip/lifetime來源；既有RGBA不用重轉',
  'resume_when':'新sourceSHA/版本/引用位置證明此scene mapping；來源取得不越過私有backing/Native工具停止界線',
  'consumer':'modern-background-scene-binding','blocked_world_or_provider_dependency':False},
 {'task_id':'modern-owner-ready','supplier':'bin_resources + server',
  'missing_input':'適用版本的independent current owner qualification/命令失效契約',
  'resume_when':'新獨立producer handoff有SHA、trigger、version与admission life','consumer':'modern-owner-ready','blocked_world_or_provider_dependency':False},
 {'task_id':'modern-ui-dpi','supplier':'human + supervisor/tool capability',
  'missing_input':'當下真人就緒和原允許native screenshot/input/DPI geometry',
  'resume_when':'兩個原有工具/人員gates合法有效且未過期；僅離線UI測量不需provider',
  'consumer':'modern-ui-dpi','blocked_world_or_provider_dependency':False}]
queue={'version':2,'owner':'modern','updated_at':NOW,'conversion_task_id':'modern-workflow-v2',
 'proposal_status':'review','execution_state':'waiting','ready_count':0,'next_eligible_task_id':None,
 'next_on_input_task_id':'modern-background-scene-binding','items':items,
 'mode_mapping':mapping,'backlog_category_mapping':categories,
 'coverage':{'historical_v47_program_complete':16,'remaining_v47_program_modes':24,'mode_map_rows':40,
             'new_full_native_bindings':0,'GUI_accepted':0,'LOADED':False,'overall_completed':False},
 'candidate_baseline':{'R542_root':base[0],'R542_EXE':progress['r542']['executable'],
    'actual_linked_R540_library':progress['r542']['linked_library'],
    'default_run':'LiToModern','default_main_unchanged':True,'CPU_CLI_only':True,'live_GUI_accepted':False},
 'source_evidence':base+list(extra.values()),'assignment_source':{'index':pin(INDEX),'task_id':assignment['id']},
 'minimal_unblocking_options':minimal,
 'review_policy':'queue proposal/review不是accepted產品。外部gates不是本清單dependencies；依賴僅本清單ID。原限制只能合法具體解除。缺失資料到達只啟用受影響task，不循環重寫此清單。',
 'independent_work_selection':'No new independent source-backed product implementation is ready. R542 routing/CLI gap was already completed; background decoder/RGBA and mode static plans exist. All remaining actual adapter changes lack qualified input or original permission/tool conditions.',
 'new_product_work_completed':False,'workflow_conversion_only':True}
qpin=save(Q,queue)
save(L/'modern-workflow-v2-selection.json',{'schema':'modern.workflow-v2.selection.v1','updated_at':NOW,
  'queue':qpin,'task_count':len(items),'ready_count':0,'execution_state':'waiting',
  'considered_independent_tasks':[{'id':x['task_id'],'missing_input':x['missing_input']} for x in minimal],
  'selected_product_task_id':None,'next_on_input':'modern-background-scene-binding',
  'reason':queue['independent_work_selection'],'no_new_implementation_or_old_tests':True})
print(json.dumps({'queue':qpin,'tasks':len(items),'mode_rows':40,'remaining_mode_tasks':24,'ready_count':0},ensure_ascii=False))
