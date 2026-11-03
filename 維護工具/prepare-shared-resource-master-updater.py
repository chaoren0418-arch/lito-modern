"""Add separately scoped 72-case installed-resource evidence to compact handoff."""
from pathlib import Path
import ast
R=Path(__file__).resolve().parents[1];T=R/'維護工具'
s=(T/'update-master-report-rental-service-entry-return.py').read_text('utf-8-sig')
at="for field, label in [('current_native_rental_entry_guard_qualification'"
assert s.count(at)==1
s=s.replace(at,'''shared=w.get('current_native_library_shared_resource_optin',{})
if shared.get('source_manifest_sha256')==source and shared.get('qualified')is True:
    assert sha(shared['qualification'])==shared['qualification_sha256']
    assert all(shared['profiles'][p]['passed']==72 for p in ['Debug','Release'])
    rows.append('| 原始共用資源額外選取 | 兩版各72過0敗；與原生77之10項不重複 | '+link('限定資格',shared['qualification'])+'；預設忽略本文尚餘402項，非GUI或實體音效 |')
else:
    rows.append('| 原始共用資源72項 | 執行／核對待完成 | '+link('逐名計畫',V/'native-library-shared-resource-optin/plan-20261019-194354-677571/plan.json')+'；PNG輸出保全工具已準備，不能算通過 |')
'''+at)
s=s.replace('3. {\'新API實際封包限定範圍', "   共用資源另跑run-current-shared-resource-library.py <當版library>，以checkpoint-current-shared-resource-library.py登記原session／terminal並逐名核對72項，與原生77十項分列。\n3. {'新API實際封包限定範圍")
ast.parse(s)
p=T/'update-master-report-rental-service-entry-return-with-resources.py';assert not p.exists();p.write_text(s,encoding='utf-8',newline='\n')
print(str(p))
