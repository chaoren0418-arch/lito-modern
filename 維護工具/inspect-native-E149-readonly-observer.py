"""Inspect compiled observer bytecode only; do not instantiate any class."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,os,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('java_run',type=Path);a=ap.parse_args()
run=a.java_run.resolve(strict=True);assert run.is_relative_to(V/'native-recovery-java-compile')
c=read(run/'cohort.json');assert c['compiler_exit_code']==0 and c['formal_reference_sha256']=='E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498'
assert c['tests_executed']==c['game_or_fixture_executions']==0
assert all(sha(p)==h for p,h in c['input_pins'].items()) and all(sha(p)==h for p,h in c['classes'].items())
out=run/('readonly-bytecode-review-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'reader.py').write_bytes(Path(__file__).read_bytes())
javap=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe')
commands=[];texts={}
for name in ['NativeRecoveryReadonlySnapshot','RuntimeInventory$Item','InventoryCatalog$Definition','CharacterSummary']:
    command=[str(javap),'-J-Dfile.encoding=UTF-8','-p','-c','-constants','-classpath',str(run/'classes')+os.pathsep+c['formal_reference'],'local.lito381.server.'+name]
    result=subprocess.run(command,cwd=out,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
    (out/(name+'.stdout.txt')).write_bytes(result.stdout);(out/(name+'.stderr.txt')).write_bytes(result.stderr)
    commands.append(dict(command=command,exit_code=result.returncode));assert result.returncode==0
    texts[name]=result.stdout.decode('utf-8-sig')
observer=texts['NativeRecoveryReadonlySnapshot']
calls=[s.strip() for s in observer.splitlines() if re.search(r'\binvoke(?:virtual|static|interface|special)\b',s)]
for call in calls:
    comment=call.split('//',1)[1].strip();symbol=comment.split()[1].split(':',1)[0]
    if '/' in symbol:
        assert symbol.startswith(('java/','local/lito381/server/RuntimeInventory$Item.',
            'local/lito381/server/InventoryCatalog$Definition.','local/lito381/server/CharacterSummary.',
            'local/lito381/server/NativePeerHarness$Session.')),call
    assert not re.search(r'java/lang/reflect/Field\.(?:set:|setInt:|setLong:|setBoolean:)',call)
assert not re.search(r'\bputfield\b',observer)
stores=[s.strip() for s in observer.splitlines() if re.search(r'\bputstatic\b',s)]
assert len(stores)==1 and 'observedIds:Ljava/util/IdentityHashMap;' in stores[0]
assert 'RuntimeInventory.items:' not in observer and 'NativeDeItemInventory.snapshot:' not in observer
def methods(text):
    lines=text.splitlines();starts=[i for i,s in enumerate(lines) if re.match(r'^  \S.*\(.*\).*;$',s)]
    return {lines[i]:'\n'.join(lines[i:starts[j+1] if j+1<len(starts) else len(lines)]) for j,i in enumerate(starts)}
required={
    'RuntimeInventory$Item':['objectId','definition','quantity','equipped','durability','enchantLevel','identified','blessing','sourceBound','tournamentReward','chargeCount','expiresAtEpochMillis','sealed','litoOwned'],
    'InventoryCatalog$Definition':['catalogId','name'],
    'CharacterSummary':['name','objectId'],
}
accessors={}
for name,keys in required.items():
    accessor_map={}
    for key in keys:
        matches=[(h,m) for h,m in methods(texts[name]).items() if (' '+key+'()') in h];assert len(matches)==1
        header,body=matches[0]
        instructions=re.findall(r'^\s*\d+:\s*(\w+)',body,re.M)
        assert len(instructions)==3 and instructions[:2]==['aload_0','getfield'] and instructions[2] in ['areturn','ireturn','lreturn']
        accessor_map[header]=hashlib.sha256(body.encode('utf-8')).hexdigest().upper()
    accessors[name]=accessor_map
(out/'commands.json').write_bytes(data(commands))
record=dict(status='compiled_E149_observer_selected_readonly_paths_statically_reviewed',java_compile_run=str(run),java_compile_cohort_sha256=sha(run/'cohort.json'),
    candidate=c['candidate'],candidate_sha256=c['candidate_sha256'],formal_reference_sha256=c['formal_reference_sha256'],
    observer_classes=c['classes'],external_method_calls=calls,record_field_accessors_verified=accessors,
    no_gameplay_method_calls_in_observer=True,no_reflective_game_field_setters=True,no_view_cache_refresh=True,
    only_static_field_write='observer-owned identity map initialization',
    caveats=['Field layout is pinned to E149; runtime getter/field availability has not been exercised',
        'Repeated reads/revisions do not establish an atomic gameplay snapshot',
        'Production item application activation and complete live owner/item/wire continuity remain unqualified'],
    tests_executed=0,classes_instantiated=0,GUI_inputs=0,whole_goal_complete=False)
(out/'review.json').write_bytes(data(record))
files={p.name:p for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in files.items())
(out/'readback.json').write_bytes(data(dict(review=str(out/'review.json'),review_sha256=sha(out/'review.json'),raw_archive_sha256=sha(out/'raw.zip'))))
assert all(sha(p)==h for p,h in c['input_pins'].items()) and all(sha(p)==h for p,h in c['classes'].items())
print(json.dumps(dict(review=str(out/'review.json'),getter_accessors=sum(len(v) for v in accessors.values()),tests_executed=0,classes_instantiated=0),ensure_ascii=False))
