from r491_common import *
import shutil
o=P/'target/r500-login-lifecycle-study/library-build-b-failure';o.mkdir(exist_ok=False);p=P/'src/modern_lito/native_city_window.rs';shutil.copyfile(p,o/p.name);save(o/'failure.json',dict(step='librarybuildb',error='3diagnostics:duplicatecomma inNETWORKmatch;2missingEM_SETLIMITTEXTimportuses',source=pin(o/p.name),messages=pin(L/'r500b-library-build.jsonl'),stderr=pin(L/'r500b-library-build.stderr.log')))
s=p.read_text(encoding='utf-8').replace('Some(LOGIN_SUBMIT)=>login_submit(hwnd,s),,Some(OFFLINE)','Some(LOGIN_SUBMIT)=>login_submit(hwnd,s),Some(OFFLINE)').replace('DRAWITEMSTRUCT,ODS_SELECTED','DRAWITEMSTRUCT,ODS_SELECTED,EM_SETLIMITTEXT');p.write_text(s,encoding='utf-8')
print('R500 failedWindow preserved;syntax/import repaired')
