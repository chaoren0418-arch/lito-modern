"""Derive pinned normal-build readers for source confirmation and 19 smoke cases."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,re
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(p.read_text('utf-8-sig'))
q=read(R/'工作佇列.json');app=q['current_native_initial_source_confirmation_application']
assert app['source_manifest_sha256']==q['current_product_identity']['source_manifest_sha256']
staged={};origins={}
for prefix in ['run','qualify','checkpoint']:
    name=prefix+'-native-accessibility-labels-normal.py';raw=(T/name).read_bytes();origins[name]=hashlib.sha256(raw).hexdigest().upper()
    body=raw.decode('utf-8-sig').replace('\r\n','\n').replace('native-accessibility-labels-normal','native-initial-source-confirmation-normal').replace('accessibility_labels','initial_source_confirmation').replace('accessibility-labels','initial-source-confirmation')
    if prefix in ['run','qualify']:
        needle="'--test','native_teleport','--test','native_services'"
        assert body.count(needle)==1
        body=body.replace(needle,needle+",'--test','native_login'")
        body=body.replace("{'native_teleport','native_services'}","{'native_teleport','native_services','native_login'}")
        body=body.replace('len(artifacts)==2','len(artifacts)==3')
    if prefix=='qualify':
        start=body.index("proofs=application['original_GUI_label_proofs'];")
        end=body.index("old=read(app/'before-compiled-inputs.json');",start)
        body=body[:start]+"failure=application['original_GUI_source_handoff_failure']\nassert failure['source_manifest_sha256']==application['before_source_manifest_sha256'] and not failure['qualification_granted']\nassert sha(failure['evidence'])==failure['evidence_sha256'] and failure['selection_to_GAME_version_seconds']>30\ncontract=failure['current_protocol_C8_contract'];assert sha(contract['method'])==contract['sha256'] and contract['live_GAME_C8_rebind_supported_by_bytecode']\n"+body[end:]
        body=body.replace("['LiTo381-Modern/src/modern_lito/native_login_window.rs']","['LiTo381-Modern/src/modern_lito/native_login.rs']")
        body=body.replace("total=14 if name=='native_teleport'else 2","total={'native_teleport':14,'native_services':2,'native_login':3}[name]")
        body=body.replace("len(c['process_history'])==8","len(c['process_history'])==10")
        body=body.replace('passed=16','passed=19').replace('passed_per_profile=16','passed_per_profile=19').replace('existing_protocol_smoke_tests_per_profile=16','existing_protocol_smoke_tests_per_profile=19')
        body=body.replace('original_GUI_label_proof_count=2','original_GUI_source_failure_count=1')
        body=body.replace('normal_focus_GUI_integration_verified=False','initial_GAME_source_confirmation_runtime_verified=False')
    if prefix=='checkpoint':
        body=body.replace("q['passed_per_profile']==16","q['passed_per_profile']==19").replace('每版16相關測試','每版19相關測試').replace('既有services／teleport十六案為冒煙回歸；技能標籤正常GUI整合另驗。','既有login3／services2／teleport14為冒煙回歸；新C8實際來源確認仍待。').replace('catalog_related16_each_qualified','catalog_related19_each_qualified')
        work="work='來源'+c['source_manifest_sha256'][:8]+'初次GAME來源確認修正版原'+str(a.session)+'：'+stage+'只改native_login.rs，先以公有GAME C8確認清單再開放選角；租裝目的地／恢復自動接續另驗。當版完整library、自然超30秒與快速交接GUI、傳送／恢復及廣域回歸尚待。4F2F兩版全庫與資源、Debug限定GUI只保留原版；Release49194原始來源失敗保留。自行完整測試後才交人工，不改JAR或30秒／8秒／60秒，不請使用者操作。'"
        body,n=re.subn(r'^work=.*$',lambda _:work,body,count=1,flags=re.M);assert n==1
    assert 'original_GUI_label_proofs'not in body
    ast.parse(body)
    target=name.replace('native-accessibility-labels-normal','native-initial-source-confirmation-normal');assert not(T/target).exists()
    staged[target]=body.encode('utf-8')
out=V/'native-initial-source-confirmation-tooling'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,payload in staged.items():(T/name).write_bytes(payload);(out/name).write_bytes(payload)
record=dict(source_manifest_sha256=app['source_manifest_sha256'],templates=origins,tools={n:hashlib.sha256(b).hexdigest().upper()for n,b in staged.items()},expected_smoke_cases_per_profile=19,new_behavior_runtime_not_claimed=True,runtime_started=False)
(out/'normal-tools.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),tools=len(staged),expected_smoke_cases_per_profile=19),ensure_ascii=False))
