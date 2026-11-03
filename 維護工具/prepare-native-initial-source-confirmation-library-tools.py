"""Pin an unchanged default library selection to the current 19-case normal baseline."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,re
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
q=json.loads((R/'工作佇列.json').read_text('utf-8-sig'));source=q['current_product_identity']['source_manifest_sha256']
assert source==q['current_native_initial_source_confirmation_application']['source_manifest_sha256']
old='4F2F911B4B70BDA16C4EE8C3EDD60223F3E72A22C8A3B1B808C3F5EABA60E6A2'
names=['run-native-accessibility-labels-library-default.py','qualify-native-accessibility-labels-library-default.py','observe-native-accessibility-labels-library-default.py','checkpoint-native-accessibility-labels-library-active.py','checkpoint-native-accessibility-labels-library-completed.py']
staged={};origins={}
for name in names:
    raw=(T/name).read_bytes();origins[name]=hashlib.sha256(raw).hexdigest().upper()
    body=raw.decode('utf-8-sig').replace('\r\n','\n').replace('native-accessibility-labels','native-initial-source-confirmation').replace('accessibility_labels','initial_source_confirmation').replace(old,source)
    body=body.replace("['passed_per_profile']==16","['passed_per_profile']==19").replace("nq['passed']==16","nq['passed']==19")
    needle="assert set(nq['targets'])=={'native_services','native_teleport'} and nq['targets']['native_services']['counts']['passed']==2 and nq['targets']['native_teleport']['counts']['passed']==14"
    if name.startswith('qualify-'):
        assert body.count(needle)==1
        body=body.replace(needle,"assert set(nq['targets'])=={'native_login','native_services','native_teleport'} and nq['targets']['native_login']['counts']['passed']==3 and nq['targets']['native_services']['counts']['passed']==2 and nq['targets']['native_teleport']['counts']['passed']==14")
    if name.startswith('checkpoint-'):
        session='args.session_id'if name.endswith('active.py')else'a.session'
        stage='當版正常兩版及相關19案已核對，完整library沿原程序執行、逐名稱與結果及終止待核對。'if name.endswith('active.py')else'當版兩版完整library預設已獨立核對，忽略本文未執行。'
        work="work='來源"+source[:8]+"原'+str("+session+")+'："+stage+"初次GAME C8來源確認之自然超30秒／快速GUI與租裝目的地／恢復、廣域／控制器仍待；4F2F成果與Release49194來源到期失敗各保留原版。自行完整測試後才交人工，不請使用者操作；相鄰只讀，30秒來源／旅館數量、8秒／60秒與Release／G11預算不變。'"
        body,n=re.subn(r'^work=.*$',lambda _:work,body,count=1,flags=re.M);assert n==1
    ast.parse(body);target=name.replace('native-accessibility-labels','native-initial-source-confirmation');assert not(T/target).exists();staged[target]=body.encode('utf-8')
out=V/'native-initial-source-confirmation-tooling'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for n,b in staged.items():(T/n).write_bytes(b);(out/n).write_bytes(b)
record=dict(source_manifest_sha256=source,templates=origins,tools={n:hashlib.sha256(b).hexdigest().upper()for n,b in staged.items()},normal_smoke_cases_per_profile=19,default_library_selection_unchanged=True,ignored_bodies_enabled=False,new_broad_controller_or_GUI_qualification_granted=False,runtime_started=False)
(out/'library-tools.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),tools=len(staged),runtime_started=False),ensure_ascii=False))
