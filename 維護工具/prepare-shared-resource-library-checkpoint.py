"""Prepare independent exact-name/resource-PNG qualification, without executing tests."""
from pathlib import Path
from datetime import datetime,timezone
import ast,json,hashlib
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
s=(T/'checkpoint-native-rental-service-entry-return-library-resource-optin.py').read_text('utf-8-sig')
s=s.replace('native-rental-service-entry-return-library-resource-optin','native-library-shared-resource-optin-execution')
s=s.replace('run-native-library-shared-resource-optin-execution.py','run-current-shared-resource-library.py')
s=s.replace("c['status']=='running_explicit_native77_resource_optin'", "c['status']=='running_explicit_shared_resource72_resource_optin'")
s=s.replace("c['status']=='executed_both_current_native77_resource_optin_selections'", "c['status']=='executed_both_current_shared_resource72_resource_optin_selections'")
s=s.replace("expected={x['test']:x for x in audit['tests']if x['test'].startswith('modern_lito::native_disintegrate_tests::native77_probe::')}",
'''plan=read(run/'plan.json');assert sha(run/'plan.json')==c['plan_sha256']==sha(c['plan'])
            assert sha(run/'test-bodies.json')==plan['selected_bodies_sha256']
            names={x['test']for x in plan['selected_tests']};assert len(names)==72
            expected={x['test']:x for x in audit['tests']if x['test']in names}
            assert set(expected)==names
            listed=(run/profile/'selection-list.txt').read_text('utf-8-sig')
            assert {line[:-6]for line in listed.splitlines()if line.endswith(': test')}==names
            assert row['command']==[str(Path(product['product']).resolve()),*sorted(names),'--exact','--ignored','--test-threads=1']
            assert c['resource_metadata_unchanged'] and c['diagnostic_historical_outputs_restored']
            assert len(row['diagnostic_PNGs'])>7
            for n,h in row['diagnostic_PNGs'].items():
                png=run/profile/'diagnostic-PNG'/n;assert sha(png)==h
                header=png.read_bytes()[:24];assert header[:8]==b'\\x89PNG\\r\\n\\x1a\\n'and header[12:16]==b'IHDR'
                assert 0<int.from_bytes(header[16:20],'big')<=4096 and 0<int.from_bytes(header[20:24],'big')<=4096''')
s=s.replace('len(expected)==10','len(expected)==72').replace('len(found)==10','len(found)==72')
s=s.replace("('ok','10','0','0','0',str(q['inventory']-10))", "('ok','72','0','0','0',str(q['inventory']-72))")
s=s.replace('passed=10','passed=72').replace("q['ignored']-10", "q['ignored']-72")
s=s.replace('current_native_library_resource_optin','current_native_library_shared_resource_optin')
s=s.replace('library_resource_optin','library_shared_resource_optin')
s=s.replace('各10過0敗','各72過0敗').replace('原生77','原始共用資源72項').replace('native77','shared_resource72')
# Every independently checked diagnostic is sealed with the stdout and exact-selection list.
s=s.replace("    hashes={n:sha(p)for n,p in files.items()}",'''    for profile in ['Debug','Release']:
        for n in ['selection-help.txt','selection-list.txt']:files[profile+'/'+n]=run/profile/n
        for n in c['profiles'][profile]['diagnostic_PNGs']:files[profile+'/diagnostic-PNG/'+n]=run/profile/'diagnostic-PNG'/n
    for n in ['plan.json','test-bodies.json','resource-metadata-before.json','diagnostic-before.json','diagnostic-before.zip']:files[n]=run/n
    hashes={n:sha(p)for n,p in files.items()}''')
s=s.replace("if qualified:\n    files=", "if qualified:\n    record['remaining_unexecuted_ignored_after_disjoint_native77']=402 if w.get('current_native_library_resource_optin',{}).get('qualified')else None\n    files=")
ast.parse(s)
p=T/'checkpoint-current-shared-resource-library.py';assert not p.exists();p.write_text(s,encoding='utf-8',newline='\n')
out=V/'native-library-shared-resource-optin'/('checkpoint-tooling-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'generator.py').write_bytes(Path(__file__).read_bytes())
(out/'receipt.json').write_text(json.dumps(dict(checkpoint=str(p),checkpoint_sha256=sha(p),prepared_only=True,executed=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),checkpoint=str(p),executed=False),ensure_ascii=False))
