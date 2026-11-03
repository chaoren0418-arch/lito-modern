"""Pin the reviewed external JAR version in new owned QA copies; never edit the launcher."""
from pathlib import Path
import ast,hashlib,json,re
ROOT=Path(__file__).resolve().parents[1];T=ROOT/'維護工具';V=ROOT/'驗證紀錄/current'
OLD='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539';NEW='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
drift=V/'formal-75DA69B0/observation-20261018-140916-340781/drift.json';d=json.loads(drift.read_text('utf-8-sig'));assert sha(d['formal_path'])==sha(d['isolated_copy'])==NEW
assert set(d['changed_entries'])=={'local/lito381/server/LegacyMySqlRepository.class'} and d['added_entries']==['local/lito381/server/SourceBuffRead.class'] and not d['removed_entries']
pins={};outputs={}
for kind in ['handoff','failure']:
    source=T/('run-teleport-'+kind+'.ps1');pins[source.name]=sha(source);s=source.read_text('utf-8-sig')
    s,n=re.subn(r"(?m)^if\(\$jarSha -notin @\([^\n]+\)\)\{throw 'Review candidate drift'\}$", "if($jarSha -ne '"+NEW+"'){throw 'Reviewed 75DA reference hash mismatch'}",s);assert n==1
    if kind=='handoff':
        s=s.replace("'"+OLD+"') -and", "'"+OLD+"','"+NEW+"') -and")
    else:
        needle="$receipt.reference_is_current=($receipt.observed_source_before_sha256 -eq $jarSha -and $receipt.observed_source_after_sha256 -eq $jarSha)";assert s.count(needle)==1
        s=s.replace(needle,needle+"\n if(!$receipt.reference_is_current){throw 'Reviewed current formal source drift during failure test'}",1)
    path=T/('run-teleport-'+kind+'-75DA.ps1');path.write_text(s,encoding='utf-8',newline='\n');outputs[path.name]=sha(path)
source=T/'qualify-native1040-G06-controller-case.py';pins[source.name]=sha(source);s=source.read_text('utf-8-sig').replace(OLD,NEW);ast.parse(s);path=T/'qualify-native33199-G06-controller-case-75DA.py';path.write_text(s,encoding='utf-8',newline='\n');outputs[path.name]=sha(path)
path=T/'run-native1040-G06-scoped-catalog-normal.py';pins[path.name]=sha(path);s=path.read_text('utf-8-sig').replace(OLD,NEW).replace('formal-server-C02AAAE8.jar','formal-server-75DA69B0.jar').replace('run-teleport-handoff.ps1','run-teleport-handoff-75DA.ps1').replace('run-teleport-failure.ps1','run-teleport-failure-75DA.ps1').replace('qualify-native1040-G06-controller-case.py','qualify-native33199-G06-controller-case-75DA.py');ast.parse(s);path.write_text(s,encoding='utf-8',newline='\n');outputs[path.name]=sha(path)
for name in ['compile-native33199-teleport-audio.py','run-native33199-teleport-audio.py','qualify-native33199-teleport-audio.py','run-native33199-audio-cohort.py']:
    path=T/name;pins[name]=sha(path);s=path.read_text('utf-8-sig').replace(OLD,NEW).replace('formal-server-C02AAAE8.jar','formal-server-75DA69B0.jar').replace('run-teleport-handoff.ps1','run-teleport-handoff-75DA.ps1').replace('run-teleport-failure.ps1','run-teleport-failure-75DA.ps1');ast.parse(s);path.write_text(s,encoding='utf-8',newline='\n');outputs[name]=sha(path)
record=dict(status='prepared_new_reference_specific_G06_QA_copies_not_executed',formal_reference_sha256=NEW,isolated_reference=d['isolated_copy'],drift_evidence=str(drift),drift_evidence_sha256=sha(drift),origin_tool_sha256=pins,current_tool_sha256=outputs,shared_historical_PS_and_case_readers_modified=False,normal_source_modified=False,adjacent_source_modified=False,acceptance_limits_changed=False,runtime_executed=False,current_formal_compatibility_qualified=False)
path=V/'formal-75DA69B0/QA-preparation.json';assert not path.exists();path.write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(preparation=str(path),reference_sha256=NEW,normal_or_adjacent_source_modified=False,executed=False),ensure_ascii=False))
