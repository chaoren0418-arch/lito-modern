"""Retain eighteen metadata commands and rebind the original six nested paths."""
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';q=read(read(R/'工作佇列.json')['current_E149_blizzard59_development']['qualification']);p=T/'compile-native-E149-blizzard59.py';require(sha(p)==q['source_pins'][str(p)],'compiler ancestor differs');s=p.read_text('utf-8-sig');nested_source=T/'compile-native-E149-summon51-hold.py';require(sha(nested_source)==q['source_pins'][str(nested_source)],'nested compiler ancestor differs');n=nested_source.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler anchor differs '+a[:90]);s=s.replace(a,b)
replace('native-E149-blizzard59-20261024-120000','native-E149-resurrection61-poll-20261024-123400')
replace("T/'templates/native_blizzard59_request_tests.rs',T/'templates/native_blizzard59.rs',T/'templates/native_blizzard59_tests.rs'","T/'templates/native_resurrection61.rs',T/'templates/native_resurrection61_request_tests.rs',T/'templates/native_resurrection61_controller_tests.rs',T/'templates/native_resurrection61_definition_tests.rs',T/'templates/native_resurrection61_render_tests.rs'")
replace("==365,'predecessor module count differs'","==366,'predecessor module count differs'");replace("==366,'supplementary module count differs'","==367,'supplementary module count differs'");replace('module_count=366','module_count=367')
lo=s.index(" for name,row in contract['runtime_overrides'].items():\n",s.index(" modern=Path(old['generated_mod'])"));hi=s.index(" for name,row in contract['modules'].items():modern+=",lo);nl=n.index(" nested_pairs=dict(old['original_nested_runtime_pairs'])");nh=n.index(" for name,row in contract['modules'].items():modern+=",nl);s=s[:lo]+n[nl:nh]+s[hi:];replace("original_nested_runtime_pairs=old['original_nested_runtime_pairs']","original_nested_runtime_pairs=nested_pairs")
s=s.replace('blizzard59','resurrection61').replace('59 selector, twenty-five independent nodes and scene history','61 corpse selector, dead-life question/answer UI and original visuals')
out=T/'compile-native-E149-resurrection61.py';require(not out.exists(),'compiler exists');out.write_text(s,encoding='utf-8',newline='\n');print(out)
