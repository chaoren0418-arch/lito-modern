"""Extract owner identity evidence from original javap data, without running it."""
from pathlib import Path
from datetime import datetime,timezone
import re,json
from native_E149_recovery_contract import R,FORMAL,read,sha,encoded,owned
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
root=R/'驗證紀錄/current'
state=root/'native-E149-owner-identity/20261023-224304-427612/GameStateStore.stdout.txt'
client=root/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt'
owner=root/'native-E149-item-gameplay-reachability/20261023-224206-735514/NativeDeItemApplication$Owner.stdout.txt'
reach=root/'native-E149-item-gameplay-reachability/20261023-224206-735514/inspection.json'
inspection=read(state.with_name('inspection.json'));assert inspection['formal_target_sha256']==FORMAL and inspection['exit_code']==0
assert inspection['output_files'][str(state)]==sha(state)
reached=read(reach);assert reached['formal_target_sha256']==FORMAL and reached['tests_executed']==0
assert all(row['exit_code']==0 for row in reached['disassembly'])
assert sha(w['current_formal_reference_identity']['formal'])==FORMAL

def method(path,declaration):
 text=path.read_text('utf-8-sig');start=text.index('  '+declaration+'\n')
 rest=text[start:];end=re.search(r'\n  \S[^\n]*\([^\n]*\);?\n',rest[1:])
 return rest[:end.start()+1] if end else rest

parts={
 'GameStateStore.sourceCharacter':method(state,'static java.lang.String sourceCharacter(java.lang.String, java.lang.String);'),
 'ClientSession.sourceCharacterIdentity':method(client,'java.lang.String sourceCharacterIdentity();'),
 'ClientSession.sourceAccountIdentity':method(client,'java.lang.String sourceAccountIdentity();'),
 'ClientSession.stateCharacter':method(client,'private java.lang.String stateCharacter();'),
 'ClientSession.stateCharacterNamed':method(client,'private java.lang.String stateCharacter(java.lang.String);'),
 'ClientSession.stateAccount':method(client,'private java.lang.String stateAccount();'),
 'ClientSession.playerName':method(client,'java.lang.String playerName();'),
 'ClientSession.accountName':method(client,'java.lang.String accountName();'),
 'Owner.constructor':method(owner,'local.lito381.server.NativeDeItemApplication$Owner(local.lito381.server.NativeDeItemApplication, local.lito381.server.ClientSession);'),
}
assert parts['GameStateStore.sourceCharacter'].count('java/lang/String.trim:')==2
assert parts['GameStateStore.sourceCharacter'].count('bipush        31')==2
assert 'invalid source-qualified character identity' in parts['GameStateStore.sourceCharacter']
assert 'InvokeDynamic #47:makeConcatWithConstants:' in parts['GameStateStore.sourceCharacter']
state_text=state.read_text('utf-8-sig')
bootstrap=re.search(r'^  47:.*\n    Method arguments:\n[^\n]+',state_text,re.M).group()
assert r'\u0001\u001f\u0001' in bootstrap
assert 'sourceCharacterIdentity:()Ljava/lang/String;' in parts['Owner.constructor']
assert 'sourceAccountIdentity:()Ljava/lang/String;' in parts['Owner.constructor']
for name in ['ClientSession.stateCharacterNamed','ClientSession.stateAccount']:
 assert 'Field selectedClassicServerName:Ljava/lang/String;' in parts[name] and 'GameStateStore.sourceCharacter:' in parts[name]
assert 'Method playerName:()Ljava/lang/String;' in parts['ClientSession.stateCharacter']
assert 'Method accountName:()Ljava/lang/String;' in parts['ClientSession.stateAccount']
assert 'Field selectedCharacter:Ljava/lang/String;' in parts['ClientSession.playerName']
assert 'Field account:Ljava/lang/String;' in parts['ClientSession.accountName']
out=R/'維護工具/resources/E149-owner-identity-contract-E1498871';out.mkdir(parents=True,exist_ok=True)
assert not (out/'contract.json').exists(),'preserve original contract'
for name,text in parts.items():(out/(name+'.txt')).write_text(text,encoding='utf-8',newline='\n')
(out/'sourceCharacter.concat-bootstrap.txt').write_text(bootstrap+'\n',encoding='utf-8',newline='\n')
contract=dict(schema='E149-owner-source-identity-contract-v1',status='original_bytecode_contract_only',formal_target_sha256=FORMAL,
 source_pins={str(p):sha(p) for p in [state,state.with_name('inspection.json'),client,owner,reach]},
 excerpt_pins={str(p):sha(p) for p in out.iterdir() if p.is_file()},
 selected_character_from='ClientSession.selectedCharacter (nonempty selected case)',account_from='ClientSession.account (nonnull authenticated case)',
 owner_source_semantics='source_qualified_character_identity',owner_account_semantics='source_qualified_account_identity',
 component_normalization='java_String_trim_UTF16_code_units_0000_through_0020',separator_code_point=31,
 concatenation=['trim(source)','U+001F','trim(name)'],reject_empty_components=True,reject_embedded_separator=True,
 case_folding_at_owner_construction=False,
 source_reachability_inspection=str(reach),source_reachability_inspection_sha256=sha(reach),
 nondefault_gameplay_route_proven=False,
 findings=['The earlier reader incorrectly compared owner.source/account with bare source/account values',
  'The original owner calls sourceCharacterIdentity/sourceAccountIdentity, both using GameStateStore.sourceCharacter',
  'The sourceCharacter bytecode trims two components, rejects empty/separator-containing values and concatenates with U+001F',
  'Nondefault setter search includes inherited-owner method refs and direct fields; only Item.setItem -> setGreater is an explicit setter caller',
  'General reflection, external callers and runtime routing are not excluded by this static search',
  'StateCopy rejects active ScheduledFuture/EquipmentTimerLease and pending magic-stone expiry; these are not ordinary successful staged cases'],
 tests_executed=0,classes_instantiated=0,desktop_inputs=0,original_methods_invoked=False,
 full_native_owner_qualified=False,full_attribute_matrix_qualified=False,whole_goal_complete=False,recorded_utc=datetime.now(timezone.utc).isoformat())
(out/'contract.json').write_bytes(encoded(contract));(out/'extractor.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),method_excerpts=len(parts),tests_executed=0),ensure_ascii=False))
