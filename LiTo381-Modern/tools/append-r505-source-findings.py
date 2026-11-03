from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
study=read(L/'r505-source-study.json');lines=Path(study['originals'][0]['path']).read_text(encoding='utf-8-sig').splitlines();data=Path(study['originals'][1]['path']).read_bytes();root=L/'r505-source-fragments'
for a,b in [(251010,251091),(253584,253681)]:
 (root/f'asm-{a}-{b}.txt').write_text('\n'.join(f'{i+1}: {lines[i]}'for i in range(a-1,b))+'\n',encoding='utf-8')
va=0x474fc0;raw=data[va-0x400000:va-0x400000+0xd0];(root/f'pe-{va:08X}.txt').write_text(f'VA {va:08X} raw offset {va-0x400000:08X}\n'+raw.hex()+'\n'+'\n'.join(f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in Cs(CS_ARCH_X86,CS_MODE_32).disasm(raw,va))+'\n',encoding='utf-8')
study['fragments']=[pin(p)for p in sorted(root.glob('*'))]
study['additional_original_only_branch']=dict(VA='00474FC0',ASM='251010..251091',effect='64B0A4 toggles1/2 (zero->1);64B0A8=3 andresult1 ->AFBC1 BD0,result2 ->AFBC0 BD1;A8=5 reverse;choosesaddress pair64B0B0 or64B0C0 via4EFE4A;close/connect;resetclock',reachability='No named474FC0/474FD0 reference found in fixed ASM;UI475F40 writesA4/A8 andaddresspairs,butdoesnotprove this anonymousblock has a reachable callback. No Modern invoker added.')
study['callers'][0]['vector_bound_correction']='476370 is fixed invalid vector<T> subscript / std::out_of_range throw path,not a producer/resizer;caller needsvalid selectedrow andvectorcount before fieldreads.'
save(L/'r505-source-study.json',study)
g=read(L/'r505-open-gaps.json');g['source_gaps'][2]['minimum_original_UI_row_conditions']='UI475F40 selectedindex atobject+30 notFFFFFFFF;valid vectorobject+1C begin/end with0x118stride;476370 isthrow onoutofbounds,not allocator;row+110 !=FFFFFFFF;additionalcapacityguard+114 vs+110 andobfuscated8316B0/B4/B8;primaryconversion+104compare before46EB30/46ED50.';g['additional_flag_switch_callback_unqualified']='00474FC0 anonymousblock and64B0A4/A8/addresspairs;no provedcaller. OriginalGUI symptomsremainfailureevidence,notoracles.';save(L/'r505-open-gaps.json',g)
print('Addedoriginal anonymous00474FC0 flagswitch andexact476370 out_of_range interpretation;no productionmutation')
