"""Prepare full default library after normal builds; no broad-regression promotion."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,re
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
OLD='AB3CF26C4227A3E9E171AEA2E7765C1BA29120681BCB226B36FC53BA2635AE2D'
NEW='4F2F911B4B70BDA16C4EE8C3EDD60223F3E72A22C8A3B1B808C3F5EABA60E6A2'
names=['run-native-keyboard-focus-library-default.py','qualify-native-keyboard-focus-library-default.py','observe-native-keyboard-focus-library-default.py','checkpoint-native-keyboard-focus-library-active.py','checkpoint-native-keyboard-focus-library-completed.py']
staged={};origins={}
for name in names:
    original=(T/name).read_bytes();origins[name]=hashlib.sha256(original).hexdigest().upper()
    body=original.decode('utf-8-sig').replace('\r\n','\n').replace(OLD,NEW)
    body=body.replace('native-keyboard-focus-library','native-accessibility-labels-library').replace('keyboard_focus_normal','accessibility_labels_normal')
    if name.startswith('run-'):
        body=body.replace("choices=['default116','retained-archive-related','retained-geometry-related'], default='default116'","choices=['normal-related'], default='normal-related'")
        start=body.index("terminal = read(base / 'completion-observation.json')")
        end=body.index("identity=Path(normal['application'])")
        replacement="""assert a.input_kind=='normal-related'
queue=read(ROOT/'工作佇列.json');assert not queue.get('active_workloads') and not queue.get('source_and_corresponding_QA_frozen')
assert base.is_relative_to(ROOT/'驗證紀錄/current/native-accessibility-labels-normal')
npath=base/'qualified/qualified.json';n=read(npath);normal_build=base;normal=b
assert b['status']=='executed_accessibility_labels_normal_related_both_profiles'and b['owned_process']is None
assert n['cohort_sha256']==sha(base/'cohort.json')and n['recorded_owned_processes_all_terminal']and n['observed_terminal_exit_code']==0
"""
        body=body[:start]+replacement+body[end:]
        body=body.replace("sha(base / 'completion-observation.json')","sha(npath)")
        body=body.replace("broad_default_integration_regression_claimed=a.input_kind == 'default116'","broad_default_integration_regression_claimed=False")
    elif name.startswith('qualify-'):
        body=body.replace("sha(base/'completion-observation.json')", "sha(base/'qualified/qualified.json')").replace("assert c['input_kind']=='default116'","assert c['input_kind']=='normal-related'and not c['broad_default_integration_regression_claimed']")
        old="broad=read(base/'completion-observation.json');assert broad['status']=='independently_qualified_current1047_default116_both_profiles_and_terminal_processes' and broad['recorded_original_owned_processes_all_terminal']"
        assert body.count(old)==1
        body=body.replace(old,"assert base.resolve()==normal_root.resolve()and normal['cohort_sha256']==sha(base/'cohort.json')")
    elif 'active' in name:
        body=body.replace('after exact-source broad was sealed.','after exact-source normal builds were independently sealed.')
        body=body.replace("qualified = read(broad/'completion-observation.json')","qualified = read(broad/'qualified/qualified.json')")
        body=body.replace("qualified['recorded_original_owned_processes_all_terminal']","qualified['recorded_owned_processes_all_terminal']")
        old="before = read(read((run if args.refresh else broad).parent/'current-continuation.json')['latest_readback'])['after_files']"
        new="before = read(Path(read(ROOT/'工作佇列.json')['last_goal_turn_evidence']).with_name('readback.json'))['after_files']"
        assert body.count(old)==1;body=body.replace(old,new)
    elif 'completed' in name:
        body=body.replace("broad=read(Path(c['input_cohort'])/'completion-observation.json')","broad=read(Path(c['normal_qualification']))")
        body=body.replace("broad['recorded_original_owned_processes_all_terminal']","broad['recorded_owned_processes_all_terminal']")
        body=body.replace('current_catalog_broad_and_full_library_default_both_qualified_other_acceptance_pending','current_full_library_default_both_qualified_broad_not_repeated_GUI_pending')
        body=body.replace("record=dict(status=", "record=dict(broad_default_integration_regression_claimed=False,status=")
    if name.startswith('checkpoint-'):
        sid='args.session_id'if'active'in name else'a.session'
        status="'正常兩版及相關16案已核對；完整library沿原程序執行，逐名稱、忽略、映像、raw與程序終止待核對。'"if'active'in name else"'當版完整library兩版已獨立核對，忽略本文未執行。'"
        work="work='來源4F2F911B原'+str("+sid+")+'：'+"+status+"+'AB3C廣域、控制器、資源及限定GUI只保留原版本資格，不藉此宣告重驗；修正版正常技能名稱與面板／巡覽、其餘GUI與G01–G12仍待。全部必要自測後才交人工，不請使用者登入，相鄰只讀，8秒／60秒、30秒數量有效期與Release／G11預算不變。'"
        body,n=re.subn(r'^work\s*=.*$',lambda _:work,body,count=1,flags=re.M);assert n==1
        if'active'in name:
            body=body.replace('for name in before:\n    path = ROOT/name','staged={}\nfor name in before:\n    path = ROOT/name')
            assert body.count('        save(path, value)')==1
            body=body.replace('        save(path, value)',"        staged[name]=(json.dumps(value,ensure_ascii=False,indent=2)+'\\n').encode('utf-8')")
            old="path.write_text((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\\n\\n'+rest, encoding='utf-8', newline='\\n')"
            assert body.count(old)==1;body=body.replace(old,"staged[name]=((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\\n\\n'+rest).encode('utf-8')")
            anchor="save(out/'readback.json', dict(before_files=before, after_files="
            assert body.count(anchor)==1;body=body.replace(anchor,"assert len(staged)==8 and all(sha(ROOT/name)==h for name,h in before.items())\nfor name,payload in staged.items():(ROOT/name).write_bytes(payload)\n"+anchor)
        else:
            body=body.replace('for n in before:\n    path=ROOT/n','staged={}\nfor n in before:\n    path=ROOT/n')
            assert body.count('        save(path,j)')==1
            body=body.replace('        save(path,j)',"        staged[n]=(json.dumps(j,ensure_ascii=False,indent=2)+'\\n').encode('utf-8')")
            old="path.write_text((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\\n\\n'+rest,encoding='utf-8',newline='\\n')"
            assert body.count(old)==1;body=body.replace(old,"staged[n]=((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\\n\\n'+rest).encode('utf-8')")
            anchor="(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json'"
            assert body.count(anchor)==1;body=body.replace(anchor,"assert len(staged)==8 and all(sha(ROOT/n)==h for n,h in before.items())\nfor n,payload in staged.items():(ROOT/n).write_bytes(payload)\n"+anchor)
    ast.parse(body);target=name.replace('native-keyboard-focus-library','native-accessibility-labels-library');assert not(T/target).exists();staged[target]=body.encode('utf-8')
out=V/'native-accessibility-labels-tooling'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for n,payload in staged.items():(T/n).write_bytes(payload);(out/n).write_bytes(payload)
receipt=dict(source_manifest_sha256=NEW,templates=origins,tools={n:hashlib.sha256(payload).hexdigest().upper()for n,payload in staged.items()},input_kind='normal-related',full_default_library_unchanged=True,inventory_or_selector_or_assertions_relaxed=False,broad_requalified=False,runtime_started=False)
(out/'library-tools.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),tools=len(staged),input_kind='normal-related',broad_requalified=False,runtime_started=False),ensure_ascii=False))
