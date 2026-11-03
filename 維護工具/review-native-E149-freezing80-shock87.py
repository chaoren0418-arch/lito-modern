"""Bind original80/87 protocol and explicitly retain unresolved4183 loop presentation."""
from pathlib import Path
from datetime import datetime,timezone
import json,re,zipfile,capstone
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require,check_pins
from native_E149_classfile_scan import parse,instructions
from importlib.util import spec_from_file_location,module_from_spec
T=R/'維護工具';V=R/'驗證紀錄/current'
def main():
 primary=V/'native-E149-freezing80-shock87-inspection/20261024-172131-694095/inspection.json';require(sha(primary)=='DBD1EA10259076A7F93FA61CA8D917228A2359FE42B7D0C9E9C00038D63CD597','primary changed');j=read(primary);check_pins(j['source_pins']);require(all(sha(p)==h for p,h in j['external_readonly_pins'].items()),'readonly original changed')
 out=V/'native-E149-freezing80-shock87-review'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 def body(name):
  rows=[Path(p).read_text('utf-8-sig')for h,p in j['method_files'].items()if ' '+name+'('in h];require(len(rows)==1,'method differs '+name);return rows[0]
 for name,checks in [('useCenteredFreezeAttackSpell',{7:'Protocol.selectorOnlySpellRequest:',2626:'iconst_0',2629:'Protocol.attackMany:'}),('useClassicWorldCenteredFreezeAttackSpell',{2953:'Protocol.attackMany:'}),('useShockStunSpell',{14:'Protocol.attackSpellRequest:',572:'supportsWalkAction:',1907:'CombatPlayerDamage.shockWire:',1920:'Protocol.effectObject:',2235:'activateShockStun:',2485:'Protocol.shockStunEffect:'}),('useClassicWorldShockStunSpell',{4:'Protocol.attackSpellRequest:',29:'supportsWalkAction:',2143:'Protocol.effectObject:',2496:'activateShockStun:',2712:'Protocol.shockStunEffect:'}),('shockStunEffect',{22:'bipush        11',48:'sipush        4183'})]:
  for at,text in checks.items():require(re.search(r'^\s*'+str(at)+r':[^\n]*'+re.escape(text),body(name),re.M),'original offset differs '+name+str(at))
 spec=spec_from_file_location('original_method_copy',T/'inspect-native-E149-freezing80-shock87.py');m=module_from_spec(spec);spec.loader.exec_module(m);protocol=V/'native-E149-area2253053-contract/Protocol.stdout.txt'
 for h,s in m.methods(protocol):
  if any(' '+n+'('in h for n in ['shockStunAttack','physicalDamageAttack']):(out/(h.split('(')[0].split()[-1]+'.txt')).write_text(s,encoding='utf-8',newline='\n')
 jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL,'jar changed')
 with zipfile.ZipFile(jar)as z:
  for name in ['CombatPlayerDamage','Protocol$ParalysisMode']:
   c=parse(z.read('local/lito381/server/'+name+'.class'));rows=[dict(name=a['name'],descriptor=a['descriptor'],instructions=instructions(a['code']['bytes'],c['cp']))for a in c['methods']if a['code']];(out/(name+'-Code.json')).write_bytes(encoded(rows))
 start=V/'native-E149-blizzard59-inspection/20261024-115029-671040/start.bin';require(sha(start)=='6937E97AE98ACA01232582E90F70DBD2BA2D656D92E92837105A233B6A318A77','native start changed');md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);code={i.address:i for i in md.disasm(start.read_bytes(),0x44f350)};pc=0x44f370;eq=None;trace=[]
 while pc!=0x45001a:
  require(pc in code and len(trace)<40,'native route bound');i=code[pc];trace.append(dict(address=pc,hex=i.bytes.hex(),mnemonic=i.mnemonic,operands=i.op_str));n=pc+i.size
  if i.mnemonic=='mov':require(i.op_str=='ax, word ptr [esi + 0x14]','unreviewed load')
  elif i.mnemonic=='cmp':a,b=i.op_str.split(', ');require(a=='ax','unreviewed cmp');eq=3933==int(b,0)
  elif i.mnemonic in ['je','jne']:require(eq is not None,'missing cmp');n=int(i.op_str,0)if eq==(i.mnemonic=='je')else n
  else:raise RuntimeError('unreviewed special native route')
  pc=n
 (out/'start3933-static-route.json').write_bytes(encoded(dict(graphics=3933,trace=trace,default_start=pc,PE_executed=False)))
 pe=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe');require(sha(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9','PE changed')
 w=read(R/'工作佇列.json');q=read(w['current_E149_greater75_massslow76_development']['qualification']);k=read(q['contract']);animator=Path(k['effective_overlay_paths']['LiTo381-Modern/src/modern_lito/native_world_animation.rs']);s=animator.read_text('utf-8-sig');require('let base_walk = self.clip(catalog, id, stance)?.duration_ns();'in s and 'label.starts_with("breath") || label.starts_with("idle")'in s,'current actor path changed')
 raw80=Path(j['raw_definitions']['3933']['path']).read_bytes();raw87=Path(j['raw_definitions']['4434']['path']).read_bytes();rawnpc=Path(j['raw_definitions']['4183']['path']).read_bytes();require(b'0.0:4<1353'in raw80 and b'!'not in raw80 and b'109.effect'not in raw80 and b'0.1:1[1498'in raw87 and b'3.animation'in rawnpc and b'0.0:4<1453'in rawnpc and b'0.animation'not in rawnpc,'original resources differ')
 (out/'pending4183.json').write_bytes(encoded(dict(actor_source=str(animator),actor_source_sha256=sha(animator),original_effect_definition=j['raw_definitions']['4183'],known_source_gap='generic actor stance0 resolution requests missing action0; action3 also has animation label, not generic idle/breath; no qualified repeating overlap1453 callback path yet',full_persistent_object_presentation_complete=False,required_next='bind original S0B4183 constructor/action3 repeat callbacks and per-incarnation typed overlap1453/History; exact S08 retirement, no local2-4s expiry',tests_executed=0)))
 (out/'reviewer.py').write_bytes(Path(__file__).read_bytes());pins=dict(j['source_pins']);pins.update({str(p):sha(p)for p in out.iterdir()});pins.update({str(p):sha(p)for p in [primary,start,animator]});external=dict(j['external_readonly_pins']);external[str(pe)]=sha(pe);p=out/'review.json';p.write_bytes(encoded(dict(schema='E149-freezing80-shock87-reviewed-primary-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,primary=str(primary),primary_sha256=sha(primary),source_pins=pins,external_readonly_pins=external,definitions=j['definitions'],raw_definitions=j['raw_definitions'],freezing80_selector_only_S39_type0_wire_caster_origin=True,freezing80_single_default_start_thirtyone_frames_overlap1353_no_Hit=True,shock87_actor_xy_range1_walk50_or54=True,shock87_physical_wire_and_S37_4434_separate_from_stun_success=True,shock87_S0B4183_server_object_and_paralysis_authoritative=True,shock87_4434_six_frames_Sound1498_source2=True,damage_cost_probability_and_duration_server_only=True,shock87_persistent_object_loop_source_complete=False,original_4183_callback_loop_binding_pending=True,constructor_clock_and_global_interleaving_qualified=False,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(review=str(p),sha256=sha(p),pending4183=str(out/'pending4183.json'),tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
