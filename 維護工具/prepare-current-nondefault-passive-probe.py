"""Stage stat-continuity sources with passive 14-field server observations."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,difflib
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('normal',type=Path);ap.add_argument('prepared',type=Path);a=ap.parse_args()
normal=a.normal.resolve(strict=True);prepared=a.prepared.resolve(strict=True)
assert normal.is_relative_to(V/'native-installed-resource-profile-normal')and prepared.is_relative_to(V/'native-installed-resource-profile-nondefault-character-fixture')
source=read(R/'工作佇列.json')['current_product_identity']['source_manifest_sha256'];qpath=normal/'qualified/qualified.json';q=read(qpath);fixture=read(prepared/'prepared.json')
assert source==q['source_manifest_sha256']==fixture['source_manifest_sha256']=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert q['recorded_owned_processes_all_terminal']and q['observed_terminal_exit_code']==0 and q['passed_per_profile']==19
for n,h in read(normal/'compiled-inputs.json').items():assert sha(R/n)==h
for n,h in fixture['fixture_sources'].items():assert sha(prepared/n)==h
prior=T/'candidates/native-nondefault-same-endpoint-20261019-202937-227052';prior_record=read(prior/'candidate.json')
assert not prior_record['compiled']and sha(prior/'probe.rs')==prior_record['probe_sha256']
old=(prepared/'NativePeerHarness.java').read_text('utf-8-sig');new=old
helper='''    /** Pure immutable record projection for the owned nondefault fixture. */
    static String characterStatRows(List<?> values)throws Exception {
        Set<String> wanted=Set.of("name","maxHp","maxMp","armorClass","strength","constitution","dexterity","charisma","intelligence","wisdom","originalStrength","originalConstitution","originalDexterity","originalIntelligence","originalWisdom");
        StringJoiner rows=new StringJoiner(",","[","]");
        for(Object value:values) {
            if(!(value instanceof CharacterSummary))throw new IllegalStateException("unexpected character record");
            Set<String> found=new HashSet<>();StringJoiner fields=new StringJoiner(",","{","}");
            for(var field:CharacterSummary.class.getRecordComponents())if(wanted.contains(field.getName())) {
                Object item=field.getAccessor().invoke(value);found.add(field.getName());
                fields.add(quote(field.getName())+":"+(item instanceof Number?item.toString():quote(String.valueOf(item))));
            }
            if(!found.equals(wanted))throw new IllegalStateException("incomplete passive stat record");
            rows.add(fields.toString());
        }
        return rows.toString();
    }
'''
anchor='    static String observe(Session s,GameHub hub)throws Exception {';assert new.count(anchor)==1;new=new.replace(anchor,helper+anchor)
anchor='        for(Object row:(List<?>)field(c,"characters"))characters.add(quote(((CharacterSummary)row).name()));';assert new.count(anchor)==1
new=new.replace(anchor,'''        List<?> characterRows=(List<?>)field(c,"characters");
        String selectedBefore=c.selectedCharacterName();
        for(Object row:characterRows)characters.add(quote(((CharacterSummary)row).name()));
        String characterStats=characterStatRows(characterRows);
        boolean characterStatsStable=field(c,"characters")==characterRows && Objects.equals(selectedBefore,c.selectedCharacterName());''')
anchor=r'",\"available_characters\":["+characters+"],\"character\":"'
# Exact Java source text: keep every original observation and append data only.
anchor='",\\\"available_characters\\\":["+characters+"],\\\"character\\\":"'
if anchor not in new:
    anchor=r'",\"available_characters\":["+characters+"],\"character\":"'
assert new.count(anchor)==1,repr(anchor)
replacement=anchor.replace(r'],\"character',r'],\"available_character_stats\":"+characterStats+",\"character_stats_observation_stable\":"+characterStatsStable+",\"character')
assert replacement!=anchor
new=new.replace(anchor,replacement)
out=T/'candidates'/('native-current-nondefault-passive-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'NativePeerHarness.java').write_text(new,encoding='utf-8',newline='\n')
for name in ['NondefaultFixture.java','NondefaultTeleportHarness.java']:(out/name).write_bytes((prepared/name).read_bytes())
(out/'probe.rs').write_bytes((prior/'probe.rs').read_bytes());(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
(out/'passive-observation.diff').write_text(''.join(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile='original-owned-peer.java',tofile='NativePeerHarness.java')),encoding='utf-8')
record=dict(status='prepared_nondefault_public_API_probe_and_passive_14_field_trace_not_compiled',base_source_manifest_sha256=source,
 normal_qualification=str(qpath),normal_qualification_sha256=sha(qpath),fixture_prepared=str(prepared),fixture_prepared_sha256=sha(prepared/'prepared.json'),
 prior_probe=str(prior/'probe.rs'),prior_probe_sha256=sha(prior/'probe.rs'),probe_sha256=sha(out/'probe.rs'),
 fixture_sources={n:sha(out/n)for n in ['NativePeerHarness.java','NondefaultFixture.java','NondefaultTeleportHarness.java']},
 peer_origin=str(prepared/'NativePeerHarness.java'),peer_origin_sha256=sha(prepared/'NativePeerHarness.java'),
 main_class='local.lito381.server.NondefaultTeleportHarness',passive_stat_fields=14,wire_stat_projection_fields=9,
 passive_snapshot_only=True,handler_or_constructor_or_clocks_changed=False,source_overlays_used=False,
 runtime_overrides=False,compiled=False,runtime_started=False,actual_sessions=0,stat_continuity_verified=False,
 original_five_wire_claimed=False,normal_source_modified=False,diagnostic_API_extension=False,GUI_verified=False,whole_goal_complete=False)
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(candidate=str(out),compiled=False,runtime_started=False,passive_fields=14,wire_fields=9),ensure_ascii=False))
