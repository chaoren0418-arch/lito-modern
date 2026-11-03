"""Read current formal request/handler bytecode and retained resource metadata."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require
V=R/'驗證紀錄/current'
def methods(text):
 starts=list(re.finditer(r'(?m)^  [^ \r\n].*(?:;|\{)\r?$',text));result={}
 for index,match in enumerate(starts):result[match.group().strip()]=text[match.start():starts[index+1].start() if index+1<len(starts) else len(text)]
 return result
def main():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['source_and_corresponding_QA_frozen'],'original workload active')
 jar=owned(P/'target/current/server-reference/formal-server-E1498871.jar');require(sha(jar)==FORMAL==sha(w['current_formal_reference_identity']['formal']),'current formal target changed')
 previous=V/'native-formal-reference-refresh/20261023-192435-953238';proof=read(previous/'evidence.json')
 require(proof['formal_sha256']==FORMAL and sha(proof['raw_archive'])==proof['raw_archive_sha256'],'retained original disassembly custody differs')
 out=owned(V/'native-E149-slow29-advance79-contract'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'scratch').mkdir()
 selected={};pins={}
 with zipfile.ZipFile(proof['raw_archive']) as z:
  for name,needles in [('ClientSession',['useAdvanceSpiritSpell(','useEntangleSpell(','useSpellBeforeDragonTargets(']),('Protocol',['objectSpellRequest(','attackSpellRequest(','spellSelector('])]:
   file=previous/(name+'.stdout.txt');require(hashlib.sha256(z.read(file.name)).hexdigest().upper()==sha(file),'original full disassembly changed')
   pins[str(file)]=sha(file);selected[name]={signature:body for signature,body in methods(file.read_text('utf-8-sig')).items() if any(needle in signature for needle in needles)}
 javap=Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javap.exe');require(sha(javap)=='09A29817B39833F5CFED07A8267C7D7DB5BFD2501513466F9C7525CDBB4CAA0A','javap changed')
 commands=[]
 for name in ['BattleSpellCatalog','NativeNegativeMaterialAction']:
  command=[str(javap),'-J-Dfile.encoding=UTF-8','-J-Djava.io.tmpdir='+str(out/'scratch'),'-p','-c','-constants','-classpath',str(jar),'local.lito381.server.'+name]
  value=subprocess.run(command,capture_output=True,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW)
  for kind,blob in [('stdout',value.stdout),('stderr',value.stderr)]:(out/(name+'.'+kind+'.txt')).write_bytes(blob)
  commands.append(dict(command=command,exit_code=value.returncode));require(value.returncode==0,'original class inspection failed')
  needles=['entangleSpell(','advanceSpiritSpell('] if name=='BattleSpellCatalog' else ['entangle(']
  selected[name]={signature:body for signature,body in methods(value.stdout.decode('utf-8-sig')).items() if any(needle in signature for needle in needles)}
 metadata=V/'native44FE-skill-book-audit/20261018-175440-043382/original-effect-definitions-20261018-181149-098541'
 mr=read(metadata/'readback.json');require(sha(metadata/'inspect.stdout')==mr['data_sha256'],'retained original effect metadata changed')
 data=read(metadata/'inspect.stdout');rows=[row for row in data['rows'] if row['graphics'] in [752,3935]];require(len(rows)==2,'both original effect definitions required')
 (out/'selected-methods.json').write_bytes(encoded(selected));(out/'original-effects.json').write_bytes(encoded(dict(catalogue_sha256=bytes(data['catalogue_sha256']).hex().upper(),rows=rows)))
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes())
 record=dict(status='original_formal_bytecode_and_retained_effect_metadata_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,jar=str(jar),
  original_reference=str(previous/'evidence.json'),original_reference_sha256=sha(previous/'evidence.json'),original_disassembly_pins=pins,
  javap=str(javap),javap_sha256=sha(javap),commands=commands,selected_methods=str(out/'selected-methods.json'),selected_methods_sha256=sha(out/'selected-methods.json'),
  original_effects=str(out/'original-effects.json'),original_effects_sha256=sha(out/'original-effects.json'),
  retained_effect_metadata=str(metadata/'inspect.stdout'),retained_effect_metadata_sha256=sha(metadata/'inspect.stdout'),
  tests_executed=0,game_classes_instantiated=0,resource_PE_executions=0,new_sessions=0,new_casts=0,desktop_inputs=0,whole_goal_complete=False)
 (out/'inspection.json').write_bytes(encoded(record));print(json.dumps(dict(inspection=str(out/'inspection.json'),javap_exit_codes=[row['exit_code'] for row in commands],tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
