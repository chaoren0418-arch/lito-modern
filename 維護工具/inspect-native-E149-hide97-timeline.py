"""Readonly timeline/material bytecode and native zero-callback scheduling."""
from pathlib import Path
from datetime import datetime,timezone
import re,json,zipfile,struct,capstone
from native_E149_recovery_contract import R,P,FORMAL,read,sha,encoded,require,check_pins
from native_E149_classfile_scan import parse,instructions
from native_E149_classfile_bootstraps import bootstraps
T=R/'維護工具';V=R/'驗證紀錄/current'
def main():
 canonical=V/'native-E149-hide97-canonical/20261024-190541-246403/canonical.json';require(sha(canonical)=='270394BBAB237F539EC4CC09FC28D2C6C89EDD31BD2F3F21621D35DD89F1FCD7','canonical changed');j=read(canonical);check_pins(j['source_pins']);require(all(sha(p)==h for p,h in j['external_readonly_pins'].items()),'canonical index changed');jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL,'formal changed');out=V/'native-E149-hide97-timeline'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 with zipfile.ZipFile(jar)as z:
  classes=[n for n in z.namelist()if n.endswith('.class')and re.match(r'local/lito381/server/NativeCastTimeline(?:[$.]|$)',n)]
  for n in classes:
   raw=z.read(n);c=parse(raw);rows=[dict(name=m['name'],descriptor=m['descriptor'],instructions=instructions(m['code']['bytes'],c['cp']))for m in c['methods']if m['code']];(out/(Path(n).stem+'-Code.json')).write_bytes(encoded(rows));_,bindings=bootstraps(raw)if any(r and r[0]==18 for r in c['cp'])else(c,{});(out/(Path(n).stem+'-bootstraps.json')).write_bytes(encoded(bindings))
 pe=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe');asm=pe.parents[1]/'ida-client-oep-v2/full-disassembly.asm';require(sha(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9' and sha(asm)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A','native changed');b=pe.read_bytes();e=struct.unpack_from('<I',b,60)[0];section=e+24+struct.unpack_from('<H',b,e+20)[0];count=struct.unpack_from('<H',b,e+6)[0]
 def raw(va,n):
  for i in range(count):
   vs,rv,sz,off=struct.unpack_from('<IIII',b,section+40*i+8)
   if rv<=va-0x400000 and va-0x400000+n<=rv+min(vs,sz):return b[off+va-0x400000-rv:off+va-0x400000-rv+n]
  raise ValueError(hex(va))
 md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);text=asm.read_text('utf-8-sig');functions={}
 for start,end in [(0x44ef80,0x44f320),(0x44f330,0x44f340),(0x4d6570,0x4d65e8),(0x4d65f0,0x4d66e8),(0x4d69f0,0x4d6a3f),(0x4e01a0,0x4e01f0),(0x4e8230,0x4e82f0)]:
  p=out/f'{start:X}.bin';p.write_bytes(raw(start,end-start));rows=[dict(address=i.address,hex=i.bytes.hex(),mnemonic=i.mnemonic,operands=i.op_str)for i in md.disasm(p.read_bytes(),start)];(out/f'{start:X}.json').write_bytes(encoded(rows));name=f'{start:X}';a=text.index('sub_'+name+' proc near');z=text.index('sub_'+name+' endp',a)+len('sub_'+name+' endp');(out/(name+'.asm.txt')).write_text(text[a:z]+'\n',encoding='utf-8',newline='\n');functions[name]=dict(start=start,end=end,bytes=end-start,sha256=sha(p))
 a=text.index('fn_ClientApplication_Run proc near');z=text.index('fn_ClientApplication_Run endp',a)+len('fn_ClientApplication_Run endp');main=text[a:z];require('call    sub_4D6570\n\ncall    off_62C9C0'in main,'main order differs');(out/'ClientApplication_Run.asm.txt').write_text(main+'\n',encoding='utf-8',newline='\n')
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p)for p in out.iterdir()};pins.update({str(p):sha(p)for p in [canonical,jar,T/'native_E149_classfile_scan.py',T/'native_E149_classfile_bootstraps.py']});p=out/'timeline.json';p.write_bytes(encoded(dict(schema='E149-hide97-native-zero-callback-and-material-source-v1',formal_target_sha256=FORMAL,source_pins=pins,external_readonly_pins={str(pe):sha(pe),str(asm):sha(asm)},canonical=str(canonical),canonical_sha256=sha(canonical),functions=functions,zero_delay_is_enqueued_not_synchronous=True,drain_snapshots_clock_and_checks_due=True,main_drain_precedes_current_active_function=True,world_active_function_network_vs_render_order_qualified=False,clock_import_and_initialization_qualified=False,hide97_first_frame_visibility_and_retirement_qualified=False,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(timeline=str(p),sha256=sha(p),classes=len(classes),native_functions=len(functions),tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
