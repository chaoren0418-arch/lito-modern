from r491_common import *
p=P/'tools/seal-r493-handoff.py';s=p.read_text(encoding='utf-8')
s=s.replace("sorted(L.glob('r493*.log'))]","sorted(L.glob('r493*.log')) if not p.name.startswith('r493-seal.')]")
s=s.replace("v.setdefault('completed',[]).append('R493","v['completed']=[x for x in v.get('completed',[]) if not x.startswith('R493：')];v['completed'].append('R493")
s=s.replace('Rust build/test failure0，獨立核對首次通過。','封存準備另有一次欄位命名語法錯誤，零封存輸出；其失敗來源與後續missinghandoff gate拒絕已記錄，另排除仍在寫入的sealstdout/stderr prospective pins。Rust build/test failure0，獨立來源／行為核對首次通過。')
s=s.replace("prep['successful_seal_revisions']=2;","prep['successful_seal_revisions']=2;prep['shell_preparation_quoting_failures']=1;prep['shell_preparation_correction']='PowerShell rejected nestedquoted inlinePython beforeexecution/zerochanges;use savedUTF8 repairscript';")
s=s.replace("'seal-r493-handoff.py','verify-r493-sealed-links.py'","'seal-r493-handoff.py','repair-r493-seal.py','verify-r493-sealed-links.py'")
p.write_text(s,encoding='utf-8')
print('R493 seal metadata/stream pins repaired;no production changes orconsumer rerun')
