"""Read original resurrection methods/classfile Code and exact resource definitions only."""
from pathlib import Path
from datetime import datetime,timezone
import json,re,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require
from native_E149_classfile_scan import parse,instructions
T=R/'維護工具';V=R/'驗證紀錄/current'
def methods(p):
 s=Path(p).read_text('utf-8-sig');hs=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M));return [(h.group().strip(),s[h.start():hs[i+1].start() if i+1<len(hs) else len(s)]) for i,h in enumerate(hs)]
def inspect():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development only expected');ref=w['current_E149_blizzard59_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'prior differs');q=read(ref['qualification']);out=V/'native-E149-resurrection61-inspection'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 session=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';protocol=V/'native-E149-area2253053-contract/Protocol.stdout.txt';require(sha(session)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25' and sha(protocol)=='9C68804A5854235F25CA81CEB357E63AE2EA656A48201AEC2A93EB0150F4F18D','javap source differs')
 selected={}
 for source,names in [(session,['useResurrectionSpell','useClassicWorldResurrectionSpell','answerResurrection','nativeMaterialResurrectionBody','nativeMaterialWorldResurrectionBody','nativeResurrectionAnswerBody']),(protocol,['objectSpellRequest','askAnswer','resurrectionAsk','resurrect'])]:
  for h,s in methods(source):
   for name in names:
    if ' '+name+'(' in h:require(name not in selected,'overload ambiguity');selected[name]=s;(out/(name+'.txt')).write_text(s,encoding='utf-8',newline='\n')
 require(len(selected)==10,'method set differs')
 require('bipush        36' in selected['resurrectionAsk'] and 'sipush        321' in selected['resurrectionAsk'] and 'Protocol$Writer.s:' not in selected['resurrectionAsk'],'S24 differs');require('bipush        35' in selected['askAnswer'] and all('Protocol$Reader.'+x+':' in selected['askAnswer'] for x in ['h','c','s']),'C23 differs')
 for name in ['nativeMaterialResurrectionBody','nativeMaterialWorldResurrectionBody']:require('Protocol.objectSpellRequest:' in selected[name] and 'Protocol.resurrectionAsk:' in selected[name] and 'Protocol.resurrect:' in selected[name] and 'ResurrectionSpell.effectGfx:' in selected[name],'formal cast body differs')
 answer=selected['nativeResurrectionAnswerBody'];require(answer.index('putfield      #10028')<answer.index('Protocol$AskAnswer.answer:') and 'sipush        230' in answer and 'Protocol.resurrect:' in answer,'answer consumption/result differs')
 jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal differs');code={}
 with zipfile.ZipFile(jar) as z:
  for name in ['NativeResurrectionMaterialAction','NativeResurrectionAnswerLife']:
   raw=z.read('local/lito381/server/'+name+'.class');parsed=parse(raw);rows=[dict(name=m['name'],descriptor=m['descriptor'],instructions=instructions(m['code']['bytes'],parsed['cp'])) for m in parsed['methods'] if m['code']];code[name]=dict(class_sha256=__import__('hashlib').sha256(raw).hexdigest().upper(),methods=rows)
 (out/'original-guard-Code.json').write_bytes(encoded(code))
 text=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(text)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','Text differs');raw=text.read_bytes();defs={}
 for id in [3944,230]:
  m=re.search(rb'(?m)^#'+str(id).encode()+rb'[ \t]+.*?(?=^#|\Z)',raw,re.S|re.M);require(m is not None,'definition absent');p=out/(str(id)+'.definition.txt');p.write_bytes(m.group());defs[str(id)]=dict(path=str(p),sha256=sha(p))
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p) for p in out.iterdir()};pins.update({str(p):sha(p) for p in [session,protocol,jar,T/'native_E149_classfile_scan.py']});result=dict(schema='E149-resurrection61-primary-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,source_pins=pins,external_readonly_pins={str(text):sha(text)},definitions=[d for d in q['next_definitions'] if d['skillId']==61],raw_definitions=defs,selector_object_bytes=7,ask_bytes=[36,65,1],answer_yes_bytes=[35,65,1,1,0],answer_no_bytes=[35,65,1,0,0],pending_consumed_before_answer_validation=True,success_effect_graphics=230,cast_effect_not_success=3944,ask_caster_identity_present=False,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False);p=out/'inspection.json';p.write_bytes(encoded(result));print(json.dumps(dict(inspection=str(p),sha256=sha(p),tests_executed=0),ensure_ascii=False))
if __name__=='__main__':inspect()
