"""Derive the metadata compiler with explicit source mapping; no compiler starts here."""
from pathlib import Path
from native_E149_recovery_contract import R,sha,require
T=R/'維護工具';source=T/'compile-native-E149-eruption45-core.py';s=source.read_text('utf-8-sig')
require(sha(source)=='42A6B02AB9790B6AE59C4F9BC6CA9B8D4F367D1A2C2BE810D7A1D2A2E12587A1','compiler ancestor differs')
s=s.replace('eruption45-core','eruption45-scene').replace('20261024-085557','20261024-091703')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler anchor differs '+a[:90]);s=s.replace(a,b)
replace('proof=read(contract[\'inspection\']);require(sha(contract[\'inspection\'])==contract[\'inspection_sha256\'],\'inspection changed\');pins.update(proof[\'files\'])',
 '''proof=read(contract['inspection']);require(sha(contract['inspection'])==contract['inspection_sha256'],'inspection changed')
 for p,h in proof['source_pins'].items():
  if Path(p).is_relative_to(R):pins[p]=h
  else:external[p]=h
 pins.update({row['path']:row['sha256'] for row in contract['runtime_overrides'].values()})''')
replace("==362,'predecessor module count differs'", "==364,'predecessor module count differs'")
replace("require(all('mod '+name+';' not in modern for name in contract['modules']),'module name collision')", '''require(all('mod '+name+';' not in modern for name in contract['modules']),'module name collision')
 for name,row in contract['runtime_overrides'].items():
  require(sha(row['predecessor'])==row['predecessor_sha256'],'runtime predecessor changed')
  needles=list(dict.fromkeys([row['predecessor'],Path(row['predecessor']).as_posix()]));found=[n for n in needles if modern.count(n)==1];require(len(found)==1 and sum(modern.count(n) for n in needles)==1,'runtime predecessor mapping differs');modern=modern.replace(found[0],Path(row['path']).as_posix())''')
replace("==364,'supplementary module count differs'", "==365,'supplementary module count differs'")
replace("module_count=364,original_nested_runtime_pairs=old['paired_runtime_sources']", "module_count=365,original_nested_runtime_pairs=old['original_nested_runtime_pairs']")
replace('scene_integrated=False','scene_integrated=True')
replace('complete predecessor graph plus two unwired modules','complete predecessor graph plus received S23/129 presentation')
out=T/'compile-native-E149-eruption45-scene.py';require(not out.exists(),'do not overwrite compiler');out.write_text(s,encoding='utf-8',newline='\n');print(out)
