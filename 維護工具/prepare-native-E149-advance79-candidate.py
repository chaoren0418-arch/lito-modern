"""Extend the unselected slow/blind candidate; data preparation and javap only."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,difflib,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
from importlib.util import spec_from_file_location,module_from_spec
T=R/'維護工具';V=R/'驗證紀錄/current'

def prepare(inspection,audio):
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'original workload active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'development only')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal changed')
 inspection=owned(inspection.resolve(strict=True));proof=read(inspection);audio=owned(audio.resolve(strict=True));ap=read(audio)
 check_pins(proof['original_disassembly_pins']);require(proof['formal_target_sha256']==ap['formal_target_sha256']==FORMAL,'evidence target differs')
 require(sha(proof['original_effects'])==proof['original_effects_sha256'] and sha(proof['selected_methods'])==proof['selected_methods_sha256'],'original metadata changed')
 require(ap['PE_executions']==ap['WAV_playbacks']==ap['tests_executed']==0 and ap['sound']['path'].endswith('1356.wav'),'offline source evidence required')
 require(sha(ap['original_PE'])==ap['original_PE_sha256'] and sha(ap['sound']['path'])==ap['sound']['sha256'],'original PE/WAV changed')
 for name,row in ap['code_ranges'].items():require(sha(audio.parent/(name+'.bytes'))==row['sha256'],'original code bytes changed')
 parser=(audio.parent/'catalog_parser_43DE10.asm').read_text('utf-8');dispatch=(audio.parent/'frame_dispatch_4E5C00.asm').read_text('utf-8')
 require('0043E977' in parser and 'cmp al, 0x3c' in parser and 'mov byte ptr [edx + edi + 0x10], 0' in parser and '004E61D1' in dispatch and 'call 0x4c7bf0' in dispatch,'native overlap source branch missing')
 spec=spec_from_file_location('slow_preparer',T/'prepare-native-E149-slow29-candidate.py');helper=module_from_spec(spec);spec.loader.exec_module(helper)
 selected=read(proof['selected_methods']);body=next(value for name,value in selected['ClientSession'].items() if ' useAdvanceSpiritSpell(' in name)
 for marker in ['Protocol.objectSpellRequest:','BattleSpellCatalog.isInNativeScreen:','activateAdvanceSpirit:','rebuildMaximumVitals:','Protocol.effectObject:']:
  require(marker in body,'current advance handler differs: '+marker)
 base_ref=w['current_native_slow29_development'];q=read(base_ref['qualification']);require(sha(base_ref['qualification'])==base_ref['qualification_sha256'],'slow qualification changed')
 base_path=Path(q['candidate']);require(sha(base_path)==q['candidate_sha256'],'slow candidate changed');base=read(base_path);check_pins({str(base_path.parent/n):h for n,h in base['candidate_files'].items()})
 out=owned(T/'candidates'/('native-E149-advance79-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir();(out/'scratch').mkdir()
 javap=Path(proof['javap']);require(sha(javap)==proof['javap_sha256'],'javap changed')
 command=[str(javap),'-J-Dfile.encoding=UTF-8','-J-Djava.io.tmpdir='+str(out/'scratch'),'-p','-c','-constants','-classpath',proof['jar'],'local.lito381.server.BattleSpellCatalog$AdvanceSpiritSpell']
 result=subprocess.run(command,capture_output=True,timeout=60,creationflags=subprocess.CREATE_NO_WINDOW)
 (out/'AdvanceSpiritSpell.stdout.txt').write_bytes(result.stdout);(out/'AdvanceSpiritSpell.stderr.txt').write_bytes(result.stderr);require(result.returncode==0,'record component inspection failed')
 fields=re.findall(r'(?m)^  private final [^ ]+ (\w+);',result.stdout.decode('utf-8-sig'));catalog=(inspection.parent/'BattleSpellCatalog.stdout.txt').read_text('utf-8-sig');end=catalog.index('// String advance-spirit');start=catalog.rfind('new ',0,end)
 values=helper.literals(catalog[start:catalog.index('invokespecial',end)]);require(len(fields)==len(values),'actual record component count differs');definition=dict(zip(fields,values))
 require(definition['skillId']==79 and definition['row']==9 and definition['column']==6 and definition['effectGfx']==3935 and definition['reuseDelayMillis']==10,'actual E149 advance definition differs')
 effect=next(row for row in read(proof['original_effects'])['rows'] if row['graphics']==3935);frames=effect['action0']['frames']
 require(effect['graphics_bank']==3935 and effect['image_count']==1 and effect['header_issue'] is None and effect['action0']['fps']==24 and effect['action0']['directionality']=='Undirected' and len(frames)==19,'original 3935 shape differs')
 require(frames[1]['events']==['Effect(1356)'] and all(not f['events'] for i,f in enumerate(frames) if i!=1),'original overlap event differs')
 original={n:(base_path.parent/n).read_bytes() for n in base['candidate_files']};after=dict(original)
 skills='LiTo381-Modern/src/modern_lito/native_skills.rs';menu='LiTo381-Modern/src/modern_lito/native_skill_selection.rs';selector='LiTo381-Modern/src/modern_lito/native_selector_effects.rs'
 for name,edits in {skills:[('1 | 8 | 9 | 19 | 26 | 35 | 42 | 43 | 57 =>','1 | 8 | 9 | 19 | 26 | 35 | 42 | 43 | 57 | 79 =>'),('matches!(skill, 8 | 9 | 26 | 42 | 43)','matches!(skill, 8 | 9 | 26 | 42 | 43 | 79)')],menu:[('                        26 =>','                        79 => "#79 靈魂昇華（未選取目標時對自己）".into(),\n                        26 =>')]}.items():
  text=after[name].decode('utf-8-sig')
  for anchor,replacement in edits:require(text.count(anchor)==1,'candidate anchor differs');text=text.replace(anchor,replacement)
  after[name]=text.encode('utf-8')
 original[selector]=(R/selector).read_bytes();text=original[selector].decode('utf-8-sig');anchor='pub const GRAPHICS:[u16;9]=[2170,10701,2173,759,3936,3104,3934,2234,6527];';require(text.count(anchor)==1,'strict original selectors differ')
 text=text.replace('//! Exact nine original','//! Exact ten original').replace(anchor,'pub const GRAPHICS:[u16;10]=[2170,10701,2173,759,3936,3104,3934,2234,6527,3935];')
 literal=lambda values:'&['+','.join(map(str,values))+']'
 entry='    Spec{sprite:3935,bank:3935,header:'+literal(effect['header_bytes'])+',attributes:&[(102,b"type",b"0"),(104,b"attr",b"8"),(109,b"effect",b"0 -1")],frames:&[\n'
 for frame in frames:entry+='        ('+','.join(map(str,[frame['image'],frame['subframe'],frame['ticks']]))+',&['+','.join('FrameEvent::'+event for event in frame['events'])+']),\n'
 entry+='    ]},\n';require(text.count(' ];\npub fn qualify')==1,'selector insertion anchor differs');text=text.replace(' ];\npub fn qualify',entry+' ];\npub fn qualify');after[selector]=text.encode('utf-8')
 test='LiTo381-Modern/tests/native_advance79.rs';require(not (R/test).exists(),'new test exists');after[test]=(T/'templates/native_advance79.rs').read_bytes()
 contract=dict(formal_target_sha256=FORMAL,inspection=str(inspection),inspection_sha256=sha(inspection),audio_inspection=str(audio),audio_inspection_sha256=sha(audio),actual_E149_definition=definition,current_handler=body,effect3935=effect,
  correction='FrameEvent::Effect(1356) names original < audio marker, mode0 overlap; it is sound/1356.wav, not a child sprite. Previous immutable slow29 reports retain the incorrect interpretation as history.',
  attr109='original parser stores class0 in647E5C and signed secondary -1 in647E50; class0 dispatch defaults to44EDE0, identical strict attrs to existing3936/3934',
  client_spends_or_predicts_vitals=False,server_owns_cost_and_1200_second_status=True,tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
 for name,blob in after.items():
  target=owned(out/name,out);target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(blob)
  if blob!=original.get(name):(out/(Path(name).name+'.diff')).write_text(''.join(difflib.unified_diff(original.get(name,b'').decode('utf-8-sig').splitlines(True),blob.decode('utf-8').splitlines(True),fromfile='slow-candidate/'+name,tofile='advance-candidate/'+name)),encoding='utf-8')
 for name,files in [('before.zip',original),('candidate.zip',after)]:
  with zipfile.ZipFile(out/name,'x',zipfile.ZIP_DEFLATED) as z:
   for n,blob in files.items():z.writestr(n,blob)
  with zipfile.ZipFile(out/name) as z:require(all(z.read(n)==blob for n,blob in files.items()),'archive readback differs')
 record=dict(status='unselected_advance79_extension_development_only',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_slow29_candidate=str(base_path),predecessor_slow29_candidate_sha256=sha(base_path),predecessor_blind20_candidate=base['predecessor_blind20_candidate'],
  contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),candidate_files={n:hashlib.sha256(blob).hexdigest().upper() for n,blob in after.items()},
  base_files=base['base_files']|{selector:sha(R/selector)},immediate_predecessor_files={n:hashlib.sha256(blob).hexdigest().upper() for n,blob in original.items()},changed_runtime_files=base['changed_runtime_files']+[selector],new_tests_written=6,retained_blind20_tests_written=6,retained_slow29_tests_written=6,
  tests_executed=0,PEs_executed=False,production_applied=False,registered_thirteen_file_overlay_unchanged=True,whole_goal_complete=False,javap_command=command,javap_exit_code=result.returncode)
 (out/'candidate.json').write_bytes(encoded(record));print(json.dumps(dict(candidate=str(out/'candidate.json'),files=len(after),runtime_files=4,new_cases_written=6,tests_executed=0,selected=False),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('inspection',type=Path);p.add_argument('audio',type=Path);a=p.parse_args();prepare(a.inspection,a.audio)
