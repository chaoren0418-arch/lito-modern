"""Static #45 wire/dispatch and original scheduler evidence; no game classes run."""
from pathlib import Path
from datetime import datetime,timezone
import capstone,json,re,struct,subprocess
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'
def methods(path):
 s=Path(path).read_text('utf-8-sig');hs=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M))
 return [(h.group().strip(),s[h.start():hs[i+1].start() if i+1<len(hs) else len(s)]) for i,h in enumerate(hs)]
def inspect():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development-only idle expected')
 prior=w['current_E149_eruption45_core_development'];require(sha(prior['qualification'])==prior['qualification_sha256'],'core changed');q=read(prior['qualification']);c=read(q['contract']);old=read(c['inspection']);check_pins(old['files']);require(all(sha(p)==h for p,h in old['original_pins'].items()),'primary input changed')
 out=V/'native-E149-eruption45-scene-inspection'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 pe=Path(next(p for p in old['original_pins'] if p.endswith('.exe')));b=pe.read_bytes();e=struct.unpack_from('<I',b,60)[0];ss=e+24+struct.unpack_from('<H',b,e+20)[0]
 def raw(a,n):
  for i in range(struct.unpack_from('<H',b,e+6)[0]):
   size,start,physical,at=struct.unpack_from('<IIII',b,ss+i*40+8)
   if start<=a-0x400000 and a-0x400000+n<=start+min(size,physical):return b[at+a-0x400000-start:at+a-0x400000-start+n]
  raise ValueError(hex(a))
 ranges={'dispatch':(0x4d6570,0x4d65f0),'insert':(0x4d65f0,0x4d66f0),'clock':(0x4d69f0,0x4d6a40),'delay_parser':(0x43e888,0x43e911)}
 md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);ops={};pins={}
 for name,(lo,hi) in ranges.items():
  data=raw(lo,hi-lo);rows=list(md.disasm(data,lo));require(sum(i.size for i in rows)==len(data) and rows[0].address==lo and all(a.address+a.size==b.address for a,b in zip(rows,rows[1:])),'partial decode '+name)
  (out/(name+'.bin')).write_bytes(data);(out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in rows)+'\n',encoding='utf-8');ops[name]={i.address:(i.mnemonic,i.op_str) for i in rows}
  for ext in ['.bin','.asm']:pins[str(out/(name+ext))]=sha(out/(name+ext))
 require(ops['insert'][0x4d6625]==('cmp','dword ptr [edx], esi') and ops['insert'][0x4d6627]==('jbe','0x4d6631') and ops['insert'][0x4d669a]==('lea','esi, [ebx + ebx*2 + 3]'),'stable insertion changed')
 require(('call','esi') in ops['dispatch'].values() and ('jb','0x4d65dc') in ops['dispatch'].values(),'front dispatch changed')
 jar=P/'target/current/server-reference/formal-server-E1498871.jar';javap=Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javap.exe');require(sha(jar)==FORMAL and sha(javap)=='09A29817B39833F5CFED07A8267C7D7DB5BFD2501513466F9C7525CDBB4CAA0A','javap/formal identity changed');commands=[]
 for name in ['SharedClientResources','BattleSpellCatalog$AttackSpell']:
  command=[str(javap),'-classpath',str(jar),'-c','-p','local.lito381.server.'+name];run=subprocess.run(command,capture_output=True,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS)
  (out/(name+'.stdout.txt')).write_bytes(run.stdout);(out/(name+'.stderr.txt')).write_bytes(run.stderr);require(run.returncode==0,'original javap failure; retain output');logs={str(out/(name+'.'+side+'.txt')):sha(out/(name+'.'+side+'.txt')) for side in ['stdout','stderr']};pins.update(logs);commands.append(dict(command=command,exit_code=run.returncode,logs=logs))
 source=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';protocol=V/'native-E149-area2253053-contract/Protocol.stdout.txt';require(sha(protocol)=='9C68804A5854235F25CA81CEB357E63AE2EA656A48201AEC2A93EB0150F4F18D','protocol changed')
 selected=[(h,s) for h,s in methods(source) if re.search(r'\b(useSpellBeforeDragonTargets|useAttackSpell|useClassicWorldAttackSpell)\(',h)]
 dispatch=next(s for h,s in selected if ' useSpellBeforeDragonTargets(' in h)
 for offset,needle in {4671:'BattleSpellCatalog.attackSpell:',4755:'useClassicWorldAttackSpell:',6909:'BattleSpellCatalog.attackSpell:',6993:'useAttackSpell:'}.items():require(re.search(r'^\s*'+str(offset)+r':[^\n]*'+re.escape(needle),dispatch,re.M),'dispatch offset changed')
 for i,(h,s) in enumerate(selected):
  p=out/('ClientSession-method-'+str(i)+'.txt');p.write_text(s,encoding='utf-8');pins[str(p)]=sha(p)
 pm=dict((re.search(r'([\w$]+)\(',h)[1],s) for h,s in methods(protocol))
 for name in ['attackSpell','attackSpellRequest','spellSelector']:
  p=out/('Protocol.'+name+'.txt');p.write_text(pm[name],encoding='utf-8');pins[str(p)]=sha(p)
 require([pm['attackSpellRequest'].count('Protocol$Reader.'+n+':') for n in ['c','d','h','remaining']]==[3,1,2,1],'request shape changed')
 require(re.search(r'^\s*258: iconst_0\s*\n\s*259:[^\n]*Writer.c:',pm['attackSpell'],re.M),'wire flight type not literal0')
 cq=read(w['current_E149_skill_catalog_development']['qualification']);facts=read(cq['facts']);require(sha(cq['facts'])==cq['facts_sha256'],'catalog changed');definition=next(x for x in facts['definitions_by_id'] if x['skill_id']==45);require(len(definition['definitions'])==1 and definition['definitions'][0]['record_type']=='AttackSpell','45 class differs')
 branches={}
 for name in ['attackSpell','thunderGrabSpell','confusionSpell','smashSpell','mindBreakSpell','boneBreakSpell','armBreakerSpell']:
  factory=next(x for x in facts['factories'] if x['name']==name)
  if name=='attackSpell':
   prefix=factory['instructions'][:10];require([(x['offset'],x['opcode']) for x in prefix]==[(0,178),(3,26),(4,184),(7,185),(12,192),(15,76),(16,43),(17,198),(20,43),(21,176)] and prefix[0]['member']['name']=='ATTACK_SPELLS' and prefix[7]['hex']=='c60005','attack factory direct nonnull return differs');field='ATTACK_SPELLS'
  else:
   require(factory['single_map_lookup'] and len(factory['referenced_map_fields'])==1,'specialized factory not exact map lookup');field=factory['referenced_map_fields'][0]
  mapping=next(x for x in facts['static_maps'].values() if x['field']['name']==field);entries=mapping['value']['entries'];require(all(x['key']['kind']=='literal' for x in entries),'opaque map key');branches[name]=45 in [x['key']['value'] for x in entries]
 require(branches=={name:name=='attackSpell' for name in branches},'45 specialized route differs')
 for p in [source,protocol,Path(cq['facts']),Path(w['current_E149_skill_catalog_development']['qualification']),jar,javap]:pins[str(p)]=sha(p)
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins[str(out/'inspector.py')]=sha(out/'inspector.py')
 record=dict(schema='E149-eruption45-scene-primary-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=prior,source_pins=pins,PE_original_pins={str(pe):sha(pe)},definition=definition['definitions'][0]['core'],factory_membership=branches,javap_commands=commands,
  C14_prefix=[20,5,4],request_bytes=11,S23_opcode=35,S23_wire_flight_type=0,native_effect_class=7,child_graphics_source='shared attribute109:7 123, not S23 flight-type7',
  scheduler_equal_deadline_policy='FIFO: scan backwards until previous due<=new due, insert after it; dispatch removes the front before invoking callback',
  scheduler_model_limits=['Native queue uses unsigned32 ticks; long-uptime rollover equivalence not qualified.','Delay parser has optional global647EA4 scaling for action0; actual runtime configuration remains unqualified.','Analysis PE import patch targets are not proved runnable.'],
  inherited_hurt_unknown='relative original graphics family/initialization remains unresolved; do not call generic hurt as a substitute',
  game_classes_loaded=0,game_methods_invoked=0,tests_executed=0,outputs_executed=False,whole_goal_complete=False)
 (out/'inspection.json').write_bytes(encoded(record));print(json.dumps(dict(inspection=str(out/'inspection.json'),sha256=sha(out/'inspection.json'),FIFO_bound=True,S23_type=0,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':inspect()
