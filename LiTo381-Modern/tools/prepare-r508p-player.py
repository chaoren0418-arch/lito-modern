from r491_common import *
root=P/'target/r508p-player-entry-candidate';root.mkdir();main=root/'LiToModern-R508P.rs'
s=(P/'target/r508-player-entry-candidate/LiToModern-R508.rs').read_text(encoding='utf-8').replace('R508','R508P').replace('r508','r508p')
s=s.replace('candidate_java_entry(profile,adapter)?;},','candidate_java_entry(profile,adapter)?},')
s=s.replace('adapter:modern_lito::native_city_runtime::login_bootstrap::JavaFixtureAdapter)->Result<()> {','adapter:modern_lito::native_city_runtime::login_bootstrap::JavaFixtureAdapter)->Result<Option<PathBuf>> {')
s=s.replace('run_synthetic_java_bootstrap(inputs,skin_inputs,profile,adapter)?;Ok(())','run_synthetic_java_bootstrap(inputs,skin_inputs,profile,adapter)?;Ok(None)')
main.write_text(s,encoding='utf-8');v=read(L/'r508-entry-source-contract.json');v['candidate_main']=pin(main);save(L/'r508p-entry-source-contract.json',v)
s=(P/'tools/build-r508-player.ps1').read_text(encoding='utf-8').replace('r508','r508p').replace('R508','R508P').replace('r508pb-library-build','r508b-library-build')
(P/'tools/build-r508p-player.ps1').write_text(s,encoding='utf-8')
save(L/'r508-static-player-first-failure.json',dict(first_main=pin(P/'target/r508-player-entry-candidate/LiToModern-R508.rs'),diagnostics=pin(L/'r508-player-build.stderr.log'),error='candidate route helper returned unit where existing route expects Option<PathBuf>; corrected in separate R508P wrapper; production library unchanged',GUI=0))
