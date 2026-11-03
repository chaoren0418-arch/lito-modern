"""Qualify static extraction and compilation; publish no runtime qualification."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-E149-bootstrap-adapter-compile')
cohort=read(run/'cohort.json');assert cohort['status']=='adapter_compilation_only_complete' and cohort['compiler_exit_code']==0
assert cohort['tests_executed']==cohort['classes_instantiated']==cohort['desktop_inputs']==0
assert not cohort['SQL_executed'] and not cohort['database_started'] and not cohort['original_factories_invoked']
assert Path(cohort['command'][0]).name=='javac.exe' and '-proc:none' in cohort['command']
assert all(sha(p)==h for p,h in cohort['input_pins'].items()) and all(sha(p)==h for p,h in cohort['classes'].items())
assert len(cohort['classes'])==2 and all(sha(p)==h for p,h in cohort['logs'].items())
cpath=Path(cohort['candidate']);c=read(cpath);assert sha(cpath)==cohort['candidate_sha256']
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
ref=w['current_formal_reference_identity'];assert sha(ref['formal'])==ref['formal_sha256']==c['formal_target_sha256']
inputs=read(cohort['base_compiled_inputs']);assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
assert all(sha(p)==h for p,h in cohort['normal_product_pins'].items())
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())

# Independent reconstruction from original offsets, plus independent anchored
# statement counts over the source dump. No SQL parser or server is invoked.
m=read(c['static_manifest']);assert sha(c['static_manifest'])==c['static_manifest_sha256']
dump=Path(m['source_dump']).read_bytes();assert hashlib.sha256(dump).hexdigest().upper()==m['source_dump_sha256']
assert sha(m['SQL'])==m['SQL_sha256']==c['SQL_sha256'] and m['table_count']==19 and m['query_count']==18
queries=[s for s in re.findall(r'// String (.*)',Path(m['query_disassembly']).read_text('utf-8-sig')) if s.startswith('SELECT ')]
assert queries==m['original_queries']
tables=sorted({n for q in queries for n in re.findall(r'(?:FROM|JOIN) `([^`]+)`',q)})
assert set(tables)==set(m['selected_tables'])
parts=[]
for name,table in m['selected_tables'].items():
    anchored=re.findall(rb'(?mi)^\s*(CREATE\s+TABLE|INSERT\s+INTO)\s+`'+re.escape(name.encode('utf-8'))+rb'`',dump)
    assert sum(v.upper().startswith(b'CREATE') for v in anchored)==table['create_statements']==1
    assert sum(v.upper().startswith(b'INSERT') for v in anchored)==table['insert_statements']==table['rows']
    for pin in table['source_statements']:
        blob=dump[pin['offset']:pin['end']]
        assert len(blob)==pin['bytes'] and hashlib.sha256(blob).hexdigest().upper()==pin['sha256']
        assert pin['kind'] in ['CREATE TABLE','INSERT INTO']
        assert pin['rows']==(1 if pin['kind']=='INSERT INTO' else 0)
        parts.append((pin['offset'],pin['end'],blob))
parts.sort();assert len(parts)==m['statement_count']==6296
assert all(b<=c for (_,b,_),(c,_,_) in zip(parts,parts[1:]))
header=b'-- E149 static items: only import through an owned isolated database runner.\nSET NAMES utf8mb4;\n'
assert header+b'\n'.join(blob for _,_,blob in parts)+b'\n'==Path(m['SQL']).read_bytes()
assert sum(t['rows'] for t in m['selected_tables'].values())==m['rows']==6277

# Inspect bytecode, not class initialization. Verify the original calls are
# present in order and no constructed snapshots or reflective writes appear.
out=V/'native-E149-bootstrap-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'reader.py').write_bytes(Path(__file__).read_bytes())
command=[r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe','-J-Dfile.encoding=UTF-8','-p','-c','-constants',
    '-classpath',str(run/'classes')+';'+c['formal_reference'],'local.lito381.server.NativeRecoveryProductionBootstrap']
result=subprocess.run(command,cwd=out,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
(out/'adapter.stdout.txt').write_bytes(result.stdout);(out/'adapter.stderr.txt').write_bytes(result.stderr)
assert result.returncode==0;bytecode=result.stdout.decode('utf-8-sig')
chain=['NativeServerLifetime.install:','NativeDeItemTemplatesResources.read:','NativeDeItemDefinitionsResources.read:','NativeDeItemApplication.install:']
locations=[bytecode.index(name) for name in chain];assert locations==sorted(locations)
for name in ['NativeServerLifetime.acceptSocket:','NativeServerLifetime.submitAccepted:','NativeServerLifetime.closeHub:',
             'NativeServerLifetime.closeResource:','NativeServerLifetime.finishClients:','java/sql/DriverManager.getConnection:']:
    assert name in bytecode
assert not re.search(r'java/lang/reflect/Field\.(?:set:|setInt:|setLong:|setBoolean:)',bytecode)
assert 'java/lang/reflect/Proxy' not in bytecode and 'NativeDeItemTemplatesResources$Snapshot."<init>"' not in bytecode and 'NativeDeItemDefinitionsResources$Snapshot."<init>"' not in bytecode
source=Path(c['source']).read_text('utf-8')
assert source.index('verifyDatabase(root,')<source.index('NativeServerLifetime.install(')
assert all(s in source for s in ['info.startInstant()','info.command()','--datadir','--bind-address','Get-NetTCPConnection',
    'connection.isClosed()','error.addSuppressed','loadingError.addSuppressed','MemoryAccountRepository.class'])

# Original compiler and pipeline identities must be terminal now. Reused PIDs
# are compared by creation time and are not mistaken for the old processes.
identities=[cohort['pipeline'],cohort['compiler']];ids=','.join(str(x['pid']) for x in identities)
query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed=subprocess.check_output(['pwsh','-NoProfile','-Command',query]);actual=json.loads(observed.decode('utf-8-sig')) if observed.strip() else []
actual=[actual] if isinstance(actual,dict) else actual
for row in identities:
    assert not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in actual)
(out/'owned-CIM.json').write_bytes(observed)
record=dict(status='E149_bootstrap_adapter_compilation_and_exact_static_data_readback_only',
    candidate=str(cpath),candidate_sha256=sha(cpath),compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),
    observed_original_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0,compiler_exit_code=0,classes=cohort['classes'],original_identities_terminal=True,
    formal_target_sha256=c['formal_target_sha256'],source_manifest_sha256=c['source_manifest_sha256'],
    static_manifest=c['static_manifest'],static_manifest_sha256=sha(c['static_manifest']),SQL=m['SQL'],SQL_sha256=m['SQL_sha256'],
    table_count=19,rows=6277,original_statements=6296,selected_statement_bytes_independently_reconstructed=True,
    original_factory_call_order=chain,bytecode_command=command,bytecode_exit_code=0,
    no_mock_JDBC_or_constructed_snapshots=True,reflective_authority_writes=False,
    original_factories_invoked=False,SQL_executed=False,database_started=False,classes_instantiated=0,tests_executed=0,desktop_inputs=0,
    future_owned_database_launcher_pending=True,future_fixture_integration_pending=True,full_Main_process_qualified=False,
    limitations=['This is a development adapter using original read/read/install factories with an isolated real-JDBC supplier; it is not a complete Main process qualification',
        'Original UnifiedConfiguration is not repurposed to open player repositories; all fixture accounts remain the concrete original memory repositories',
        'Database owner/import receipts and live PID/birth/image/datadir/port checks must be produced by a real future isolated launcher',
        'SQL import compatibility, actual resource bootstrap, complete owner/item/wire continuity and full Debug/Release/GUI remain unexecuted'],
    whole_goal_complete=False,recorded_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified-development.json').write_bytes(data(record))
raw={'candidate.json':cpath,'adapter.java':Path(c['source']),'compile-cohort.json':run/'cohort.json','compiler.py':run/'compiler.py',
    'stdout.log':run/'stdout.log','stderr.log':run/'stderr.log','static-manifest.json':Path(c['static_manifest']),
    'adapter.stdout.txt':out/'adapter.stdout.txt','adapter.stderr.txt':out/'adapter.stderr.txt','reader.py':Path(__file__),
    'qualified-development.json':out/'qualified-development.json'}
for p in cohort['classes']:raw['classes/'+Path(p).relative_to(run/'classes').as_posix()]=Path(p)
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in raw.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in raw.items())
record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),
    previous_goal_turn_evidence=w['last_goal_turn_evidence'],previous_goal_turn_classification='progress',test_execution_deferred_by_user=True)
(out/'evidence.json').write_bytes(data(record))
work='E149原始物品資源初始化已新增獨立開發接法，依原lifetime→templates.read→definitions.read→item.install順序，保留原SQL白名單／rollback／close／freeze；只供真實隔離JDBC，不造空snapshot或owner。19原靜態表6277列／6296陳述式已逐片段讀回，未匯入；新Java原'+a.terminal_chunk+' exit0、兩class與原程序終止已核對。現有Rust與13檔三候選沒有變更，沿用原編譯資格。DB啟動／匯入／原factory／整合fixture／功能與GUI全部未執行；依使用者要求測試暫緩。下一步完成owned MySQL啟動／匯入證據與fixture整合的開發，之後才實跑原版6反例／14案、自然33／31秒、最終library／Release／完整GUI，整體未完成。'
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j['current_E149_resource_bootstrap_development']=record
        j['current_E149_recovery_development'].update(resource_bootstrap_development=record['qualification'],resource_bootstrap_development_sha256=record['qualification_sha256'],full_native_owner_initialization_pending=True)
        j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='G06_3d_E149_original_resource_bootstrap_development',work=work,applies_to_current_source=False,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep
        staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items()) and sha(ref['formal'])==ref['formal_sha256']
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),qualification=record['qualification'],tables=19,rows=6277,classes=2,tests_executed=0,SQL_executed=False),ensure_ascii=False))
