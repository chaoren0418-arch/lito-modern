"""Bind #45 target/wire/bootstrap/resource override source without executing game code."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require,check_pins
from native_E149_classfile_bootstraps import bootstraps
T=R/'維護工具';V=R/'驗證紀錄/current'
def methods(path):
 s=Path(path).read_text('utf-8-sig');hs=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M))
 return [(h.group().strip(),s[h.start():hs[i+1].start() if i+1<len(hs) else len(s)]) for i,h in enumerate(hs)]
def inspect():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development-only idle expected')
 prior=w['current_E149_eruption45_scene_development'];require(sha(prior['qualification'])==prior['qualification_sha256'],'scene predecessor changed');q=read(prior['qualification']);check_pins(q['source_pins']);k=read(q['contract']);proof=read(k['inspection']);require(sha(k['inspection'])==k['inspection_sha256'],'scene proof changed')
 out=V/'native-E149-eruption45-entry-inspection'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL,'formal jar differs')
 with zipfile.ZipFile(jar) as z:data=z.read('local/lito381/server/ClientSession.class')
 parsed,indy=bootstraps(data);selected={str(index):indy[index] for index in [35958,45726]}
 require(selected['35958']['method_handles']==[dict(kind=5,owner='local/lito381/server/ClientSession',name='lambda$useClassicWorldAttackSpell$360',descriptor='(Llocal/lito381/server/Protocol$AttackSpellRequest;Llocal/lito381/server/ClientSession;)Z')],'classic target bootstrap differs')
 require(len(selected['45726']['method_handles'])==1 and selected['45726']['method_handles'][0]['name']=='repair020$lambda$useAttackSpell$350','battle target bootstrap differs')
 (out/'bootstrap-targets.json').write_bytes(encoded(dict(ClientSession_class_sha256=hashlib.sha256(data).hexdigest().upper(),targets=selected)))
 source=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';require(sha(source)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','session differs');all_methods=methods(source);selected_methods={}
 for h,s in all_methods:
  if any(' '+name+'(' in h for name in ['lambda$useClassicWorldAttackSpell$360','repair020$lambda$useAttackSpell$350']):selected_methods[h]=s
 for index in [1,6]:
  p=Path(k['inspection']).parent/('ClientSession-method-'+str(index)+'.txt');s=p.read_text('utf-8-sig');selected_methods[s.splitlines()[0].strip()]=s
 classic=next(s for h,s in selected_methods.items() if h.startswith('private void useClassicWorldAttackSpell('));battle=next(s for h,s in selected_methods.items() if h.startswith('private void useAttackSpell('))
 expected=[(classic,{204:'learnedSkills:',247:'AttackSpell.canCastAtLawful:',1242:'BattleSpellCatalog.isInNativeScreen:',1263:'AttackSpell.maximumRange:',1266:'ClassicWorldMapCatalog$ClassicMap.attackPosition:',2342:'AttackSpell.reuseDelayMillis:',4251:'Protocol.attackSpell:'}),
  (battle,{596:'ifne          615',614:'return',903:'BattleSpellCatalog.isInNativeScreen:',926:'BattleSpellCatalog.hasNativeLineOfSight:',953:'java/lang/Math.max:',961:'AttackSpell.maximumRange:',1885:'AttackSpell.reuseDelayMillis:',4043:'Protocol.attackSpell:'})]
 for body,rows in expected:
  for offset,needle in rows.items():require(re.search(r'^\s*'+str(offset)+r':[^\n]*'+re.escape(needle),body,re.M),'formal offset differs '+str(offset))
 for i,(header,body) in enumerate(selected_methods.items()):(out/('method-'+str(i)+'.txt')).write_text(body,encoding='utf-8')
 javap=Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javap.exe');require(sha(javap)=='09A29817B39833F5CFED07A8267C7D7DB5BFD2501513466F9C7525CDBB4CAA0A','javap differs');command=[str(javap),'-classpath',str(jar),'-c','-p','local.lito381.server.ResourceDatabase'];run=subprocess.run(command,capture_output=True,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS);(out/'ResourceDatabase.stdout.txt').write_bytes(run.stdout);(out/'ResourceDatabase.stderr.txt').write_bytes(run.stderr);require(run.returncode==0,'javap failed; retain output')
 pins={str(p):sha(p) for p in out.iterdir() if p.is_file()};pins.update({str(p):sha(p) for p in [source,jar,Path(k['inspection']),T/'native_E149_classfile_bootstraps.py',T/'native_E149_classfile_scan.py']})
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins[str(out/'inspector.py')]=sha(out/'inspector.py')
 record=dict(schema='E149-eruption45-entry-primary-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=prior,source_pins=pins,
  javap_tool=dict(path=str(javap),sha256=sha(javap)),javap_command=command,javap_exit_code=0,selected_target_bootstraps=selected,
  literal_definition=proof['definition'],wire_prefix=[20,5,4],wire_bytes=11,self_rejected=True,required_other_visible_live_actor=True,base_range=8,base_reuse_millis=100,
  server_authority=['learned book and current state','actual ResourceDatabase overrides and effective MP','lawful, weight, silence, polymorph, same map, line of sight and target kind/attack eligibility','damage, hit and HP; no local cost or success prediction'],
  tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,whole_goal_complete=False)
 (out/'inspection.json').write_bytes(encoded(record));print(json.dumps(dict(inspection=str(out/'inspection.json'),sha256=sha(out/'inspection.json'),bootstrap_targets_bound=2,javap_exit_code=0),ensure_ascii=False))
if __name__=='__main__':inspect()
