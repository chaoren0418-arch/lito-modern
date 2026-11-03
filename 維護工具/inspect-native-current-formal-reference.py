"""Freeze/read the newly observed formal JAR; javap only, never run a server."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json,os,re,shutil,subprocess,zipfile
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
save=lambda p,v:Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
actual=R.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
expected='E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498'
assert sha(actual)==expected
old_jar=P/'target/current/server-reference/formal-server-75DA69B0.jar'
assert sha(old_jar)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
w=read(R/'工作佇列.json');assert w['current_user_requested_goal']['test_execution_deferred_by_user']
q=read(w['current_development_typecheck_qualification']['qualification']);inputs=read(Path(q['run'])/'base-compiled-inputs.json')
assert all(sha(R/n)==h for n,h in inputs.items())
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert all(sha(R/n)==h for n,h in before.items())
out=V/'native-formal-reference-refresh'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'inspector.py').write_bytes(Path(__file__).read_bytes());(out/'scratch').mkdir()
cache=P/'target/current/server-reference/formal-server-E1498871.jar'
if cache.exists():assert sha(cache)==expected
else:
    shutil.copyfile(actual,cache);assert sha(cache)==expected and sha(actual)==expected
save(out/'reference.json',dict(formal=str(actual),formal_sha256=expected,cached=str(cache),cached_sha256=sha(cache),
    bytes=actual.stat().st_size,modified_ns=actual.stat().st_mtime_ns,previous_formal=str(old_jar),previous_formal_sha256=sha(old_jar)))
names=['ClientSession','GameHub','Protocol','ServerConfig','RuntimeInventory','RentalLoadoutPlan','SourceRoutedAccountRepository','MemoryAccountRepository','NativeLitoRentalTeleportEndpointCatalog','BattleSpellCatalog','BattleSpellCatalog$BlindSpell']
classes={}
with zipfile.ZipFile(old_jar) as old,zipfile.ZipFile(cache) as new:
    for name in names:
        n='local/lito381/server/'+name+'.class';a=hashlib.sha256(old.read(n)).hexdigest().upper();b=hashlib.sha256(new.read(n)).hexdigest().upper()
        classes[name]=dict(previous_sha256=a,current_sha256=b,byte_identical=a==b)
    manifest=new.read('META-INF/MANIFEST.MF').decode('utf-8')
(out/'manifest.txt').write_text(manifest,encoding='utf-8')
old_root=V/'native-source-handoff-expiry/contract-20261019-174028-047309'
old_texts={'ClientSession':old_root/'ClientSession.stdout.txt','GameHub':old_root/'GameHub.stdout.txt',
    'Protocol':V/'native-blind-formal75-contract/20261023-181722-242463/Protocol.stdout.txt'}
keys={
 'ClientSession':['login','completeGameBootstrap','autoSelectPendingRentalBattleHandoff','requestLitoCharacterInfo','characterSelect','handleRentalTownTeleportAction','quitFight','activatePendingRentalBattleHandoff','lambda$autoSelectPendingRentalBattleHandoff$30'],
 'GameHub':['stageClassicSourceHandoff','pendingClassicSourceHandoff','consumeClassicSourceHandoff','stageRentalBattleHandoff','hasRentalBattleHandoff','consumeRentalBattleHandoff'],
 'Protocol':['litoCharacterSourceRequest','litoCharacterDirectoryCount','litoCharacterDirectoryEntry','litoCharacterSelectResult','litoServerDirectory','vsServerStatus','litoVsServerIp','blind','attackSpellRequest'],
}
def methods(text):
    lines=text.splitlines();starts=[i for i,line in enumerate(lines) if re.match(r'^  \S.*\(.*\).*;$',line)]
    return {lines[i]:'\n'.join(lines[i:starts[j+1] if j+1<len(starts) else len(lines)])+'\n' for j,i in enumerate(starts)}
def method_name(header):return header.split('(',1)[0].rsplit(' ',1)[-1]
def normalize(method):return re.sub(r'#\d+','#POOL',method)
jdk=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin');commands=[];comparison={};raw_files={}
for name,previous_path in old_texts.items():
    command=[str(jdk/'javap.exe'),'-J-Dfile.encoding=UTF-8','-J-Djava.io.tmpdir='+str(out/'scratch'),'-p','-c','-constants','-classpath',str(cache),'local.lito381.server.'+name]
    result=subprocess.run(command,cwd=out,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
    (out/(name+'.stdout.txt')).write_bytes(result.stdout);(out/(name+'.stderr.txt')).write_bytes(result.stderr)
    commands.append(dict(command=command,exit_code=result.returncode));save(out/'commands.json',commands)
    assert result.returncode==0 and sha(actual)==expected and sha(cache)==expected
    previous=methods(previous_path.read_text('utf-8-sig'));current=methods(result.stdout.decode('utf-8-sig'))
    selected_old={h:m for h,m in previous.items() if method_name(h) in keys[name]}
    selected_new={h:m for h,m in current.items() if method_name(h) in keys[name]}
    assert selected_old and selected_new
    save(out/(name+'-previous-selected.json'),selected_old);save(out/(name+'-current-selected.json'),selected_new)
    comparison[name]={}
    for header in sorted(set(selected_old)|set(selected_new)):
        old_method=selected_old.get(header);new_method=selected_new.get(header)
        same=old_method is not None and new_method is not None and normalize(old_method)==normalize(new_method)
        comparison[name][header]=dict(previous_present=old_method is not None,current_present=new_method is not None,
            constant_pool_normalized_identical=same,previous_sha256=None if old_method is None else hashlib.sha256(old_method.encode()).hexdigest().upper(),
            current_sha256=None if new_method is None else hashlib.sha256(new_method.encode()).hexdigest().upper())
    diff=''.join(difflib.unified_diff('\n'.join(normalize(m) for m in selected_old.values()).splitlines(True),'\n'.join(normalize(m) for m in selected_new.values()).splitlines(True),fromfile='75DA69B0/'+name,tofile='E1498871/'+name))
    (out/(name+'-selected.diff')).write_text(diff,encoding='utf-8')
    raw_files[str(previous_path)]=sha(previous_path)
proof=dict(status='new_formal_reference_detected_static_selected_contract_comparison_only',formal=str(actual),formal_sha256=expected,
    cached_reference=str(cache),cached_reference_sha256=sha(cache),previous_formal_sha256=sha(old_jar),source_manifest_sha256=w['current_product_identity']['source_manifest_sha256'],
    class_comparison=classes,selected_method_comparison=comparison,commands=commands,previous_disassemblies=raw_files,
    old_runtime_qualifications_transfer_to_new_formal=False,runtime_compatibility_verified=False,callee_closure_compatibility_proven=False,
    tests_executed=0,game_or_fixture_executions=0,desktop_inputs=0,normal_source_modified=False,normal_EXEs_modified=False,
    recorded_utc=datetime.now(timezone.utc).isoformat(),whole_goal_complete=False)
save(out/'comparison.json',proof)
files={p.name:p for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in files.items())
proof.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'))
save(out/'evidence.json',proof)
assert sha(actual)==expected and all(sha(R/n)==h for n,h in inputs.items()) and all(sha(R/n)==h for n,h in before.items())
print(json.dumps(dict(evidence=str(out/'evidence.json'),formal_sha256=expected,classes_identical=sum(x['byte_identical'] for x in classes.values()),classes_compared=len(classes),
    selected_methods_identical={n:sum(x['constant_pool_normalized_identical'] for x in rows.values()) for n,rows in comparison.items()},
    selected_methods_compared={n:len(rows) for n,rows in comparison.items()},tests_executed=0),ensure_ascii=False))
