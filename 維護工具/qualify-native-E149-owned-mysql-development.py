"""Archive static-only DB launcher development and synchronize eight files."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse, ast, hashlib, json, zipfile

R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('plan',type=Path);ap.add_argument('--plan-terminal-chunk',required=True);a=ap.parse_args()
plan_path=a.plan.resolve(strict=True);assert plan_path.is_relative_to(R/'LiTo381-Modern/target/current/verification/native-E149-owned-mysql')
p=read(plan_path);assert p['status']=='development_plan_only' and not p['database_started'] and p['tests_executed']==0
assert set(x.name for x in plan_path.parent.iterdir())=={'plan.json'}
launcher=Path(p['launcher']);assert sha(launcher)==p['launcher_sha256'] and launcher==T/'native-E149-owned-mysql.py'
assert all(sha(name)==digest for name,digest in p['input_pins'].items())
bundle=read(p['bundle_manifest']);base=Path(bundle['bundle_root']);assert base.is_relative_to(R/'LiTo381-Modern/target/current/owned-database-toolchains')
assert bundle['machine']=='0x8664' and set(bundle['versions'].values())=={'5.5.22.0'}
assert all(sha(base/name)==digest for name,digest in bundle['files'].items())
assert not bundle['database_started'] and not bundle['SQL_executed'] and bundle['executables_invoked']==bundle['tests_executed']==0
assert not bundle['data_directories_copied'] and not bundle['original_configuration_copied']
assert all(Path(n).suffix.lower() not in ['.frm','.myd','.myi','.ibd'] for n in bundle['files'])
assert sha(bundle['JDBC']['source'])==sha(bundle['JDBC']['cached'])==bundle['JDBC']['cached_sha256']
static=read(p['static_manifest']);assert sha(static['SQL'])==static['SQL_sha256'] and static['table_count']==19 and static['rows']==6277

# Compile an AST in memory only: never import or invoke the launcher, driver,
# database, fixture, resource factory, tests, sockets or any native UI surface.
source=launcher.read_text('utf-8');tree=ast.parse(source,filename=str(launcher));compile(tree,str(launcher),'exec')
functions={n.name:n for n in tree.body if isinstance(n,ast.FunctionDef)}
assert isinstance(functions['serve'].body[0],ast.Expr)
assert ast.unparse(functions['serve'].body[0])=='authorize_execution()'
authorize=ast.unparse(functions['authorize_execution'])
assert 'test_execution_deferred_by_user' in authorize and "get('test_execution_deferred_by_user', True)" in authorize
plan_code=ast.unparse(functions['plan'])
assert not any(word in plan_code for word in ['Popen','socket.socket','serve(', 'root-client.ini', 'bootstrap.sql'])
assert [ast.unparse(n) for n in tree.body if isinstance(n,ast.If)]==["if __name__ == '__main__':\n    main()"]
for marker in ['--no-defaults','--bootstrap','--skip-networking','--tmpdir=',
               'SO_EXCLUSIVEADDRUSE','creation_filetime_ticks','CommandLineToArgvW',
               'Get-NetTCPConnection','root_client_sha256','actual_table_rows',
               'SHOW GRANTS FOR','static-import.properties','database-owner.properties',
               'WaitForSingleObject','TerminateProcess','owned_orphan_terminal']:
    assert marker in source,marker
assert 'kill(' not in source and 'taskkill' not in source and 'rmtree' not in source

w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
ref=w['current_formal_reference_identity'];assert sha(ref['formal'])==ref['formal_sha256']==bundle['formal_target_sha256']
q=read(w['current_development_typecheck_qualification']['qualification'])
inputs=read(Path(q['run'])/'base-compiled-inputs.json');assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
assert all(sha(n)==h for n,h in q['normal_product_pins'].items())
assert all(q['candidates'][k]['sha256']==v['sha256'] for k,v in w['current_isolated_development']['candidates'].items())
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())

out=V/'native-E149-owned-mysql-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
record=dict(status='owned_mysql_launcher_source_plan_and_dependency_readback_only',
    formal_target_sha256=ref['formal_sha256'],source_manifest_sha256=w['current_product_identity']['source_manifest_sha256'],
    launcher=str(launcher),launcher_sha256=sha(launcher),plan=str(plan_path),plan_sha256=sha(plan_path),
    observed_plan_original_tool_chunk=a.plan_terminal_chunk,observed_plan_actual_exit_code=0,
    bundle_manifest=p['bundle_manifest'],bundle_manifest_sha256=sha(p['bundle_manifest']),
    copied_files=len(bundle['files']),copied_bytes=bundle['total_bytes'],bundle_machine=bundle['machine'],
    vendor_versions=bundle['versions'],JDBC=bundle['JDBC'],static_manifest=p['static_manifest'],static_manifest_sha256=sha(p['static_manifest']),
    table_count=19,rows=6277,original_statements=6296,AST_parsed_and_compiled_in_memory=True,launcher_module_imported=False,
    static_review=['serve checks user defer before datadir/socket/process',
        'plan contains no process/socket/SQL invocation', 'private fresh datadir/tmpdir and no-defaults',
        'real PID/FILETIME/image/hash/command/listener checks before client/admin',
        'exact static import, actual table/count and SELECT grant readback required',
        'proofs only after actual import; bounded lease and exact handle cleanup; separate orphan recovery'],
    database_executables_invoked=0,database_started=False,SQL_executed=False,tests_executed=0,
    desktop_inputs=0,original_factories_invoked=False,normal_source_unchanged=True,normal_PEs_unchanged=True,
    current_overlay_compilation_reused=True,launcher_source_development_complete=True,
    launcher_runtime_qualified=False,fixture_integration_pending=True,full_Main_process_qualified=False,
    limitations=['AST and file readback do not prove any startup, SQL import, authentication, JDBC or cleanup runtime behavior',
        'MySQL 5.5.22 / Connector-J 8.0.33 / Unicode paths remain unexecuted',
        'Vendor bootstrap, actual schema/row/grant validation and orphan shutdown must be exercised after user resumes tests',
        'No actual import/owner proof exists; prepared plan is not READY',
        'New adapter is still separate from G06 fixture; all current E149 API/GUI and full owner qualification pending'],
    whole_goal_complete=False,test_execution_deferred_by_user=True,recorded_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified-development.json').write_bytes(data(record))
raw={'launcher.py':launcher,'preparer.py':T/'prepare-native-E149-mysql-toolchain.py',
     'reader.py':Path(__file__),'README.md':T/'native-E149-owned-mysql-README.md',
     'bundle.json':Path(p['bundle_manifest']),'plan.json':plan_path,'static-manifest.json':Path(p['static_manifest']),
     'qualified-development.json':out/'qualified-development.json','report-updater-before.py':T/'update-master-report-text-index-order.py'}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,path in raw.items():z.write(path,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(path) for n,path in raw.items())
record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),
    raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
(out/'evidence.json').write_bytes(data(record))
work='E149隔離MySQL工具已完成來源初稿與AST／檔案核對：私有x64 5.5.22工具包及缺少的JDBC已備妥；新啟動器含全新data／tmp、PID／FILETIME／image／command／listener驗證、19表逐表匯入核對、唯讀授權讀回、實際proof門檻與期限／孤兒handle回收。原'+a.plan_terminal_chunk+' exit0只建立計畫，沒有啟動DB／SQL／factory／功能或GUI。1049正常輸入、兩EXE及13檔候選均未變，沿用原編譯資格。依使用者要求測試暫緩，下一步先整合Java bootstrap與G06 fixture之原accept／submit／Future／cleanup；恢復測試後再驗工具實跑、原版6反例／候選14案、33／31秒自然到期、最終library／Release／完整GUI，整體未完成。'
staged={}
for n in before:
    path=R/n
    if path.suffix=='.json':
        j=read(path);j['current_E149_owned_mysql_development']=record
        j['current_E149_resource_bootstrap_development'].update(future_owned_database_launcher_pending=False,
            owned_database_launcher_development=record['qualification'],owned_database_launcher_development_sha256=record['qualification_sha256'],
            owned_database_launcher_runtime_qualified=False,future_fixture_integration_pending=True)
        j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='G06_3d_E149_owned_static_database_development',work=work,applies_to_current_source=False,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep
        staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items())
assert sha(ref['formal'])==ref['formal_sha256'] and all(sha(n)==h for n,h in q['normal_product_pins'].items())
for n,blob in staged.items():(R/n).write_bytes(blob)
assert all((R/n).read_bytes()==blob for n,blob in staged.items())
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},
    before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),qualification=record['qualification'],copied_files=record['copied_files'],
    AST_only=True,plan_only=True,database_started=False,SQL_executed=False,tests_executed=0,eight_files_read_back=True),ensure_ascii=False))
