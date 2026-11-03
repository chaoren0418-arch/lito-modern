"""Prepare C02 pre-session character overlays; preserve real server processing."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json
ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'維護工具/candidates/native-selector-semantic-live/NativeSelectorHarness.java'
OUT=ROOT/'維護工具/candidates/native77-live'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
s=BASE.read_text(encoding='utf-8-sig')
def replace(old,new):
    global s
    assert s.count(old)==1,old[:80];s=s.replace(old,new)
replace('    static String fixtureCasterIdentity="";',r'''    static boolean disintegrateFixture(String p) {return Set.of("disintegrate","disintegrate-refused","disintegrate-lawful").contains(p);}
    static void initialDisintegrateFixture(Path root,String profile,AccountRepository accounts)throws Exception {
        StringJoiner rows=new StringJoiner(",");boolean mage=false;
        for(String account:List.of("PeerFixture","PeerFixture2"))for(var row:accounts.characters(account)) {
            boolean knows=accounts.learnedSkills(row.objectId()).contains(77);
            if(row.maxHp()!=32767)throw new IllegalStateException("initial HP overlay");
            if(row.objectId()==3) {
                mage=true;
                if(!knows||row.maxMp()!=(profile.equals("disintegrate-refused")?2:5000)||row.lawful()!=(profile.equals("disintegrate-lawful")?499:1000))throw new IllegalStateException("initial caster overlay/book");
            }
            rows.add("{"+quote("object_id")+":"+row.objectId()+","+quote("max_hp")+":"+row.maxHp()+","+quote("max_mp")+":"+row.maxMp()+","+quote("lawful")+":"+row.lawful()+","+quote("learned77")+":"+knows+"}");
        }
        if(!mage)throw new IllegalStateException("memory Mage absent");
        write(root.resolve("initial-disintegrate-fixture.json"),"{"+quote("profile")+":"+quote(profile)+","+quote("before_all_sessions")+":true,"+quote("runtime_mutations")+":0,"+quote("natural_special_critical")+":true,"+quote("rows")+":["+rows+"]}");
    }
    static String fixtureCasterIdentity="";''')
replace('&&!selectorFixture(profile))return original;','&&!selectorFixture(profile)&&!disintegrateFixture(profile))return original;')
needle='            if(selectorFixture(profile)&&method.getName().equals("characters")) {'
replace(needle,r'''            if(disintegrateFixture(profile)&&method.getName().equals("characters")) {
                var fields=CharacterSummary.class.getRecordComponents();var types=Arrays.stream(fields).map(RecordComponent::getType).toArray(Class<?>[]::new);var constructor=CharacterSummary.class.getDeclaredConstructor(types);List<CharacterSummary> rows=new ArrayList<>();
                for(Object row:(List<?>)method.invoke(original,values)) {
                    Object[] copy=new Object[fields.length];boolean mage=((CharacterSummary)row).objectId()==3;
                    for(int i=0;i<fields.length;i++) {
                        String name=fields[i].getName();Object value=fields[i].getAccessor().invoke(row);
                        if(name.equals("maxHp"))value=32767;
                        if(mage&&name.equals("maxMp"))value=profile.equals("disintegrate-refused")?2:5000;
                        if(mage&&name.equals("lawful"))value=profile.equals("disintegrate-lawful")?499L:1000L;
                        copy[i]=value;
                    }
                    rows.add((CharacterSummary)constructor.newInstance(copy));
                }return rows;
            }
'''+needle)
replace('if(!selectorFixture(profile)&&!Set.of(', 'if(!disintegrateFixture(profile)&&!selectorFixture(profile)&&!Set.of(')
replace('        int peerCount=profile.equals("siege-native")?30:',r'''        if(args.length==3&&args[2].equals("--verify-only")) {
            if(!disintegrateFixture(profile))throw new IllegalArgumentException("verify profile");
            try(AccountRepository accounts=fixture(profile)){initialDisintegrateFixture(root,profile,accounts);}
            System.out.println("PRESESSION_FIXTURE_VERIFIED="+profile);return;
        }
        int peerCount=profile.equals("siege-native")?30:''')
replace('            // These two peers both select test source 1 (測試乙). Supply its',r'''            if(disintegrateFixture(profile))initialDisintegrateFixture(root,profile,second);
            // These two peers both select test source 1 (測試乙). Supply its''')
replace('field(c,"playerMp")',r'field(c,"playerMp")+",\"lawful\":"+field(c,"playerLawful")')
replace('quote(selectorFixture(profile)?',r'quote(disintegrateFixture(profile)?"Initial isolated character HP32767; Mage MP5000 or refusal MP2, lawful1000 or refusal499; original books/classes/stats retained, natural C02 special critical; no runtime authority writes":selectorFixture(profile)?')
OUT.mkdir();(OUT/'NativeSelectorHarness.java').write_text(s,encoding='utf-8',newline='\n')
record=dict(prepared_utc=datetime.now(timezone.utc).isoformat(),base_harness=str(BASE),base_harness_sha256=sha(BASE),candidate_harness=str(OUT/'NativeSelectorHarness.java'),candidate_harness_sha256=sha(OUT/'NativeSelectorHarness.java'),
    reference_jar_sha256='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539',profiles=['disintegrate','disintegrate-refused','disintegrate-lawful'],fixture_scope='initial memory character rows only, before all LOGIN/GAME sessions; original book77 and natural critical retained',production_source_modified=False,new_sessions=0,new_casts=0,whole_goal_complete=False)
(OUT/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(OUT)
