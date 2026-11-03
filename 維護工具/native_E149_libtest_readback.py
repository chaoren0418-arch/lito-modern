"""Pure libtest text contracts. Importing starts no process and runs no tests."""
import re

def require(ok,message):
 if not ok:raise RuntimeError(message)

def inventory(stdout,stderr,exit_code):
 require(exit_code==0 and not stderr.strip(),'libtest inventory did not exit cleanly')
 names=[];summary=None
 for line in stdout.splitlines():
  if not line.strip():continue
  entry=re.fullmatch(r'([^\r\n]+): test',line)
  count=re.fullmatch(r'(\d+) tests?, (\d+) benchmarks?',line)
  if entry and summary is None:names.append(entry[1])
  elif count and summary is None:summary=(int(count[1]),int(count[2]))
  else:raise RuntimeError('unreviewed libtest inventory line')
 require(summary is not None and summary==(len(names),0) and len(names)==len(set(names)) and names,
  'libtest inventory cardinality/duplicates differ')
 return names

def selected_result(stdout,stderr,exit_code,test,total,expected,panic_site=None,panic_contains=()):
 require(type(total) is int and total>0 and expected in ['pass','specific_assertion_failure'],'invalid selected libtest contract')
 summaries=re.findall(r'(?m)^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;(?:[^\r\n]*)$',stdout)
 require(len(summaries)==1,'selected result summary missing or ambiguous')
 outcome,*numbers=summaries[0];numbers=list(map(int,numbers))
 passed=expected=='pass'
 require(exit_code==(0 if passed else 101),'selected case exit is not ordinary pass/panic; crashes do not qualify')
 require(outcome==('ok' if passed else 'FAILED') and numbers==([1,0,0,0,total-1] if passed else [0,1,0,0,total-1]),
  'selected case count/ignored/filtered result differs from actual inventory')
 require(re.search(r'(?m)^running 1 test\s*$',stdout),'not a single exact test execution')
 status='ok' if passed else 'FAILED'
 require(re.search(r'(?m)^test '+re.escape(test)+r' \.\.\. '+status+r'\s*$',stdout),'wrong exact selected test status')
 panics=re.findall(r"(?m)^thread '([^'\r\n]+)' panicked at (.+):(\d+):(\d+):\s*$",stderr)
 if passed:require(not panics,'passing case retained a worker panic')
 else:
  require(panic_site is not None and len(panics)==1 and panics[0][0]==test,'expected one panic from this test, not another worker')
  name,path,line,column=panics[0]
  require(path.replace('\\','/').casefold()==panic_site['file'].replace('\\','/').casefold()
   and int(line)==panic_site['line'] and int(column)>0,'panic came from another source/assertion site')
  require(panic_contains and all(value in stderr for value in panic_contains),'designated assertion evidence missing')
 return dict(test_name=test,expected=expected,Rust_exit_code=exit_code,passed=int(passed),
  expected_assertion_failure_observed=not passed,inventory=total,filtered_out=total-1,
  compiler_error_or_crash_accepted=False,full_library_qualified=False,full_G06_qualified=False)
