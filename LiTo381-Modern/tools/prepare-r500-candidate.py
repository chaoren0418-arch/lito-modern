from r491_common import *
import struct,re,shutil,difflib
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from capstone.x86 import X86_OP_MEM
e=read(L/'r500-source-study.json');pe=Path(e['analysis_PE']['path']).read_bytes();asm=Path(e['original_disassembly']['path']).read_text(encoding='utf-8');o=P/'target/r500-login-lifecycle-study'
# Pin archived observations separately from executable PE initialvalues;neverreadcurrentregistry/credentials.
lines=[]
for sym in ['dword_636584 dd','byte_75F754 db','byte_75F755 db','byte_75F756 db']:
 m=re.search('^'+sym+r'.*$',asm,re.M);assert m;lines.append(dict(line=asm[:m.start()].count('\n')+1,text=m.group(0)))
p=o/'archived-global-observations.txt';p.write_text('\n'.join(f"{q['line']}: {q['text']}"for q in lines),encoding='utf-8');e['archived_global_observations']=dict(source=pin(p),meaning='archived controlled-analysis data labels;notPEinitialorcurrentenvironmentorcredentialauthority',lines=lines)
cs=Cs(CS_ARCH_X86,CS_MODE_32);cs.detail=True;tables=[]
for jump,mapva in [(0x4B2A0C,0x4B3470),(0x4B3156,0x4B36B4)]:
 i=next(cs.disasm(pe[jump-0x400000:jump-0x400000+7],jump));assert i.mnemonic=='jmp'and i.operands[0].type==X86_OP_MEM;table=i.operands[0].mem.disp;handlers={}
 for opcode in [1,3,58,59,203]:
  index=pe[mapva-0x400000+opcode];dest=struct.unpack_from('<I',pe,table-0x400000+index*4)[0];handlers[str(opcode)]=dict(index=index,handler=hex(dest))
 tables.append(dict(jump=hex(jump),map=hex(mapva),table=hex(table),handlers=handlers))
 p=o/f'dispatch-{jump:X}.bin';p.write_bytes(pe[table-0x400000:table-0x400000+640]+pe[mapva-0x400000:mapva-0x400000+256]);e['fragments'].append(dict(dispatch_jump=hex(jump),table=hex(table),map=hex(mapva),**pin(p)))
p=o/'631E38-login-table.bin';p.write_bytes(pe[0x231E38:0x231E38+300]);e['fragments'].append(dict(table='631E38',**pin(p)));rows=[list(struct.unpack_from('<15I',p.read_bytes(),i*60))for i in range(5)]
assert rows[2][4]==0x492270;assert pe[0x236584:0x236588]==b'\xff'*4
oracle=dict(state='R500_PINNED_PE_LOGIN_INPUT_LIMIT_FIRSTBYTE_NORMALIZATION_AND_DISPATCH_ORACLE',source_study=pin(L/'r500-source-study.json'),table=pin(p),rows=rows,account_min=2,account_max=27,password_min=4,password_max=13,account_object_slots=28,password_object_slots=14,Native_object_stride=12,normalization='OnlyfirstASCIIa..z uppercased by4D3860;rest unchanged',normalized_cases=[dict(input=s,expected=(s[:1].upper()+s[1:]))for s in ['ab','tester09','aBcD',' AB','0demo','x'*27]],dispatcher=tables,synthetic_inputs=True,GUI=0)
save(L/'r500-login-input-oracle.json',oracle);e['state']='R500_SOURCE_LOGIN_CALLBACK_INPUT_FLAG_ENV_DISPATCH_LIFECYCLE_PINNED';save(L/'r500-source-study.json',e)
# Update source pin after final study additions.
oracle['source_study']=pin(L/'r500-source-study.json');save(L/'r500-login-input-oracle.json',oracle)
mapping=[
 dict(event='startup input objects',source='41A430/41A490 ->4F12E0 ctor4C3CC0,dtor4C3D20',data='account641348 28objects;password6412A0 14objects;stride12;randomized64byte backingper scalar',modern='fixedownedZeroizing28/14bytearrays;noNativepointer/obfuscation emulation;noDebug/export ofsecretfields'),
 dict(event='environment initialization',source='41A4E0 configcase4 ->636584;fn_LoadStringTable_Check(4D4860) iffFFFFFFFF ->external5A40CBF;58locale updates',data='PE636584initialFFFFFFFF;archivedASMdd3B6 isruntimeobservation;config/runtime/environment allseparate;5033C4PEinitial114F8',modern='noactualconfig/registry/localeenvironmenthardcode;480560 chdch gatedunqualified'),
 dict(event='entry background',source='496DA0/496D40/496D60',data='group20/backgroundtable631F68;75F6E5gatesESC/mouse201/204 ->496E30/externalexit5AD0986',modern='differentfromaccountsubmit;formcancelclearandreturnEntry isexplicitModernpolicy,notNativeexit emulation'),
 dict(event='actual input form',source='4966A0 ->631E38/count5/group0 ->495230/4952F0',data='row0/1 inputtype7,rows2 callback492270;4952F0 group0 clearsbothinputwidgets. 4966A0 also64BBD4=2 andprebuildscharacterstack.',modern='sameRuntime open/edit/stage/cancel;standardModernEDIT/ES_PASSWORD/EM_SETLIMITTEXT. Modernformstage doesnotsetNativephase2orinjectroster;fullNativeconstructorstacknotclaimed'),
 dict(event='login click',source='492270 ->440E50 slots0/1 ->492540(401680EDX27),48FEA0 ->4D3860',data='account2..27 bytes;password4..13;ASCIIletters/digits/space commonbranches;onlyfirstaccountbyteuppercase;MBlocale64B6DC/rangetablesneededforotherbytes',modern='closedASCIIinputonly;boundcapacities/ownerrevisionguard;unsupportedlocaleexpliciterror;localstagevalidatedinputonly'),
 dict(event='plaintext lifetime',source='492270 loops28/14objects via4C3D30;128bytepasswordstackrepstosd;slot1empty440E90',data='NativeglobalobfuscatedcredentialsretainedafterUIpasswordclear;formgroup0resetswidgets',modern='passwordEDIT clearedonstaging;privateZeroizingstagedinput retaineduntilcancel/reopen/sourceboundary/newroster/leave/Close;clearstagedglobalsattheseModernboundaries explicitpolicy'),
 dict(event='first login vs retained connection re-login',source='492270 byte75F754;492190 waitsflag thengetsockname ->492090;58 writes75F754=1',data='normal492090 extractsobjects via490390 ->cssd orcssdswith64BBF8;environmentcollect49AD70/491ED0/491FE0/492010;75F754 isversionreadinessnotauthsuccess',modern='retainedepoch/version availableonBackbutUIstagingnevercallsactualauthsender;socketaddress/env/transformauthorityremainOPEN'),
 dict(event='alternate auth flags',source='75F755:reads58/59,clear48FAEB;75F756:read/clear48FAD9,write1inline48FCDA',data='PEallencodeddirectrefs scanned;75F755nonzero producer notclosed;75F756 set1 externalexitpath;initialPE andarchivedflags0',modern='noautologin/resume flags synthesized;qualifiedexternal/indirectproducerOPEN'),
 dict(event='59 -> account/character selection list',source='dispatcher4B2970 bothswitches59 ->4B307B ->48FAA0',data='48FAA0 countc then632598 format+bounded64string,byte/wordfields;75F758 count*72;defaultcallback48F580 maycs200(selectedreceivedlistname);flagsvar1mayc53;zero noUIpromotion',modern='59 causalitymappedonly;locale834BB0 listconversion/externalcallback semantics notconnected;notauthACK'),
 dict(event='new roster',source='ordinarydispatcher3 ->4B3175/4A7450;203 ->4B3093/4A7550 ->497010/4952F0',data='3defaultsinthealternatephase4/special64AF98 dispatcher;ordinarynew3/203 rebuildrecords/showcharstack;notunconditional59=>3serverguarantee',modern='onlyactualexistingtypedstream3/203 retiresform/stagedinput andactivatesnewroster;58alonecannotpromoteUIorcreateauthsuccess')]
save(L/'r500-event-state-data-mapping.json',dict(state='R500_READABLE_EVENT_STATE_DATA_AUTHORITY_MAPPING',source=pin(L/'r500-source-study.json'),input_oracle=pin(L/'r500-login-input-oracle.json'),mapping=mapping,next_dependency='qualifyNativeSocket getsockname/localaddress,FromCD/currentenvironment/64B6DC locale,normal492090 environmenttransform/serializer contract and75F755indirectproducer;then59listconversion/cs200->server3/203 chain;noactualcredentialsneededforthisclosedUIsegment',GUI=0))
old=read(L/'r499-production-source-snapshot.json');delta=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert set(delta)=={'native_city_runtime.rs','native_city_window.rs','native_city_presentation.rs'},delta
new=P/'src/modern_lito/native_login_form.rs';diffs.append(''.join(difflib.unified_diff([],new.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(new))))
diff=L/'r500-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');files=[p for p in sorted((P/'src').rglob('*'))if p.is_file()];assert len(files)==613;out=P/'target/r500-production-source';out.mkdir(exist_ok=False);pins=[]
for p in files:
 dest=out/p.relative_to(P);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest);q=pin(p);assert pin(dest)['sha256']==q['sha256'];q['snapshot']=str(dest);pins.append(q)
save(L/'r500-production-source-snapshot.json',dict(state='R500_ALL613_PRODUCTION_SOURCE_FROZEN_BEFORE_CONSUMER',source_pins=pins,source_difference=pin(diff),GUI=0))
scope=read(L/'r499-source-scope.json');scope.pop('changed_from_fixed_R498',None);scope.pop('added_from_fixed_R498',None);scope.update(state='R500_NORMAL_LOGIN_FORM_INPUT_LIFETIME_SCOPE',source_files=613,changed_from_fixed_R499=delta,added_from_fixed_R499=[new.name],scope='3changed+1new/609previousunchanged;inputlife+localstagingonly;GUI0/authwire0/Modernowneddirectories')
scope['new_source_files'].append(dict(path='src/modern_lito/native_login_form.rs',sha256=pin(new)['sha256']))
for q in scope['new_source_files']:q['sha256']=pin(P/q['path'])['sha256']
for q in scope['frozen_binary_pins']:assert pin(q['path'])['sha256']==q['sha256']
save(L/'r500-source-scope.json',scope);save(L/'source-change-scope.json',scope)
player=read(L/'r499-player-candidate.json');root=P/'target/r500-player-entry-candidate';root.mkdir(exist_ok=False);main=root/'LiToModern-R500.rs';main.write_text(Path(player['candidate_main']['path']).read_text(encoding='utf-8').replace('r499','r500').replace('R499','R500'),encoding='utf-8');entry=read(L/'r499-entry-source-contract.json');entry.update(state='R500_NORMAL_LOGIN_FORM_LIFETIME_CANDIDATE_STATIC_ONLY',candidate_main=pin(main),previous_R499_player_record=pin(L/'r499-player-candidate.json'),UI_assets='sameR498skin;ModernEDIT account/passwordmasked/sourcecapacity/EnterandCancel;localstagingonlynocredentialswire;candidate-menuNetworkbuttonopensforminSAME Runtime;mainnoargsroute unchanged')
save(L/'r500-entry-source-contract.json',entry)
for oldname,newname in [('build-r499-player-candidate.ps1','build-r500-player-candidate.ps1'),('build-r499-back-roundtrip-consumer.ps1','build-r500-login-lifecycle-consumer.ps1')]:
 s=(P/'tools'/oldname).read_text(encoding='utf-8').replace('r499','r500').replace('R499','R500').replace('r500b-library-build','r500c-library-build')
 if 'consumer'in newname:
  s=s.replace('back-roundtrip-consumer','login-lifecycle-consumer').replace('back_roundtrip_consumer','login_lifecycle_consumer').replace("'r498-candidate.json'","'r499-candidate.json'").replace('R498 source snapshot','R499 source snapshot').replace('200','203').replace(",$source)",",$source,(Join-Path $project 'src\\modern_lito\\native_login_form.rs'))")
 (P/'tools'/newname).write_text(s,encoding='utf-8')
print('R500 frozen613sources/3changed1new/203pairs/diff',diff.stat().st_size,'bytes/sourcefragments',len(e['fragments']))
