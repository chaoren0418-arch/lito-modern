"""Derive a separate normal-build QA chain without changing historical tools."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,re
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
names=['run-native-keyboard-focus-normal.py','qualify-native-keyboard-focus-normal.py','checkpoint-native-keyboard-focus-normal.py']
staged={};origins={}
for name in names:
    raw=(T/name).read_bytes();origins[name]=hashlib.sha256(raw).hexdigest().upper()
    body=raw.decode('utf-8-sig').replace('\r\n','\n').replace('native-keyboard-focus-normal','native-accessibility-labels-normal').replace('keyboard_focus','accessibility_labels').replace('keyboard-focus','accessibility-labels')
    if name.startswith('qualify-'):
        old="proof=Path(application['isolated_focus_API_proof']);assert sha(proof)==application['isolated_focus_API_proof_sha256'];probe=read(proof);assert probe['status']=='isolated_owned_control_focus_loss_reproduced_candidate_API_checks_passed'and probe['normal_UI_source_sha256']==read(app/'before-compiled-inputs.json')['LiTo381-Modern/src/modern_lito/native_login_window.rs']"
        new="proofs=application['original_GUI_label_proofs'];assert len(proofs)==2\nfor proof in proofs:\n    assert sha(proof['qualification'])==proof['qualification_sha256'] and sha(proof['events'])==proof['events_sha256']\n    assert read(proof['qualification'])['source_manifest_sha256']==application['before_source_manifest_sha256']\n    assert any('新密碼'in event['tree']and'ID: 208'in event['tree']for event in read(proof['events']))"
        assert body.count(old)==1;body=body.replace(old,new)
        body=body.replace('isolated_owned_focus_API_conditions=4','isolated_owned_focus_API_conditions=0,original_GUI_label_proof_count=2')
    if name.startswith('checkpoint-'):
        work="work='來源'+c['source_manifest_sha256'][:8]+'正常GUI技能標籤修正版原'+str(a.session)+'：'+stage+'只改一個正常視窗來源檔；原始錯誤名稱已有兩版Sky證據。當版完整library與GUI名稱／巡覽／面板切換核對尚待；AB3C廣域、全庫及限定GUI成果僅保留原版資格。繼續剩餘開發，自行完整測試後才交人工，不請使用者登入，相鄰只讀，G01–G12未完成。'"
        body,n=re.subn(r'^work=.*$',lambda _:work,body,count=1,flags=re.M);assert n==1
        body=body.replace('焦點正常GUI整合另驗。','技能標籤正常GUI整合另驗。')
        body=body.replace("date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')","date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d');staged={}")
        assert body.count('        save(path,value)')==1
        body=body.replace('        save(path,value)',"        staged[n]=(json.dumps(value,ensure_ascii=False,indent=2)+'\\n').encode('utf-8')")
        old="path.write_text(date+'：'+work+'\\n\\n'+rest,encoding='utf-8',newline='\\n')"
        assert body.count(old)==1;body=body.replace(old,"staged[n]=(date+'：'+work+'\\n\\n'+rest).encode('utf-8')")
        anchor="save(out/'readback.json',dict(before_files=before,after_files="
        assert body.count(anchor)==1
        body=body.replace(anchor,"assert len(staged)==8 and all(sha(ROOT/n)==h for n,h in before.items())\nfor n,payload in staged.items():(ROOT/n).write_bytes(payload)\n"+anchor)
    ast.parse(body);target=name.replace('native-keyboard-focus-normal','native-accessibility-labels-normal');assert not(T/target).exists();staged[target]=body.encode('utf-8')
out=V/'native-accessibility-labels-tooling'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for n,content in staged.items():(T/n).write_bytes(content);(out/n).write_bytes(content)
receipt=dict(templates=origins,tools={n:hashlib.sha256(content).hexdigest().upper()for n,content in staged.items()},runtime_started=False)
(out/'normal-tools.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),tools=len(staged),runtime_started=False),ensure_ascii=False))
