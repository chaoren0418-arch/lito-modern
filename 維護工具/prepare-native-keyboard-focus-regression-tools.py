"""Prepare strict same-source regression tools; retain all prior-source tools."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,re
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
OLD='2DAF4DE08EF396B1DE5B2264820E689859BFCD837015D71A6D74643A2DDEE9DE'
NEW='AB3CF26C4227A3E9E171AEA2E7765C1BA29120681BCB226B36FC53BA2635AE2D'
names=['run-native-catalog-controller.py','qualify-native-catalog-controller-case.py','qualify-native-catalog-controller-cohort.py',
       'observe-native-catalog-controller-completion.py','checkpoint-native-catalog-controller-active.py','checkpoint-native-catalog-controller-complete.py',
       'run-native-catalog-default116-both.py','run-native-catalog-default116.py','qualify-native-catalog-default116.py',
       'native_catalog_broad_gate.py','observe-native-catalog-default116-completion.py','checkpoint-native-catalog-default116-active.py',
       'checkpoint-native-catalog-default116-progress.py','checkpoint-native-catalog-default116-completed.py',
       'run-native-catalog-library-default.py','qualify-native-catalog-library-default.py','observe-native-catalog-library-default.py',
       'checkpoint-native-catalog-library-active.py','checkpoint-native-catalog-library-completed.py',
       'prepare-native-catalog-sky-gui.py','start-native-catalog-sky-gui-case.py','checkpoint-native-catalog-sky-gui.py']
def rename(s):
    return(s.replace('native-catalog-controller','native-keyboard-focus-controller')
             .replace('native-catalog-default116','native-keyboard-focus-default116')
             .replace('native_catalog_broad_gate','native_keyboard_focus_broad_gate')
             .replace('native-catalog-library-default','native-keyboard-focus-library-default')
             .replace('native-catalog-library-active','native-keyboard-focus-library-active')
             .replace('native-catalog-library-completed','native-keyboard-focus-library-completed')
             .replace('native-catalog-sky-gui','native-keyboard-focus-sky-gui'))
staged={};origins={}
note='修正版所有實測資格均須依當版來源與實際產物建立；2DAF同端點12、音效96及旅館32sessions只保留原版資格。正常GUI畫面擷取仍逾時，完整GUI、裝置、其他G01–G12未完成；先自行驗證才交人工，不請使用者登入，相鄰只讀，8秒／60秒與Release／G11預算不變。'
for name in names:
    original=(T/name).read_bytes();origins[name]=hashlib.sha256(original).hexdigest().upper()
    body=original.decode('utf-8-sig').replace('\r\n','\n');body=rename(body)
    body=body.replace('native-same-endpoint-catalog-normal','native-keyboard-focus-normal').replace('same_endpoint_catalog_normal','keyboard_focus_normal')
    body=body.replace(OLD,NEW)
    if name=='checkpoint-native-catalog-controller-complete.py':
        before="scope=dict(state='current_source_controller_and_same_endpoint_rental_qualified_Presentation_audio_other_services_GUI_pending',complete=False,current_controller_evidence=record,current_same_endpoint_evidence=v['current_native_catalog_handoff_qualification'])"
        after="scope=dict(state='current_source_controller_qualified_same_endpoint_Presentation_audio_other_services_GUI_pending',complete=False,current_controller_evidence=record,prior_source_same_endpoint_evidence=v['current_native_catalog_handoff_qualification'],current_source_same_endpoint_qualification_granted=False)"
        assert body.count(before)==1;body=body.replace(before,after)
    if name.startswith('checkpoint-') and 'sky-gui' not in name:
        sid='args.session_id'if name=='checkpoint-native-catalog-library-active.py'else'a.session_id'if'controller'in name else'a.session'
        if 'controller-active'in name:
            message="'修正版當版正常兩版及每版16既有協定冒煙已核對；當版G06六分支待原session實際完成及獨立逐案核對。'"
        elif 'controller-complete'in name:
            message="'當版控制器兩版六分支12案／96sessions已actual exit0與逐案、來源、映像及程序身分核對；廣域與完整library接續。'"
        elif 'default116-progress'in name:
            message="'廣域兩版仍執行中；已核對的profile：'+text+'，未推廣成整批通過；待原session終止。'"
        elif 'default116-active'in name:
            message="'當版正常兩版及控制器96已核對；廣域116整合目標及main兩版仍執行中，未給予整批資格。'"
        elif 'default116-completed'in name:
            message="'當版廣域兩版實際完成並獨立核對：'+counts+'；完整library接續，忽略本文未執行。'"
        elif 'library-active'in name:
            message="'當版廣域兩版已核對；完整library沿原程序執行，待逐名稱、忽略來源、產物、raw與程序終止核對。'"
        else:
            message="'當版完整library兩版已actual exit0及獨立核對：Debug'+str(profiles['Debug']['passed'])+'／Release'+str(profiles['Release']['passed'])+'過，各0敗；忽略本文未執行。'"
        replacement="work='來源AB3CF26C原'+str("+sid+")+'：'+"+message+"+'"+note+"'"
        body,count=re.subn(r'^work\s*=.*$',lambda _:replacement,body,count=1,flags=re.MULTILINE);assert count==1,name
    ast.parse(body);target=rename(name);assert target!=name
    assert not (T/target).exists(),'Retain previously prepared tools: '+target
    staged[target]=body.encode('utf-8')
out=V/'native-keyboard-focus-regression-tooling'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for n,h in origins.items():assert hashlib.sha256((T/n).read_bytes()).hexdigest().upper()==h
for n,payload in staged.items():(T/n).write_bytes(payload);(out/n).write_bytes(payload)
receipt=dict(status='strict_current_AB3C_regression_tools_prepared_not_executed',source_manifest_sha256=NEW,
    templates=origins,tools={n:hashlib.sha256(p).hexdigest().upper()for n,p in staged.items()},
    actual_current_controller_required_before_broad=True,old_G06_qualification_not_transferred=True,
    profiles=['Debug','Release'],source_files=1047,compiled_input_files=1048,normal_GUI_qualified=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),tool_count=len(staged),new_controller_required=True),ensure_ascii=False))
