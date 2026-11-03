from r491_common import *
import shutil
B=P.parent.parent/'lito脫殼資源';S=P.parent.parent/'lito伺服器'
expected={
'CD2-ready-outbound-contract.json':'F3867EC1B86DA3DF66A7493E92CD239E40AEC59B658EC0B588F463EFF13A2667',
'CD2-ready-guard-source-contract.json':'FCD54F0D67C11EC6BE3E3BDFD03C579F117A205773B67F8A29A850BB99501AB2',
'CD2-selected-record-source-contract.json':'9238357B9079A14F91132143E2FDED79B0F6F0733DAC716A6925AD7C44F19EF7',
'CD2-directory-index-storage-contract.json':'02B15800508F5608AAB3BF9B04600717E02B3D5042EF7389D31B8F00AE09929D',
'D1-E1-prestart-contract.json':'5DD6E760A08D3CD3B096FC6B2F25BF053FB51EA12E9141887A7473423EFC7AB1',
'Modern-ready-directory-adoption-contract.json':'B746EF77645DD5BE45B005C6C1F808B0DB83963D36F019CB8368548EC088A88E',
'server-room-ready-485F-version-handoff.json':'3F31E26FF006B132C4531BA53811597D3D8EF8581531AD04E8599B0C2B170947',
'server-ordinary-room-equal-sides-network.json':'F04272781DABA42A11F5C4283BC801FFF280B0B6004EB59AE75DA00473A049A0'}
root=L/'r526-inputs';root.mkdir(exist_ok=True);verified={};refs=[];roots=[]
def walk(o):
 if isinstance(o,dict):
  if isinstance(o.get('path'),str) and isinstance(o.get('sha256'),str) and len(o['sha256'])==64:
   p=Path(o['path']);p=p if p.is_absolute() else Path(handoff['manifest']['path']).parent/p;actual=verified.get(str(p))
   if actual is None:actual=pin(p);verified[str(p)]=actual
   if actual['sha256']!=o['sha256'].upper() and p.is_relative_to(P/'src'):
    prior=read(L/'r525-production-source-snapshot.json');old=next(e for e in prior['source_pins']if e['path']==str(p));actual=pin(old['snapshot']);verified[str(p)]=dict(actual,historical_live_path=str(p),basis='Batch33 read-before-R526; frozenR525 source')
   assert actual['sha256']==o['sha256'].upper(),str(p)
   if isinstance(o.get('bytes'),int):assert actual['bytes']==o['bytes'],str(p)
   refs.append(actual)
  for v in o.values():walk(v)
 elif isinstance(o,list):
  for v in o:walk(v)
for n,h in expected.items():
 p=(S if n.startswith('server-')else B)/n;actual=pin(p);assert actual['sha256']==h;verified[str(p)]=actual;walk(read(p));dst=root/n;shutil.copyfile(p,dst);roots.append(dict(original=actual,copy=pin(dst)))
handoff=read(root/'server-room-ready-485F-version-handoff.json');assert handoff['document_revision']==2 and handoff['selected_method_count']==99 and handoff['class_count']==23
for k in ['manifest','consumer_handoff','adoption_matrix','chain_comparison','class_shell','finite_IL','prestart_vectors','cancel_vectors','request_receipt_refusal_matrix','document_correction']:
 v=read(handoff[k]['path']);walk(v);shutil.copyfile(handoff[k]['path'],root/Path(handoff[k]['path']).name)
comparison=read(B/'Modern-ready-directory-adoption/implementation-comparison-table.json');walk(comparison);shutil.copyfile(B/'Modern-ready-directory-adoption/implementation-comparison-table.json',root/'implementation-comparison-table.json')
pre=read(root/'accepted-AD368-prestart-vectors.json');world=pre['examples'][0]['cutoff_first_following_packet'][0]['frame']['body_hex'];assert world=='28340200'
data=read(L/'r525-test-inputs.json');data['WORLD']=world;data['R526_scope']='ONE new synthetic normal-entry case with two sequential room epochs; savedGateNet static body observations and syntheticselfD1/E1. No new Java wire qualification.';data['adoption_roots']=roots;save(L/'r526-test-inputs.json',data)
save(L/'r526-input-adoption.json',dict(roots=roots,verified_pins=list(verified.values()),reference_occurrences=len(refs),copied_payloads=[pin(p)for p in root.iterdir()if p.is_file()],revision=2,AD368_actual='7168 WAITINGstate2 and participant repeat1 COUNTDOWNcancel; existing3/2/1 success wire',candidate485F='conditional exact necessary chain/CP/shell adoption;96 Code same/3 upstream changes; no new485F countdown/cancelwire',source_only=['explicit0cancel','COUNTDOWNstate2','WAITINGrepeat1','ownerrepeat','alternateowner/lastnonowner','observers/brokenmembership/malformed'],caveats=['nonowner opcodeonly D1=0 not hubready membership proof','commit then sequentialsendfailure no rollback','sameepoch E1 no room/generation/sequence;cannot universally identify stale','Native localflag/XOR/selectedrecord+3E no ready/owner authority','Batch33 M01-M08 retained;M09/M10 new;M13 superseded by accepted revision2'],equal_sides='actual485F1→0→1 map564→565→564 attrs40→8→40 GET-only; no attrs D52/D53 or optimisticattrs',new_Java_SQL_processes=0,external_writes=0,read_diagnostic_failures=['initial console inspection used cp950 and stoppedUnicodeEncodeError; corrected UTF8; adoption unchanged']))
print('R526 adoption verified',len(verified),'unique pins',len(refs),'references; WORLD cutoff',world)
