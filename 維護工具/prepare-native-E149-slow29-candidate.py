"""Develop slow request/menu/exact finite frames without selecting or running it."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,difflib,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
V=R/'驗證紀錄/current';T=R/'維護工具'
def section(text,needle):
 starts=list(re.finditer(r'(?m)^  [^ \r\n].*(?:;|\{)\r?$',text));found=[]
 for index,match in enumerate(starts):
  if needle in match.group():found.append(text[match.start():starts[index+1].start() if index+1<len(starts) else len(text)])
 require(len(found)==1,'original method is missing/ambiguous: '+needle);return found[0]
def literals(block):
 values=[]
 for line in block.splitlines():
  match=re.match(r'\s*\d+: (iconst_m1|iconst_[0-5]|bipush|sipush|ldc2_w|ldc_w|ldc)\s*(.*)',line)
  if not match:continue
  op,value=match.groups()
  if op.startswith('iconst_'):values.append(-1 if op=='iconst_m1' else int(op[-1]))
  elif op in ['bipush','sipush']:values.append(int(value))
  else:
   kind,value=value.split('// ',1)[1].split(' ',1)
   values.append(value if kind=='String' else float(value.rstrip('df')) if kind in ['double','float'] else int(value.rstrip('l')))
 return values
def prepare(inspection):
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'original workload active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only preparation')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed')
 inspection=owned(inspection.resolve(strict=True));proof=read(inspection)
 require(proof['formal_target_sha256']==FORMAL and all(row['exit_code']==0 for row in proof['commands']),'original E149 inspection required')
 check_pins(proof['original_disassembly_pins']);require(sha(proof['original_effects'])==proof['original_effects_sha256'],'original metadata changed')
 selected=read(proof['selected_methods']);require(sha(proof['selected_methods'])==proof['selected_methods_sha256'],'original selected methods changed')
 client_path=next(Path(name) for name in proof['original_disassembly_pins'] if Path(name).name=='ClientSession.stdout.txt')
 body=section(client_path.read_text('utf-8-sig'),' nativeMaterialEntangleBody(')
 for marker in ['Protocol.attackSpellRequest:','EntangleSpell.maximumRange:','Math.abs:','Math.max:',
  'ClassicMap.attackPosition:','BattleSpellCatalog.isInNativeScreen:','NativeNegativeMaterialAction.admit:',
  'EntangleSpell.reuseDelayMillis:','BattleTimedStatus.activateSlow:','Protocol.speed:','Protocol.effectObject:']:
  require(marker in body,'original slow handler no longer contains '+marker)
 attack=next(value for signature,value in selected['Protocol'].items() if ' attackSpellRequest(' in signature)
 require(attack.count('Protocol$Reader.d:')==1 and attack.count('Protocol$Reader.h:')==2 and 'invalid native attack C_USE_SPELL layout' in attack,'target/coordinate wire shape differs')
 base_ref=w['current_isolated_development']['candidates']['blind20'];base_path=Path(base_ref['path']);require(sha(base_path)==base_ref['sha256'],'previous #20 candidate changed')
 base=read(base_path);check_pins({str(base_path.parent/name):h for name,h in base['candidate_files'].items()})
 out=owned(T/'candidates'/('native-E149-slow29-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir();(out/'scratch').mkdir()
 javap=Path(proof['javap']);require(sha(javap)==proof['javap_sha256'],'original read-only javap changed')
 command=[str(javap),'-J-Dfile.encoding=UTF-8','-J-Djava.io.tmpdir='+str(out/'scratch'),'-p','-c','-constants','-classpath',proof['jar'],'local.lito381.server.BattleSpellCatalog$EntangleSpell']
 result=subprocess.run(command,capture_output=True,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW)
 (out/'EntangleSpell.stdout.txt').write_bytes(result.stdout);(out/'EntangleSpell.stderr.txt').write_bytes(result.stderr)
 require(result.returncode==0,'original record component inspection failed')
 fields=re.findall(r'(?m)^  private final [^ ]+ (\w+);',result.stdout.decode('utf-8-sig'))
 catalog=(inspection.parent/'BattleSpellCatalog.stdout.txt').read_text('utf-8-sig');end=catalog.index('// String slow')
 # javap aligns comments with spaces; find by the stable string label instead.
 start=catalog.rfind('new ',0,end);block=catalog[start:catalog.index('invokespecial',end)]
 values=literals(block);require(len(fields)==len(values),'original record constructor/component count differs')
 definition=dict(zip(fields,values));require(definition['skillId']==29 and definition['row']==3 and definition['column']==4
  and definition['maximumRange']==8 and definition['reuseDelayMillis']==500 and definition['effectGfx']==752
  and definition['itemConsumeId']==40318 and definition['itemConsumeCount']==1,'actual E149 slow definition differs')
 effects=read(proof['original_effects']);slow=next(row for row in effects['rows'] if row['graphics']==752)
 frames=slow['action0']['frames'];require(slow['graphics_bank']==752 and slow['header_issue'] is None and slow['image_count']==1
  and slow['action0']['definition_sprite']==752 and slow['action0']['directionality']=='Undirected' and slow['action0']['fps']==24
  and len(frames)==16 and [frame['subframe'] for frame in frames]==[7,0,1,2,3,4,5,6,7,0,1,2,3,4,5,6]
  and all(frame['image']==0 and frame['ticks']==3 and frame['events']==[] for frame in frames),'original 752 finite definition differs')
 original={name:(base_path.parent/name).read_bytes() for name in base['candidate_files']};after=dict(original)
 skills='LiTo381-Modern/src/modern_lito/native_skills.rs';menu='LiTo381-Modern/src/modern_lito/native_skill_selection.rs'
 replacements={skills:[('18 | 20 | 28 | 34','18 | 20 | 28 | 29 | 34'),('11|20=>1,28=>4','11|20=>1,28=>4,29=>8'),('18|20|49|57','18|20|29|49|57')],
  menu:[('                        26 =>','                        29 => "#29 緩速術（選取目標，八格內；消耗 1 顆魔法寶石）".into(),\n                        26 =>')]}
 for name,edits in replacements.items():
  text=after[name].decode('utf-8-sig')
  for before,replacement in edits:require(text.count(before)==1,'candidate insertion anchor differs');text=text.replace(before,replacement)
  after[name]=text.encode('utf-8')
 finite='LiTo381-Modern/src/modern_lito/native_additional_finite_effects.rs';original[finite]=(R/finite).read_bytes();text=original[finite].decode('utf-8-sig')
 anchor='pub const GRAPHICS:&[u16]=&[746,750,755,763,2171,2176,2177,2232,2236,2251,6342];';require(text.count(anchor)==1,'original finite bank changed')
 text=text.replace(anchor,anchor[:-2]+',752];')
 literal=lambda values:'&['+','.join(str(value) for value in values)+']'
 spec='Spec{sprite:752,bank:752,header:'+literal(slow['header_bytes'])+',attrs:&[(104,&[56])],frames:&['+','.join('(0,'+str(frame['subframe'])+',3,&[])' for frame in frames)+']},\n'
 require(text.count('];\n\npub fn qualify')==1,'finite spec insertion anchor differs');text=text.replace('];\n\npub fn qualify',spec+'];\n\npub fn qualify');after[finite]=text.encode('utf-8')
 test='LiTo381-Modern/tests/native_slow29.rs';require(not (R/test).exists(),'new test name already exists');after[test]=(T/'templates/native_slow29.rs').read_bytes()
 contract=dict(formal_target_sha256=FORMAL,inspection=str(inspection),inspection_sha256=sha(inspection),actual_E149_definition=definition,
  current_handler_body=body,current_handler_body_sha256=hashlib.sha256(body.encode('utf-8')).hexdigest().upper(),
  original_effect_catalogue_sha256=effects['catalogue_sha256'],effect752=slow,
  advance79_remaining='3935 has Effect(1356) and attr109(0 -1); child-effect/placement semantics need original-renderer evidence before enabling its request/menu',
  client_does_not_spend_MP_or_reagent=True,server_controls_probability_duration_and_native_lifetime=True,
  code_not_selected_or_applied=True,tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract))
 for name,blob in after.items():
  path=owned(out/name,out);path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(blob)
  if blob!=original.get(name):
   (out/(Path(name).name+'.diff')).write_text(''.join(difflib.unified_diff(original.get(name,b'').decode('utf-8-sig').splitlines(True),blob.decode('utf-8').splitlines(True),fromfile='prior/'+name,tofile='slow-candidate/'+name)),encoding='utf-8')
 for name,files in [('before.zip',original),('candidate.zip',after)]:
  with zipfile.ZipFile(out/name,'x',zipfile.ZIP_DEFLATED) as z:
   for path,blob in files.items():z.writestr(path,blob)
  with zipfile.ZipFile(out/name) as z:require(all(z.read(path)==blob for path,blob in files.items()),'candidate archive readback differs')
 record=dict(status='isolated_slow29_extension_not_selected_not_compiled_not_tested',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,
  predecessor_blind20_candidate=str(base_path),predecessor_blind20_candidate_sha256=base_ref['sha256'],contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),
  candidate_files={name:hashlib.sha256(blob).hexdigest().upper() for name,blob in after.items()},base_files={name:hashlib.sha256(blob).hexdigest().upper() for name,blob in original.items()},
  changed_runtime_files=[skills,menu,finite],retained_blind20_tests_unchanged=True,new_tests_written=6,retained_blind20_tests_written=6,
  request=[20,3,4,'target u32 LE','current x/y u16 LE'],reuse_millis=500,maximum_range=8,new_finite_graphics=752,
  normal_product_sources_unchanged=True,registered_thirteen_file_overlay_unchanged=True,existing_images_qualifications_not_transferred=True,
  tests_executed=0,PEs_executed=False,GUI_verified=False,production_applied=False,whole_goal_complete=False,
  pending=['full graph Debug/Release metadata compilation','all component/resource/formal/GUI cases after user resumes tests',
   'independent merge/selection into keyboard and recovery overlays; current selected plans remain unchanged','79 child-effect and attr109 semantics'])
 (out/'candidate.json').write_bytes(encoded(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
 print(json.dumps(dict(candidate=str(out/'candidate.json'),runtime_files=3,new_tests_written=6,retained_blind_tests=6,selected=False,tests_executed=0),ensure_ascii=False))
def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('inspection',type=Path);args=ap.parse_args();prepare(args.inspection)
if __name__=='__main__':main()
