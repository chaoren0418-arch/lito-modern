"""Prepare existing six-branch controller regression for current source confirmation."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,re
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
w=json.loads((R/'工作佇列.json').read_text('utf-8-sig'));source=w['current_product_identity']['source_manifest_sha256']
assert source==w['current_native_initial_source_confirmation_application']['source_manifest_sha256']
names=['run-native-keyboard-focus-controller.py','qualify-native-keyboard-focus-controller-case.py','qualify-native-keyboard-focus-controller-cohort.py','observe-native-keyboard-focus-controller-completion.py','checkpoint-native-keyboard-focus-controller-active.py','checkpoint-native-keyboard-focus-controller-complete.py']
staged={};origins={}
def once(body,old,new):
    assert body.count(old)==1,(old[:100],body.count(old));return body.replace(old,new)
for name in names:
    raw=(T/name).read_bytes();origins[name]=hashlib.sha256(raw).hexdigest().upper()
    body=raw.decode('utf-8-sig').replace('\r\n','\n').replace('native-keyboard-focus','native-initial-source-confirmation').replace('keyboard_focus','initial_source_confirmation')
    body=body.replace("['passed_per_profile']==16","['passed_per_profile']==19").replace('passed_each=16','passed_each=19')
    if name.startswith('run-'):
        body=once(body,"len(c['process_history'])==8","len(c['process_history'])==10")
        body=once(body,"set(info['targets'])=={'native_services','native_teleport'}","set(info['targets'])=={'native_login','native_services','native_teleport'}")
        body=once(body,"=={'native_services':{'passed':2,'failed':0,'ignored':0},'native_teleport':{'passed':14,'failed':0,'ignored':0}}","=={'native_login':{'passed':3,'failed':0,'ignored':0},'native_services':{'passed':2,'failed':0,'ignored':0},'native_teleport':{'passed':14,'failed':0,'ignored':0}}")
    if name=='qualify-native-keyboard-focus-controller-case.py':
        anchor="assert sum(f['plain'] == 'MASKED_C_LOGIN' for f in frames) == expected_sessions"
        body=once(body,anchor,anchor+"\nsource_requests=[bytes.fromhex(f['plain'])for f in frames if f['direction']=='client_to_server'and f['plain']!='MASKED_C_LOGIN'and bytes.fromhex(f['plain'])[0]==0xc8]\nassert len(source_requests)>=4 and all(b[1:].rstrip(b'\\0').decode('cp950')=='測試乙'for b in source_requests)\nif cross:assert len(source_requests)==4")
        anchor="    original_owned_processes_terminal=True, inventory_all_fields_from_original_S7C=checks,"
        body=once(body,anchor,anchor+"\n    public_source_C8_requests_observed=len(source_requests),initial_source_confirmation_frames_present=True,")
    if name.startswith('checkpoint-'):
        active=name.endswith('active.py')
        scope='原session執行中，兩版六分支待實際結果及逐案核對。'if active else'兩版六分支12案／96sessions已獨立核對，保留完整物品接續與原C9拒重播斷言。'
        work="work='來源"+source[:8]+"原'+str(a.session_id)+'："+scope+"初次GAME C8來源確認、自然超30秒GUI及其他廣域／服務從各自當版資格讀回；同來源已核對library不重跑或移除。4F2F Release49194到期來源錯誤保留。完整GUI／G01–G12仍待，不請使用者操作，相鄰只讀，30秒／8秒／60秒與Release／G11預算不變。'"
        body,n=re.subn(r'^work=.*$',lambda _:work,body,count=1,flags=re.M);assert n==1
        if active:
            body=once(body,"for n in before:\n    path=ROOT/n","staged={}\nfor n in before:\n    path=ROOT/n")
            body=once(body,'        save(path,value)',"        staged[n]=(json.dumps(value,ensure_ascii=False,indent=2)+'\\n').encode('utf-8')")
            body=once(body,"path.write_text((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\\n\\n'+rest,encoding='utf-8',newline='\\n')","staged[n]=((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\\n\\n'+rest).encode('utf-8')")
            anchor="save(out/'readback.json',dict(before_files=before,after_files="
            body=once(body,anchor,"assert len(staged)==8 and all(sha(ROOT/n)==h for n,h in before.items())\nfor n,payload in staged.items():(ROOT/n).write_bytes(payload)\n"+anchor)
        else:
            body=once(body,"date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')","date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d');staged={}")
            body=once(body,'        save(p,v)',"        staged[n]=(json.dumps(v,ensure_ascii=False,indent=2)+'\\n').encode('utf-8')")
            body=once(body,"p.write_text(date+'：'+work+'\\n\\n'+rest,encoding='utf-8',newline='\\n')","staged[n]=(date+'：'+work+'\\n\\n'+rest).encode('utf-8')")
            anchor="save(out/'readback.json',dict(before_files=before,after_files="
            body=once(body,anchor,"assert len(staged)==8 and all(sha(R/n)==h for n,h in before.items())\nfor n,payload in staged.items():(R/n).write_bytes(payload)\n"+anchor)
            body=body.replace('current-source broad/library remain pending.','Retain matching-source independently qualified library; broad and GUI have separate scope.')
    ast.parse(body);target=name.replace('native-keyboard-focus','native-initial-source-confirmation');assert not(T/target).exists();staged[target]=body.encode('utf-8')
out=V/'native-initial-source-confirmation-tooling'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for n,b in staged.items():(T/n).write_bytes(b);(out/n).write_bytes(b)
record=dict(source_manifest_sha256=source,templates=origins,tools={n:hashlib.sha256(b).hexdigest().upper()for n,b in staged.items()},normal_smoke_cases_per_profile=19,controller_scenarios_unchanged=6,original_inventory_and_C9_assertions_retained=True,checkpoints_stage_all_eight_before_writing=True,controller_runtime_started=False)
(out/'controller-tools.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),tools=len(staged),runtime_started=False),ensure_ascii=False))
