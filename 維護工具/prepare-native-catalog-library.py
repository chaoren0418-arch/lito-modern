"""Stage the existing full-library checks against the genuine current normal/broad chain."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
names=['run-native33A3-library-default.py','qualify-native33A3-library-default.py','observe-native33A3-library-default.py','checkpoint-native33A3-library-active.py','checkpoint-native33A3-library-completed.py'];mapping={n:n.replace('native33A3','native-catalog')for n in names}
out=V/'native-catalog-library-default'/('tooling-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);pins={}
def rep(s,a,b):assert s.count(a)==1,(a[:90],s.count(a));return s.replace(a,b)
for oldname,newname in mapping.items():
    original=(T/oldname).read_text('utf-8-sig');s=original
    for a,b in mapping.items():s=s.replace(a,b)
    s=s.replace('native33A3-library-default','native-catalog-library-default').replace('native33A3-default116','native-catalog-default116').replace('current_native33A3_library','current_native_catalog_library').replace('current33A3','current_catalog')
    s=s.replace('33A3CCF73EFA564FD5AFABB0024C26911ED49D8F67F8E1E2D8072CA5CF95EB21','2DAF4DE08EF396B1DE5B2264820E689859BFCD837015D71A6D74643A2DDEE9DE')
    if oldname=='run-native33A3-library-default.py':
        s=rep(s,"n['entry_sha256']=normal['entry_sha256'];n['profiles']={p:dict(normal_sha256=i['normal']['sha256']) for p,i in normal['profiles'].items()}","assert n['status']=='independently_qualified_same_endpoint_catalog_normal_related_both_profiles' and n['passed_per_profile']==16 and n['recorded_owned_processes_all_terminal']\nassert not n['source_overlays_used'] and n['observed_terminal_exit_code']==0\nn['entry_sha256']=sha(ROOT/'啟動現代客戶端.ps1')")
        s=rep(s,"source_archive=Path(normal['compiled_source_archive']['path']);assert sha(source_archive)==normal['compiled_source_archive']['sha256']","source_archive=Path(n['compiled_source_archive']);assert sha(source_archive)==n['compiled_source_archive_sha256']")
        s=rep(s,"normal_mains={p:v['normal_sha256'] for p,v in n['profiles'].items()}","entry_script_sha256=n['entry_sha256'],normal_mains={p:v['normal']['sha256'] for p,v in n['profiles'].items()}")
    elif oldname=='qualify-native33A3-library-default.py':
        start=s.index("assert sha(ROOT/'啟動現代客戶端.ps1')==normal_cohort['entry_sha256']");end=s.index("for name,digest in c['validation_pins']",start)
        s=s[:start]+'''assert sha(ROOT/'啟動現代客戶端.ps1')==c['entry_script_sha256']
assert normal['status']=='independently_qualified_same_endpoint_catalog_normal_related_both_profiles' and normal['passed_per_profile']==16
assert normal['observed_terminal_exit_code']==0 and normal['recorded_owned_processes_all_terminal'] and not normal['source_overlays_used']
identity=Path(normal_cohort['application'])/'applied-source-inputs.json';assert sha(identity)==manifest and read(identity)==pins
broad=read(base/'completion-observation.json');assert broad['status']=='independently_qualified_current1047_default116_both_profiles_and_terminal_processes' and broad['recorded_original_owned_processes_all_terminal']
for profile,info in normal_cohort['profiles'].items():
    nq=normal['profiles'][profile];assert nq['passed']==16 and nq['failed']==nq['ignored']==0
    assert set(nq['targets'])=={'native_services','native_teleport'} and nq['targets']['native_services']['counts']['passed']==2 and nq['targets']['native_teleport']['counts']['passed']==14
    assert sha(info['normal']['path'])==nq['normal']['sha256']==c['normal_mains'][profile]
    expected={'LiToModern.exe':nq['normal']['sha256'],**{name+'.exe':target['sha256'] for name,target in nq['targets'].items()}}
    verify_zip(nq['product_archive'],nq['product_archive_sha256'],expected)
''' +s[end:]
    elif oldname=='checkpoint-native33A3-library-active.py':
        start=s.index("work = '當版33A3CCF7");end=s.index('\n',start)
        s=s[:start]+"work = '來源2DAF4DE0當版廣域116整合目標及main已獨立核對。完整library原'+str(args.session_id)+'沿同一程序編譯、盤點、忽略來源稽核及執行，來源與對應QA固定；待原session實際退出及逐名稱／產物／raw／程序身分讀回。正常兩版各16相關案、控制器96、同端點租裝12、音效96及旅館32sessions保留各自限定證據；舊33A3廣域／library不移記。正常GUI、音效裝置與其他必要功能仍待，開發期間不請使用者登入，8秒／60秒／Release及G11預算不變，相鄰只讀且登入器產物獨立，完整G01–G12未完成。'"+s[end:]
    elif oldname=='checkpoint-native33A3-library-completed.py':
        s=rep(s,"before=read(read(run.parent/'current-continuation.json')['latest_readback'])['after_files']","before=read(Path(read(ROOT/'工作佇列.json')['last_goal_turn_evidence']).with_name('readback.json'))['after_files']")
        start=s.index("work='當版33A3CCF7");end=s.index('\n',start)
        s=s[:start]+"work='來源2DAF4DE0完整library原'+str(a.session)+'已actual exit0及獨立核對：Debug'+str(profiles['Debug']['passed'])+'／Release'+str(profiles['Release']['passed'])+'過、各0敗；忽略'+str(profiles['Debug']['ignored'])+'／'+str(profiles['Release']['ignored'])+'本文未執行。來源、逐名稱、實際映像、raw及原程序終止皆符；當版廣域各116目標維持限定範圍。正常兩版各16相關案、控制器96、同端點租裝12、音效96及旅館32sessions各有當版限定證據，不推廣正常GUI或實體音效。其餘必要WORLD／服務、相機及可見性、全部技能、GUI／可攜交付與G11／G01–G12仍待。開發期間不請使用者登入，8秒／60秒／Release及G11預算不變，相鄰只讀且登入器產物獨立。'"+s[end:]
        s=s.replace("item='current_catalog-audio-handoff-and-necessary-resource-qualification'","item='current-catalog-remaining-WORLD-services-visibility-GUI'")
    ast.parse(s,filename=newname);assert not(T/newname).exists();(out/oldname).write_text(original,encoding='utf-8',newline='\n');(T/newname).write_text(s,encoding='utf-8',newline='\n')
    (out/(newname+'.diff')).write_text(''.join(difflib.unified_diff(original.splitlines(True),s.splitlines(True))),encoding='utf-8',newline='\n');pins[newname]=dict(original=oldname,original_sha256=hashlib.sha256(original.encode()).hexdigest().upper(),new_sha256=hashlib.sha256(s.encode()).hexdigest().upper())
(out/'preparation.json').write_text(json.dumps(dict(tools=pins,normal_related_prerequisite_each=16,current_broad_terminal_qualification_required=True,library_inventory_ignored_source_body_product_archive_and_fixture_checks_preserved=True,library_tests_executed=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(dict(tooling=str(out),AST='passed',tools=len(pins),library_tests_executed=False),ensure_ascii=False))
