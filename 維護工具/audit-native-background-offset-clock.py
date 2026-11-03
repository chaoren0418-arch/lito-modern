"""Read-only evidence for the distinct64ms background-offset producer and missing binding."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,struct
ROOT=Path(__file__).resolve().parents[1]
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('--output',required=True,type=Path);a=ap.parse_args()
output=a.output.resolve();assert output.is_relative_to(ROOT/'驗證紀錄/current') and output.parent.is_dir() and not output.exists()
PE=ROOT.parent/'lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe'
ASM=ROOT.parent/'lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm'
assert sha(PE)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
assert sha(ASM)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
data=PE.read_bytes();u16=lambda n:struct.unpack_from('<H',data,n)[0];u32=lambda n:struct.unpack_from('<I',data,n)[0]
header=u32(60);optional=header+24;base=u32(optional+28);sections=[]
for i in range(u16(header+6)):
    at=optional+u16(header+20)+40*i;sections.append((u32(at+12),u32(at+16),u32(at+20)))
def get(va,size):
    for start,length,offset in sections:
        if start<=va-base and va-base+size<=start+length:return data[offset+va-base-start:offset+va-base-start+size]
    raise AssertionError(hex(va))
initial=list(struct.unpack('<II',get(0x62f4c0,8)));assert initial==[int('FFCC',16),int('FFFE',16)]
keep=list(struct.unpack('<9I',get(0x6365e0,36)));assert keep[6]==0x4878c0
ranges={'callback':(287804,287823),'renderer_arguments':(289061,289075),
    'direct_cancel_site':(269386,269420),'map_handler_site':(354145,354166)}
spans={k:[] for k in ranges};writes=[];function=None
with ASM.open(encoding='utf-8-sig',newline='\n') as f:
    for n,line in enumerate(f,1):
        text=line.rstrip('\r\n');match=re.match(r'(\S+) proc\b',text)
        if match:function=match[1]
        for key,(first,last) in ranges.items():
            if first<=n<=last:spans[key].append(dict(newline_line=n,text=text))
        if re.match(r'\s*(?:mov|add|inc|sub|dec|xor|and|or)\s+(?:dword_62F4C0|dword_62F4C4)(?:,|\s*$)',text):
            writes.append(dict(newline_line=n,function=function,instruction=text))
assert len(writes)==2 and all(v['function']=='sub_4878C0' for v in writes)
callback='\n'.join(row['text'] for row in spans['callback'])
assert 'inc     edx' in callback and 'inc     ecx' in callback
assert callback.index('mov     dword_62F4C4, ecx')<callback.index('call    sub_4D65F0')
assert "mov     ecx, 40h" in callback and 'push    0' in callback
core=ROOT/'LiTo381-Modern/src/modern_lito/native_water_clock.rs'
rust=core.read_text(encoding='utf-8-sig');assert 'Callback::Kept(_)|Callback::User(_)=>{}' in rust
result=dict(status='readonly_background_offset_callback_and_missing_current_binding_identified',
    original_PE_sha256=sha(PE),original_ASM_sha256=sha(ASM),disk_initial_raw_offsets=initial,
    disk_values_not_claimed_as_observed_runtime_state=True,native_callback_address=0x4878c0,
    native_increment=[1,1],native_counter_word_bits=32,native_delay_ms=64,native_owner_argument=0,
    native_update_before_schedule=True,keep_table=keep,background_keep_index=6,
    direct_global_write_sites=writes,indirect_writers_not_ruled_out=True,source_spans=spans,
    current_core_path=str(core),current_core_sha256=sha(core),current_dispatch_kept_callback_is_noop=True,
    remaining='Verify source start/cancel/rearm lifecycle, then bind this distinct offset producer to the shared queue. Water scroll periods, the50ms green counter and elapsed-time catchup are not established substitutes.',
    full_lifecycle_verified=False,modern_scene_binding_verified=False,product_source_changed=False,
    actual_formal_sessions=0,GUI_verified=False,whole_goal_complete=False,observed_utc=datetime.now(timezone.utc).isoformat())
output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
output.with_suffix('.auditor.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(offset_seed=initial,callback_delay_ms=64,keep_index=6,current_binding_missing=True),ensure_ascii=False))
