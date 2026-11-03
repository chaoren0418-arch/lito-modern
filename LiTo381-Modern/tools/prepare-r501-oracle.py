from r491_common import *
from r495_source_oracle import source_cipher,counter_add_adc
from r496_source_oracle import version_body
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
import re,struct,hashlib
study=read(L/'r501-source-study.json');pe=Path(study['analysis_PE']['path']).read_bytes();asm=Path(study['original_disassembly']['path']).read_text(encoding='utf-8');root=P/'target/r501-normal-auth-send-study'
cs=Cs(CS_ARCH_X86,CS_MODE_32);cs.detail=True
def dis(lo,hi):return list(cs.disasm(pe[lo-0x400000:hi-0x400000],lo))
def cstr(va):return pe[va-0x400000:].split(b'\0',1)[0]
# Retain command-line producers, independently from current argv/flag authority.
lines=asm.splitlines();hits=[i for i,s in enumerate(lines)if re.search(r'mov\s+byte_64BBF8, 1|mov\s+edi, offset dword_64B97C',s)]
assert len(hits)==2,hits
for idx in hits:
 p=root/f'startup-producer-line{idx+1}.asm';p.write_text('\n'.join(f'{j+1}: {lines[j]}'for j in range(max(0,idx-35),idx+35))+'\n',encoding='utf-8');study['fragments'].append(dict(line_start=idx-34,**pin(p)))
for lo,hi in [(0x481590,0x481760),(0x632620,0x63262E),(0x4C6A74,0x4C6AB1)]:
 p=root/f'{lo:X}-{hi:X}-additional.bin';p.write_bytes(pe[lo-0x400000:hi-0x400000]);study['fragments'].append(pin(p))
salt=b'43@#$asdfsag%$#%gfgfd9038YYI%$()_';at=pe.find(salt+b'\0');assert at>=0;p=root/'492010-fixed-salt.bin';p.write_bytes(pe[at:at+len(salt)+1]);study['fragments'].append(dict(address=hex(at+0x400000),**pin(p)))
study['state']='R501_NORMAL_INPUT_FORMATTER_ENVIRONMENT_LOCALADDR_AND_COMPATIBILITY_SOURCE_PINNED';save(L/'r501-source-study.json',study)
assert cstr(0x632620)==b'cssd' and cstr(0x632628)==b'cssds'
code={i.address:i for i in dis(0x492090,0x492181)}
assert code[0x492130].op_str=='0x492010'
after=[i for i in code.values()if i.address>=0x492135]
assert all('[ebp - 0x10]'not in i.op_str for i in after)
def login_format(special,account,password,ip,supplemental=b''):
 # Evaluate ONLY post-digest branch and vararg construction, with synthetic locals.
 regs={'esi':int.from_bytes(ip,'little'),'eax':0,'ecx':0,'edx':0};stack=[];pc=0x49213E if special else 0x492162
 strings={-0x3C:account,-0x20:password,0x64B97C:supplemental}
 while True:
  i=code[pc]
  if i.mnemonic=='lea':regs[i.op_str.split(',')[0]]=strings[i.operands[1].mem.disp]
  elif i.mnemonic=='push':
   o=i.operands[0];stack.append(regs[i.reg_name(o.reg)]if o.type==1 else strings.get(o.imm,o.imm))
  elif i.mnemonic=='call':assert i.op_str=='0x4c6ae0';break
  else:raise AssertionError(i.op_str)
  pc+=i.size
 args=list(reversed(stack));fmt=cstr(args[0]);return formatter(fmt,args[1:]),dict(call=hex(pc),format=fmt.decode(),order=['opcode','account','password','socket_local_sin_addr']+(['supplemental64B97C']if special else[]))
handlers={}
for ch in 'chds':
 index=pe[0xC6A90+ord(ch)-0x53];dest=struct.unpack_from('<I',pe,0xC6A74+index*4)[0];handlers[ch]=dest
assert handlers=={'c':0x4C6916,'h':0x4C6930,'d':0x4C6951,'s':0x4C6989}
texts={ch:'\n'.join(i.mnemonic+' '+i.op_str for i in dis(dest,{'c':0x4C6930,'h':0x4C6951,'d':0x4C6989,'s':0x4C69D6}[ch]))for ch,dest in handlers.items()}
assert 'mov byte ptr [ebx], cl' in texts['c'];assert 'sar ecx, 8' in texts['h'];assert 'sar ecx, 0x18' in texts['d'];assert 'repne scasb' in texts['s'] and 'lea ebx, [ebx + eax + 1]'in texts['s']
def formatter(fmt,args):
 assert len(fmt)==len(args);out=bytearray()
 for ch,v in zip(fmt.decode(),args):
  if ch=='s':out.extend(bytes(v).split(b'\0')[0]+b'\0')
  else:
   n={'c':1,'h':2,'d':4}[ch];out.extend((v&((1<<(n*8))-1)).to_bytes(n,'little'))
 assert len(out)<=0x13FE;return list(out)
seedbody=[1]+list((0xFFFFFFFE).to_bytes(4,'little'));seed=(0x87546CA1<<32)|0xFFFFFFFE
records=read(L/'r498-source-oracle.json')['setup_bodies'][2:];assert len(records)==4
def wire(body,counter):
 cipher,steps=source_cipher(bytes(body),counter,pe);return list((len(body)+2).to_bytes(2,'little')+cipher)
profiles=[]
for special in [False,True]:
 env=dict(label='R501_SYNTHETIC_SPECIAL'if special else'R501_SYNTHETIC_NORMAL',code_page=950,from_cd_registry_dword=0x123456A5,revision_word_source=0x112345,special=special,supplemental=list(b'FIXTURE-EXTRA')if special else[])
 compat=formatter(b'chdch',[0,0x8033,env['code_page'],2 if special else env['from_cd_registry_dword'],env['revision_word_source']]);tx=seed;sends=[]
 for name,body in [('compatibility',compat),('first_login',login_format(special,b'FixtureA',b'dummyPass1',bytes([127,0,0,1]),bytes(env['supplemental']))[0]),('character_back',[15]),('retained_relogin',login_format(special,b'FixtureB',b'dummyPass2',bytes([127,0,0,1]),bytes(env['supplemental']))[0]),('second_back',[15]),('failing_login',login_format(special,b'FixtureC',b'dummyPass3',bytes([127,0,0,1]),bytes(env['supplemental']))[0])]:
  after=counter_add_adc(tx,len(body));sends.append(dict(name=name,plain=body,wire=wire(body,tx),before=tx,after=after));tx=after
 rx=seed;receives=[]
 # Nine actual version frames exercise bounded-batch tail, not authentication success.
 for name,bodies in [('versions',[version_body(0x9E38)]*9),('first_roster',[[3,4]]+records),('second_roster',[[3,4]]+[[203]+list(f'Again{i}\0'.encode())+b[7:]for i,b in enumerate(records)])]:
  raw=[];before=rx
  for body in bodies:raw+=wire(body,rx);rx=counter_add_adc(rx,len(body))
  receives.append(dict(name=name,wire=raw,bodies=bodies,before=before,after=rx))
 profiles.append(dict(environment=env,send_vectors=sends,receive_vectors=receives))
envcases=[]
for serial,adapters in [(b'   SERIAL-123456789XYZ',[(6,bytes.fromhex('0011AAEEFF22')),(1,bytes(6)),(6,bytes.fromhex('ABCDEF010203'))]),(b'    ',[]),(b'A\0B',[(1,bytes(6))])]:
 normalized=serial.lstrip(b' ').split(b'\0')[0][:16].ljust(16,b'\0');adapter=bytes(16)
 for kind,mac in adapters:
  if kind==6:adapter=mac.hex().upper().encode().ljust(16,b'\0')
 mixed=bytes((a+b)&255 for a,b in zip(normalized,adapter));envcases.append(dict(serial=list(serial),adapters=[dict(kind=k,bytes=list(m))for k,m in adapters],normalized=list(normalized),adapter=list(adapter),mixed=list(mixed+b'0'),digest_reference_only=list(hashlib.md5(mixed+salt).digest())))
vectors=[]
for special in [False,True]:
 for a,pw,ip,extra in [(b'AB',b'1234',bytes([1,2,3,4]),b''),(b'A'*27,b'P'*13,bytes([127,0,0,1]),b'Z'*107)]:
  body,meta=login_format(special,a,pw,ip,extra);vectors.append(dict(special=special,account=list(a),password=list(pw),ip=list(ip),extra=list(extra),plain=body,meta=meta))
save(L/'r501-source-oracle.json',dict(state='R501_SOURCE_BRANCH_VARARGS_FORMATTER_AND_PINNED_PE_CIPHER_SYNTHETIC_ORACLE',source_study=pin(L/'r501-source-study.json'),source_cipher_tool=pin(P/'tools/r495_source_oracle.py'),schema_reference=pin(L/'r498-source-oracle.json'),version_schema_tool=pin(P/'tools/r496_source_oracle.py'),seed=seed,seed_wire=list((7).to_bytes(2,'little'))+seedbody,profiles=profiles,serializer_vectors=vectors,environment_cases=envcases,formatter_handlers={k:hex(v)for k,v in handlers.items()},environment_digest_not_in_login_varargs=True,normal_body_length_range=[13,47],Native_formatter_capacity=0x13FE,Modern_candidate_capacity=155,synthetic_credentials_only=True,PE_execution=False,GUI=0))
print('R501 sourceoracle generated:',len(study['fragments']),'source fragments;2profiles/12newTXvectors/4formatter/3environmentcases')
