"""Derive metadata compiler and rebuild only original six nested path bindings."""
from pathlib import Path
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';w=read(R/'工作佇列.json');q=read(w['current_E149_freeze50_development']['qualification']);p=T/'compile-native-E149-freeze50.py';require(sha(p)==q['source_pins'][str(p)],'ancestor differs');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler anchor differs '+a[:90]);s=s.replace(a,b)
replace('native-E149-freeze50-20261024-103539','native-E149-summon51-20261024-105000')
replace("T/'templates/native_freeze50_request_tests.rs',T/'templates/native_freeze50_clock_tests.rs',T/'templates/native_freeze50_scene_tests.rs'", "T/'templates/native_summon51_support.rs',T/'templates/native_summon51_tests.rs'")
lo=s.index(" for name,row in contract['runtime_overrides'].items():\n  require(sha(row['predecessor'])");hi=s.index(" for name,row in contract['modules'].items():modern+=",lo)
s=s[:lo]+''' nested_pairs=dict(old['original_nested_runtime_pairs'])
 for name,row in contract['runtime_overrides'].items():
  require(sha(row['predecessor'])==row['predecessor_sha256'],'runtime predecessor changed')
  previous=row['predecessor'];replacement=row['path']
  if name in nested_pairs:
   pair=nested_pairs[name];require(pair['source']==previous and sha(pair['source'])==pair['source_sha256'] and sha(pair['path'])==pair['sha256'],'original nested pair differs');text=Path(row['path']).read_text('utf-8-sig')
   for rewrite in pair['rewrites']:
    require(text.count(rewrite['original_attribute'])==1 and Path(rewrite['resolved_path']).is_file(),'nested source binding differs');text=text.replace(rewrite['original_attribute'],rewrite['compiled_attribute']);pins[rewrite['resolved_path']]=sha(rewrite['resolved_path'])
   compiled=out/('paired-runtime-'+Path(name).name);compiled.write_text(text,encoding='utf-8',newline='\\n');pins[str(compiled)]=sha(compiled);nested_pairs[name]=dict(path=str(compiled),sha256=sha(compiled),source=row['path'],source_sha256=row['sha256'],rewrites=pair['rewrites'],predecessor=pair);previous=pair['path'];replacement=str(compiled)
  needles=list(dict.fromkeys([previous,Path(previous).as_posix()]));found=[n for n in needles if modern.count(n)==1];require(len(found)==1 and sum(modern.count(n) for n in needles)==1,'runtime predecessor mapping differs');modern=modern.replace(found[0],Path(replacement).as_posix())
'''+s[hi:]
replace("original_nested_runtime_pairs=old['original_nested_runtime_pairs']", 'original_nested_runtime_pairs=nested_pairs')
s=s.replace('freeze50','summon51').replace('50 request and silent class4 plus flight history/failed preparation','51 typed summon selection UI and controller domain')
out=T/'compile-native-E149-summon51.py';require(not out.exists(),'compiler exists');out.write_text(s,encoding='utf-8',newline='\n');print(out)
