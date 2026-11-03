"""Preserve original extern-name failure; fix metadata filename only, no Rust/QA edit."""
from pathlib import Path
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';V=R/'驗證紀錄/current';failed=V/'native-E149-eruption45-entry-metadata/20261024-094354-536672/cohort.json';c=read(failed)
require(c['status']=='original_compile_failure_preserved' and [r['exit_code'] for r in c['compiler_runs']]==[0,0,1] and c['owned_process'] is None,'original compiler failure differs')
logs=c['compiler_runs'][-1]['logs'];stderr=Path(next(p for p in logs if p.endswith('stderr.log'))).read_text('utf-8-sig');require('extern location for lito_modern is of an unknown type' in stderr and 'Debug-library.rmeta' in stderr,'extern-name diagnosis differs')
source=T/'compile-native-E149-eruption45-entry.py';s=source.read_text('utf-8-sig');failure=dict(cohort=str(failed),cohort_sha256=sha(failed),observed_original_tool_chunk='180a76',observed_actual_exit_code=1,source_or_QA_changed=False,prior_partial_success_transferred=False,source_pins={str(failed):sha(failed),str(source):sha(source),**{p:h for r in c['compiler_runs'] for p,h in r['logs'].items()}})
def replace(a,b):
 global s
 require(s.count(a)==1,'libname compiler anchor differs '+a[:100]);s=s.replace(a,b)
replace("product=products/(profile+'-'+kind+'.rmeta')", "product=products/('liblito_modern-'+profile+'.rmeta') if kind=='library' else products/(profile+'-'+kind+'.rmeta')")
replace("str(products/(profile+'-library.rmeta'))", "str(products/('liblito_modern-'+profile+'.rmeta'))")
literal=repr(failure)
replace("pins=dict(old['input_pins']);external=dict(old['external_toolchain_dependency_pins'])", "pins=dict(old['input_pins']);external=dict(old['external_toolchain_dependency_pins']);original_tooling_failure="+literal+";pins.update(original_tooling_failure['source_pins']);pins[str(T/'prepare-native-E149-eruption45-entry-libname.py')]=sha(T/'prepare-native-E149-eruption45-entry-libname.py')")
replace("pipeline=identity(kernel().GetCurrentProcess(),os.getpid()),compiler_runs=[]", "original_tooling_failure=original_tooling_failure,pipeline=identity(kernel().GetCurrentProcess(),os.getpid()),compiler_runs=[]")
out=T/'compile-native-E149-eruption45-entry-libname.py';require(not out.exists(),'do not overwrite corrected compiler');out.write_text(s,encoding='utf-8',newline='\n');print(out)
