"""Port exact normal-build readers to the one-file focus repair, with truthful scope."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,re
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
out=V/'native-keyboard-focus-normal'/('tooling-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
mapping={'run-native-same-endpoint-catalog-normal.py':'run-native-keyboard-focus-normal.py',
         'qualify-native-catalog-normal.py':'qualify-native-keyboard-focus-normal.py',
         'checkpoint-native-catalog-normal-work.py':'checkpoint-native-keyboard-focus-normal.py'}
def sha(b):return hashlib.sha256(b).hexdigest().upper()
staged={};origins={}
for source,target in mapping.items():
    original=(T/source).read_bytes();origins[source]=sha(original);body=original.decode('utf-8-sig').replace('\r\n','\n')
    body=body.replace('native-same-endpoint-catalog-normal','native-keyboard-focus-normal').replace('same_endpoint_catalog','keyboard_focus').replace('same-endpoint-catalog','keyboard-focus')
    body=body.replace('current_native_catalog_normal_qualification','current_native_keyboard_focus_normal_qualification').replace('current_native_catalog_normal_work','current_native_keyboard_focus_normal_work')
    if source=='qualify-native-catalog-normal.py':
        before="and len(application['changed_files'])==3"
        assert body.count(before)==1
        body=body.replace(before,"and application['changed_files']==['LiTo381-Modern/src/modern_lito/native_login_window.rs']")
        body=body.replace('new_behavior_tests=6','new_behavior_tests=0,existing_protocol_smoke_tests_per_profile=16,isolated_owned_focus_API_conditions=4,normal_focus_GUI_integration_verified=False')
        body=body.replace('keyboard_focus_rebinds_only_matching_routes_once_after_fresh_game_character','same_endpoint_catalog_rebinds_only_matching_routes_once_after_fresh_game_character')
        marker="old=read(app/'before-compiled-inputs.json');"
        assert body.count(marker)==1
        check="proof=Path(application['isolated_focus_API_proof']);assert sha(proof)==application['isolated_focus_API_proof_sha256'];probe=read(proof);assert probe['status']=='isolated_owned_control_focus_loss_reproduced_candidate_API_checks_passed'and probe['normal_UI_source_sha256']==read(app/'before-compiled-inputs.json')['LiTo381-Modern/src/modern_lito/native_login_window.rs']\n"
        body=body.replace(marker,check+marker)
    if source=='checkpoint-native-catalog-normal-work.py':
        body=body.replace('六個新目錄生命週期案例已跑過；只通過相關範圍。','既有services／teleport十六案為冒煙回歸；焦點正常GUI整合另驗。')
        pattern=r"^work='來源'.*$"
        replacement="work='來源'+c['source_manifest_sha256'][:8]+'正常GUI鍵盤焦點修正版原'+str(a.session)+'：'+stage+'只改native_login_window.rs；隔離API原失焦與四條件已核對，正常GUI完整流程與Sky擷取仍待。2DAF廣域544／540、兩版library各1969過0敗484忽略及236服務sessions保留原版本資格；修正版廣域／完整library與必要GUI須重新建立資格。開發期間不請使用者登入，相鄰只讀，G01–G12未完成。'"
        body,n=re.subn(pattern,lambda _:replacement,body,count=1,flags=re.MULTILINE);assert n==1
        body=body.replace("item='keyboard-focus-normal-build-and-related-tests'","item='keyboard-focus-normal-build-and-existing-protocol-smoke-tests'")
        body=body.replace("then fresh same-endpoint cases using current75 staged-rental inventory semantics.","then final-source broad/library and focused normal GUI keyboard regression; preserve prior-source qualifications and capture failures separately.")
    ast.parse(body)
    staged[target]=body.encode('utf-8')
for source,h in origins.items():assert sha((T/source).read_bytes())==h
for name,payload in staged.items():
    assert not (T/name).exists(),'Do not overwrite retained tooling'
    (T/name).write_bytes(payload);(out/name).write_bytes(payload)
receipt=dict(status='keyboard_focus_normal_tools_prepared',origins=origins,tools={n:sha(b)for n,b in staged.items()},
    profiles=['Debug','Release'],existing_smoke_targets=['native_services','native_teleport'],passed_expected_per_profile=16,
    new_focus_GUI_qualification_inferred=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),tools=receipt['tools']),ensure_ascii=False))
