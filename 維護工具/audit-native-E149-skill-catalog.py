"""Decode E149 literal catalog data and index current Rust recognition, without Java execution.

The whitelist symbolically reads straight-line <clinit> data. Constructor parameter
bindings are source facts only: constructor validation, configuration overrides,
learning and phase activation are not executed or claimed.
"""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,re,struct,traceback,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
from native_E149_classfile_scan import parse,instructions
T=R/'維護工具';V=R/'驗證紀錄/current';CAT='local/lito381/server/BattleSpellCatalog'

def descriptor(text):
 require(text.startswith('('),'not a method descriptor');result=[];at=1
 while text[at]!=')':
  start=at
  while text[at]=='[':at+=1
  if text[at]=='L':at=text.index(';',at)+1
  else:require(text[at] in 'BCDFIJSZ','unsupported descriptor');at+=1
  result.append(text[start:at])
 return result,text[at+1:]

def load_slot(row):
 op=row['opcode'];raw=bytes.fromhex(row['hex'])
 if 0x15<=op<=0x19:return raw[1]
 for base in [0x1a,0x1e,0x22,0x26,0x2a]:
  if base<=op<base+4:return op-base
 if op==0xc4 and 0x15<=raw[1]<=0x19:return int.from_bytes(raw[2:4],'big')
 return None

def store_slot(row):
 op=row['opcode'];raw=bytes.fromhex(row['hex'])
 if 0x36<=op<=0x3a:return raw[1]
 for base in [0x3b,0x3f,0x43,0x47,0x4b]:
  if base<=op<base+4:return op-base
 if op==0x84:return raw[1]
 if op==0xc4 and (0x36<=raw[1]<=0x3a or raw[1]==0x84):return int.from_bytes(raw[2:4],'big')
 return None

def parameter_fields(c,desc,trail=()):
 require(desc not in trail and len(trail)<4,'constructor delegation cycle/bound')
 params,result=descriptor(desc);require(result=='V','constructor result differs');matches=[m for m in c['methods'] if m['name']=='<init>' and m['descriptor']==desc];require(len(matches)==1 and matches[0]['code'] is not None,'constructor absent');method=matches[0];rows=instructions(method['code']['bytes'],c['cp']);slots={};slot=1
 for index,param in enumerate(params):slots[slot]=(index,param);slot+=2 if param in ['J','D'] else 1
 require(not any(store_slot(row) is not None and store_slot(row)<slot for row in rows),'constructor modifies argument local')
 writes=[row for row in rows if row['opcode']==0xb5]
 if not writes:
  require(len(rows)>=3 and rows[0]['opcode']==0x2a and rows[-2]['opcode']==0xb7 and rows[-1]['opcode']==0xb1,'unrecognized constructor delegation')
  target=rows[-2]['member'];require(target['owner']==c['class_name'] and target['name']=='<init>','constructor does not delegate to same class');arguments=[]
  for row in rows[1:-2]:
   local=load_slot(row)
   if local in slots:arguments.append(dict(kind='argument',index=slots[local][0]))
   elif 'constant' in row:arguments.append(dict(kind='literal',value=row['constant']))
   else:raise RuntimeError('nonliteral constructor delegation argument')
  target_params,_=descriptor(target['descriptor']);require(len(arguments)==len(target_params),'delegating constructor arity differs')
  target_mapping,proof=parameter_fields(c,target['descriptor'],(*trail,desc))
  mapped={field:(arguments[expression['index']] if expression['kind']=='argument' else expression) for field,expression in target_mapping.items()}
  return mapped,dict(descriptor=desc,code_sha256=hashlib.sha256(method['code']['bytes']).hexdigest().upper(),delegates_to=target['descriptor'],argument_expressions=arguments,delegate_binding=proof,argument_locals_not_modified=True,constructor_validation_executed=False)
 mapped={};assignments=[]
 for i,row in enumerate(rows):
  if row['opcode']!=0xb5:continue
  ref=row['member'];require(ref['owner']==c['class_name'] and i>=2 and rows[i-2]['opcode']==0x2a,'noncanonical field assignment');source=load_slot(rows[i-1]);require(source in slots,'field not directly bound to argument');index,param=slots[source]
  require(ref['descriptor']==param and ref['name'] not in mapped and index not in mapped.values(),'argument/field binding differs');mapped[ref['name']]=index;assignments.append(dict(offset=row['offset'],field=ref['name'],argument=index,local_slot=source,descriptor=param))
 fields={f['name'] for f in c['fields'] if not f['access']&8}
 require(set(mapped)==fields and set(mapped.values())==set(range(len(params))),'incomplete constructor field binding')
 return {field:dict(kind='argument',index=index) for field,index in mapped.items()},dict(descriptor=desc,code_sha256=hashlib.sha256(method['code']['bytes']).hexdigest().upper(),assignments=assignments,argument_locals_not_modified=True,constructor_validation_executed=False)

def source_ids(path):
    """Recognize explicit request arms and the shared selector/item classifiers."""
    text = path.read_text(encoding='utf-8-sig')
    scope = text[text.index('pub fn request('):text.index('pub fn reuse_millis(')]
    ids = set()
    for match in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>', scope, re.M):
        ids.update(map(int, re.findall(r'\d+', match[1])))

    def helper_ids(name):
        pattern = (r'pub fn ' + name + r'\s*\(\s*skill\s*:\s*u16\s*\)\s*->\s*bool\s*'
                   r'\{\s*matches!\(\s*skill\s*,\s*([\d\s|]+)\)\s*\}')
        match = re.search(pattern, text)
        require(match is not None or 'pub fn ' + name not in text,
                name + ' classification syntax differs')
        return set(map(int, re.findall(r'\d+', match[1]))) if match else set()

    selectors = helper_ids('selector_only')
    require(not selectors or re.search(r'\bid\s+if\s+selector_only\(id\)\s*=>', scope),
            'selector helper is not wired into request')
    ids.update(selectors)
    ground_scope = text[text.index('pub fn ground_request('):text.index('pub fn in_native_screen(')]
    ground = re.search(r'ensure!\(\s*matches!\(\s*skill\s*,\s*([\d\s|]+)\)', ground_scope)
    require(ground is not None, 'ground arm syntax differs')
    ground_ids = set(map(int, re.findall(r'\d+', ground[1])))
    item_ids = helper_ids('item_targeted')
    return dict(actor_or_selector_request_ids=sorted(ids), ground_request_ids=sorted(ground_ids),
                inventory_request_ids=sorted(item_ids), all_request_ids=sorted(ids | ground_ids | item_ids),
                path=str(path), sha256=sha(path))


def main():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'workload/freeze active');require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only phase required')
 ref=w['current_E149_object374448_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'latest source qualification changed');q=read(ref['qualification']);check_pins(q['source_pins']);cp=Path(q['candidate']);require(sha(cp)==q['candidate_sha256'],'source candidate changed');candidate=read(cp)
 jar=owned(w['current_formal_reference_identity']['cached_reference']);require(sha(jar)==FORMAL==sha(w['current_formal_reference_identity']['formal']),'formal target changed')
 out=owned(V/'native-E149-skill-catalog-audit'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True);(out/'auditor.py').write_bytes(Path(__file__).read_bytes());(out/'classfile-reader.py').write_bytes((T/'native_E149_classfile_scan.py').read_bytes())
 try:
  with zipfile.ZipFile(jar) as z:
   require(len({x.filename for x in z.infolist()})==len(z.infolist()),'duplicate JAR entry');cache={};class_pins={}
   def klass(name):
    if name not in cache:
     raw=z.read(name+'.class');c=parse(raw);require(c['class_name']==name,'class name differs');cache[name]=c;class_pins[name]=hashlib.sha256(raw).hexdigest().upper()
    return cache[name]
   catalog=klass(CAT);clinit=next(m for m in catalog['methods'] if m['name']=='<clinit>');require(not clinit['code']['exceptions'],'exceptional initializer unsupported');code=clinit['code']['bytes'];rows=instructions(code,catalog['cp']);stack=[];statics={};allocations=[];opaque=[]
   def popn(n):
    require(0<=n<=len(stack),'symbolic stack underflow');items=stack[-n:] if n else []
    if n:del stack[-n:]
    return items
   for row in rows:
    op=row['opcode'];pool=catalog['cp'];at=row['offset']
    if 'constant' in row:stack.append(dict(kind='literal',value=row['constant']));continue
    if 'constant_string' in row:
     require('\ufffd' not in row['constant_string'],'modified UTF8 literal requires review');stack.append(dict(kind='literal',value=row['constant_string']));continue
    if op in [0x0b,0x0c,0x0d,0x0e,0x0f]:stack.append(dict(kind='literal',value=float(op-(0x0b if op<=0x0d else 0x0e))));continue
    if op in [0x12,0x13,0x14] and pool[row['cp_index']][0] in [4,6]:
     v=pool[row['cp_index']];stack.append(dict(kind='literal',value=struct.unpack('>f' if v[0]==4 else '>d',bytes.fromhex(v[1]))[0]));continue
    if op==0xbb:
     name=pool[pool[row['cp_index']][1]][1];require(name.startswith(CAT+'$'),'initializer creates external object');value=dict(kind='object',class_name=name,new_offset=at,arguments=None,constructor=None);stack.append(value);allocations.append(value);continue
    if op==0x59:require(stack,'dup on empty stack');stack.append(stack[-1]);continue
    if op==0xb2:
     field=row['member'];key=field['owner']+'.'+field['name']+':'+field['descriptor'];stack.append(statics[key]['value'] if key in statics else dict(kind='static_reference',member=field));continue
    if op==0xb7:
     member=row['member'];params,result=descriptor(member['descriptor']);args=popn(len(params));receiver=popn(1)[0]
     require(member['name']=='<init>' and receiver['kind']=='object' and receiver['class_name']==member['owner'] and receiver['constructor'] is None and result=='V','initializer constructor pattern differs');receiver.update(arguments=args,constructor=member,constructor_offset=at);continue
    if op==0xbd:
     count=popn(1)[0];name=pool[pool[row['cp_index']][1]][1];require(count['kind']=='literal' and type(count['value']) is int and 0<=count['value']<=1024 and name=='java/util/Map$Entry','unsupported initializer array');stack.append(dict(kind='array',component=name,values=[None]*count['value']));continue
    if op==0x53:
     array,index,value=popn(3);require(array['kind']=='array' and index['kind']=='literal' and type(index['value']) is int and 0<=index['value']<len(array['values']) and array['values'][index['value']] is None,'invalid/overwritten symbolic array cell');array['values'][index['value']]=value;continue
    if op==0xb8:
     member=row['member'];params,result=descriptor(member['descriptor']);args=popn(len(params));owner,name=member['owner'],member['name']
     if owner in ['java/lang/Integer','java/lang/Long'] and name=='valueOf':require(len(args)==1 and args[0]['kind']=='literal' and type(args[0]['value']) is int,'boxing noninteger');value=args[0]
     elif owner=='java/util/Map' and name=='entry':require(len(args)==2,'entry arity');value=dict(kind='entry',key=args[0],value=args[1])
     elif owner=='java/util/Map' and name in ['of','ofEntries']:
      if name=='of':require(len(args)%2==0,'map arity');entries=[dict(kind='entry',key=args[i],value=args[i+1]) for i in range(0,len(args),2)]
      else:require(len(args)==1 and args[0]['kind']=='array' and all(v is not None and v['kind'] in ['entry','opaque_entry'] for v in args[0]['values']),'map entry array incomplete');entries=args[0]['values']
      keys=[json.dumps(e['key'],sort_keys=True) for e in entries if e['kind']=='entry'];require(len(keys)==len(set(keys)),'duplicate map key');value=dict(kind='map',entries=entries)
     elif owner==CAT and (name,member['descriptor']) in [('summon','(IIIIZ)Ljava/util/Map$Entry;'),('summonSpecial','(IIII)Ljava/util/Map$Entry;')]:
      value=dict(kind='opaque_entry',member=member,arguments=args,offset=at);opaque.append(value)
     else:raise RuntimeError('unreviewed initializer invocation '+str(member))
     stack.append(value);continue
    if op==0xb3:
     field=row['member'];key=field['owner']+'.'+field['name']+':'+field['descriptor'];require(field['owner']==CAT and key not in statics,'reassigned/outside static field');statics[key]=dict(field=field,offset=at,value=popn(1)[0]);continue
    if op==0xb1:require(not stack and at==rows[-1]['offset'],'initializer return stack differs');continue
    raise RuntimeError('unreviewed initializer opcode '+hex(op)+' at '+str(at))
   require(not stack and all(x['constructor'] is not None for x in allocations),'incomplete symbolic initializer')
   binding_cache={};definitions=[];unmapped=[]
   def bind(value):
    if value['kind']!='object':return value
    name=value['class_name'];desc=value['constructor']['descriptor'];key=name+desc
    if key not in binding_cache:binding_cache[key]=parameter_fields(klass(name),desc)
    mapping,proof=binding_cache[key];fields={field:(bind(value['arguments'][expression['index']]) if expression['kind']=='argument' else expression) for field,expression in mapping.items()}
    return dict(kind='record',class_name=name,fields=fields,new_offset=value['new_offset'],constructor_offset=value['constructor_offset'],binding=key)
   def literal(value):
    if value['kind']=='literal':return value['value']
    if value['kind']=='record':return {n:literal(v) for n,v in value['fields'].items()}
    return value
   for key,static in statics.items():
    value=static['value']
    if value['kind']!='map':continue
    for entry in value['entries']:
     if entry['kind']!='entry':unmapped.append(dict(static_field=static['field'],entry=entry,reason='opaque helper entry, not interpreted'));continue
     obj=entry['value']
     if obj['kind']!='object' or not obj['class_name'].endswith('Spell'):continue
     try:record=bind(obj)
     except RuntimeError as e:unmapped.append(dict(static_field=static['field'],new_offset=obj['new_offset'],class_name=obj['class_name'],reason=str(e)));continue
     fields=literal(record);core=fields
     if 'skillId' not in core:
      nested=[v for v in fields.values() if isinstance(v,dict) and 'skillId' in v];require(len(nested)==1,'wrapper has no unambiguous skill record');core=nested[0]
     require(entry['key']['kind']=='literal' and entry['key']['value']==core['skillId'] and 1<=core['skillId']<=220 and core['skillId']==core['row']*8+core['column']+1,'literal map key/selector identity differs')
     definitions.append(dict(skill_id=core['skillId'],record_type=obj['class_name'].split('$')[-1],map_field=static['field']['name'],putstatic_offset=static['offset'],definition=fields,core=core,record=record))
   factories=[]
   for method in catalog['methods']:
    if not method['code'] or not re.fullmatch(r'\(I\)L'+CAT+r'\$\w+Spell;',method['descriptor']):continue
    body=instructions(method['code']['bytes'],catalog['cp']);gets=[x for x in body if x['opcode']==0xb2 and x['member']['owner']==CAT and x['member']['descriptor']=='Ljava/util/Map;'];calls=[x for x in body if x['opcode'] in [0xb6,0xb8,0xb9]]
    require(gets and any(x['member']['owner']=='java/util/Map' and x['member']['name']=='get' for x in calls),'factory map lookup absent')
    factories.append(dict(name=method['name'],descriptor=method['descriptor'],referenced_map_fields=list(dict.fromkeys(x['member']['name'] for x in gets)),single_map_lookup=len(gets)==1,code_sha256=hashlib.sha256(method['code']['bytes']).hexdigest().upper(),instructions=body,branch_activation_verified=False))
   for definition in definitions:definition['factories']=[dict(name=f['name'],descriptor=f['descriptor']) for f in factories if definition['map_field'] in f['referenced_map_fields']]
   # This index deliberately stops at method-level links. A shared dispatch
   # method is not evidence that a particular skill selects every listed helper.
   client=klass('local/lito381/server/ClientSession');factory_by_name={(f['name'],f['descriptor']):f for f in factories};factory_calls=[]
   for method in client['methods']:
    if not method['code']:continue
    body=instructions(method['code']['bytes'],client['cp']);calls=[x for x in body if x['opcode'] in [0xb6,0xb7,0xb8,0xb9]]
    for call in calls:
     member=call['member']
     if member['owner']!=CAT or (member['name'],member['descriptor']) not in factory_by_name:continue
     factory=factory_by_name[(member['name'],member['descriptor'])];record_descriptor=member['descriptor'][3:];helpers=[x for x in calls if record_descriptor in x['member']['descriptor'] and x['member']['owner']!=CAT]
     factory_calls.append(dict(factory=member,offset=call['offset'],client_method=method['name'],client_descriptor=method['descriptor'],code_sha256=hashlib.sha256(method['code']['bytes']).hexdigest().upper(),same_method_typed_helper_calls=helpers,record_argument_dataflow_verified=False,per_skill_branch_verified=False))
   normal=source_ids(P/'src/modern_lito/native_skills.rs');latest=source_ids(Path(candidate['effective_overlay_paths']['LiTo381-Modern/src/modern_lito/native_skills.rs']));catalog_ids=sorted({d['skill_id'] for d in definitions});missing=sorted(set(catalog_ids)-set(latest['all_request_ids']))
   byid=[dict(skill_id=i,definitions=[d for d in definitions if d['skill_id']==i],normal_request_recognized=i in normal['all_request_ids'],candidate_request_recognized=i in latest['all_request_ids'],current_learned_book_observed=False,gameplay_verified=False) for i in catalog_ids]
   facts=dict(catalog_class_sha256=class_pins[CAT],initializer_code_sha256=hashlib.sha256(code).hexdigest().upper(),initializer_instructions=len(rows),allocated_objects=len(allocations),initialized_static_fields=len(statics),opaque_helper_calls=opaque,static_maps=statics,constructor_bindings={key:proof for key,(_,proof) in binding_cache.items()},factories=factories,factory_calls=factory_calls,definitions_by_id=byid,unmapped_entries=unmapped,class_pins=class_pins)
   (out/'catalog-facts.json').write_bytes(encoded(facts))
  source_pins={str(p):sha(p) for p in [Path(__file__).resolve(),T/'native_E149_classfile_scan.py',cp,Path(normal['path']),Path(latest['path']),Path(ref['qualification'])]}
  check_pins(source_pins);require(sha(jar)==FORMAL==sha(w['current_formal_reference_identity']['formal']),'formal changed while reading')
  record=dict(status='E149_catalog_literal_maps_and_source_recognition_static_only',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,jar=str(jar),source_qualification=ref,source_pins=source_pins,facts=str(out/'catalog-facts.json'),facts_sha256=sha(out/'catalog-facts.json'),catalog_literal_definitions=len(definitions),catalog_literal_unique_ids=len(catalog_ids),catalog_literal_ids=catalog_ids,normal_request=normal,candidate_request=latest,candidate_missing_catalog_ids=missing,candidate_missing_catalog_count=len(missing),normal_missing_catalog_ids=sorted(set(catalog_ids)-set(normal['all_request_ids'])),catalog_skill_maps_with_unmapped_objects=[x for x in unmapped if x.get('class_name','').endswith('Spell')],opaque_non_skill_entries=len(opaque),factory_count=len(factories),ClientSession_factory_call_sites=len(factory_calls),classfiles_read=len(class_pins),tests_executed=0,Java_classes_loaded=False,Java_methods_invoked=False,learned_book_currently_observed=False,per_skill_dispatch_or_packet_contract_verified=False,costs_or_configuration_overrides_executed=False,normal_sources_or_PEs_modified=False,production_applied=False,GUI_operated=False,whole_goal_complete=False,limitations=['Literal constructor/map definitions are not actual learned books or active phases','Parameter-to-field bindings do not execute constructor validation or ResourceDatabase overrides','ClientSession same-method typed helpers are an overapproximation, not record argument or per-skill dispatch dataflow','Source request arms denote recognition only, not correct payload, assets, audio, server acceptance or GUI','Opaque summon helpers are retained and excluded from spell definition completeness claims'],recorded_utc=datetime.now(timezone.utc).isoformat())
  (out/'inspection.json').write_bytes(encoded(record));print(json.dumps({k:record[k] for k in ['catalog_literal_definitions','catalog_literal_unique_ids','candidate_missing_catalog_count','factory_count','ClientSession_factory_call_sites','classfiles_read','tests_executed']},ensure_ascii=False));print(json.dumps(dict(inspection=str(out/'inspection.json')),ensure_ascii=False))
 except BaseException:
  (out/'original-failure.txt').write_text(traceback.format_exc(),encoding='utf-8');raise

if __name__=='__main__':main()
