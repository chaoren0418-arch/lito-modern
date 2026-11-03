package local.lito381.server;

import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.lang.reflect.*;
import java.security.MessageDigest;
import java.util.*;
import java.util.concurrent.*;

/** Real two-player sessions. Reflection observes state; never mutates game authority. */
public final class NativePeerHarness {
    static void write(Path p,String s)throws Exception{Files.writeString(p,s,StandardCharsets.UTF_8);}
    static Object field(Object o,String name)throws Exception{Field f=o.getClass().getDeclaredField(name);f.setAccessible(true);return f.get(o);}
    static String quote(String s){return "\""+s.replace("\\","\\\\").replace("\"","\\\"")+"\"";}
    static String questNpcJson(BattleQuestEncounter.Snapshot npc){var stats=SourceNpcPhysicalStats.memoryFixture(npc.npcId());return "{\"id\":"+npc.objectId()+",\"force\":"+npc.force()+",\"room_number\":"+npc.roomNumber()+",\"npc\":"+npc.npcId()+",\"graphics\":"+npc.gfx()+",\"x\":"+npc.x()+",\"y\":"+npc.y()+",\"hp\":"+npc.hp()+",\"max_hp\":"+npc.maxHp()+",\"alive\":"+npc.alive()+",\"IsTU\":"+stats.turnUndeadEligible()+",\"undead\":"+stats.undead()+",\"mr\":"+stats.magicResistance()+"}";}
    record Session(Socket socket,ClientSession owner,Thread thread,boolean game){}
    static boolean visibilityFixture(String profile) {
        return Set.of("detection-invisibility","detection-refused","invisibility-refused","invisibility-missing").contains(profile);
    }
    static AccountRepository fixture(String profile) {
        return fixture(profile,new MemoryAccountRepository("PeerFixture","TestOnlyPassword"));
    }
    static AccountRepository fixture(String profile, MemoryAccountRepository original) {
        if(profile.equals("siege-native")) {try {
            // Thirty distinct accounts/character identities exist before sessions.
            // Only identity fields differ from the original memory character rows.
            var characters=new HashMap<String,List<CharacterSummary>>();
            for(int n=3;n<=30;n++) {
                String account=String.format("Siege%02d",n);original.createAccount(account,"TestOnlyPassword");
                var template=original.characters(n%2==0?"PeerFixture":"PeerFixture2").get(0);
                var fields=CharacterSummary.class.getRecordComponents();Object[] values=new Object[fields.length];Class<?>[] types=new Class<?>[fields.length];
                for(int j=0;j<fields.length;j++){types[j]=fields[j].getType();values[j]=fields[j].getName().equals("name")?account:fields[j].getName().equals("objectId")?1000L+n:fields[j].getAccessor().invoke(template);}
                var ctor=CharacterSummary.class.getDeclaredConstructor(types);ctor.setAccessible(true);
                characters.put(account,List.of(ctor.newInstance(values)));
            }
            return (AccountRepository)java.lang.reflect.Proxy.newProxyInstance(AccountRepository.class.getClassLoader(),new Class<?>[]{AccountRepository.class},(proxy,method,values)->{
                if(method.getName().equals("characters")&&characters.containsKey(values[0]))return characters.get(values[0]);
                if(values!=null&&values.length>0&&values[0] instanceof Long id&&id>=1003&&id<=1030){values=values.clone();values[0]=(id%2==0?1L:3L);}
                return method.invoke(original,values);
            });
        }catch(ReflectiveOperationException error){throw new IllegalStateException("initial siege identity fixture",error);}}
        if(!profile.equals("world-entry")&&!profile.equals("world-tree-disabled")&&!profile.equals("ranged")&&!profile.equals("holy-weapon")&&!profile.endsWith("refused")&&!profile.equals("rental-weight")&&!profile.equals("turn-undead-quest")&&!profile.equals("turn-undead-player")&&!visibilityFixture(profile))return original;
        SourceProjectilePhysicalStats arrow=new SourceProjectilePhysicalStats(40007,-2,1,1,66,true,20);
        SourceWeaponPhysicalStats bow=new SourceWeaponPhysicalStats(19,4,1,1,15,0,0,0,0).withProjectile(arrow);
        SourceEquipmentMagicStats stats=new SourceEquipmentMagicStats(new SourceEquipmentPhysicalStats(bow,false,false),0,0);
        return (AccountRepository)java.lang.reflect.Proxy.newProxyInstance(AccountRepository.class.getClassLoader(),new Class<?>[]{AccountRepository.class},(proxy,method,values)->{
            if(profile.equals("detection-invisibility")&&method.getName().equals("learnedSkills")) {
                var skills=new HashSet<Integer>((Set<Integer>)method.invoke(original,values));
                skills.add(13);skills.add(60);return Set.copyOf(skills);
            }
            if((profile.equals("world-entry")||profile.equals("world-tree-disabled"))&&method.getName().equals("learnedSkills")) {var skills=new HashSet<Integer>((Set<Integer>)method.invoke(original,values));skills.add(131);return Set.copyOf(skills);}
            if(values!=null && values.length>0 && values[0] instanceof Number id && ((profile.equals("ranged") && id.longValue()==1)||(profile.equals("holy-weapon") && id.longValue()==1))){
                if(method.getName().equals("equipmentMagicStats"))return stats;
                if(method.getName().equals("equipmentEvaluation"))return new SourceEquipmentEvaluation(stats,SourceEquipmentVitalStats.none());
            }
            if((profile.equals("turn-undead-quest")||profile.equals("turn-undead-player"))&&method.getName().equals("characters")) {
                var fields=CharacterSummary.class.getRecordComponents();var types=Arrays.stream(fields).map(RecordComponent::getType).toArray(Class<?>[]::new);var constructor=CharacterSummary.class.getDeclaredConstructor(types);List<CharacterSummary> rows=new ArrayList<>();
                for(Object row:(List<?>)method.invoke(original,values)){
                    Object[] copy=new Object[fields.length];boolean caster=((CharacterSummary)row).objectId()==1;
                    for(int i=0;i<fields.length;i++){String name=fields[i].getName();copy[i]=profile.equals("turn-undead-quest")&&name.equals("maxHp")?60000:caster&&name.equals("maxMp")?(profile.equals("turn-undead-player")?500:5000):caster&&name.equals("intelligence")?40:fields[i].getAccessor().invoke(row);}
                    rows.add((CharacterSummary)constructor.newInstance(copy));
                }return rows;
            }
            if((profile.endsWith("refused")||profile.equals("rental-weight")) && method.getName().equals("characters") && (((profile.equals("detection-refused")||profile.equals("invisibility-refused")||profile.equals("holy-weapon-refused")||profile.equals("poison-refused")||profile.equals("cure-refused")||profile.equals("chill-touch-refused")||profile.equals("vampiric-touch-refused")||profile.matches("heal(19|35|57)-refused"))?values[0].equals("PeerFixture2"):values[0].equals("PeerFixture"))||(profile.equals("rental-weight")&&values[0].equals("PeerFixture2")))){
                var fields=CharacterSummary.class.getRecordComponents();
                var types=Arrays.stream(fields).map(RecordComponent::getType).toArray(Class<?>[]::new);
                var constructor=CharacterSummary.class.getDeclaredConstructor(types);
                List<CharacterSummary> rows=new ArrayList<>();
                for(Object row:(List<?>)method.invoke(original,values)){
                    Object[] copy=new Object[fields.length];
                    for(int i=0;i<fields.length;i++){String name=fields[i].getName();copy[i]=profile.equals("rental-weight")&&Set.of("strength","constitution","originalStrength","originalConstitution").contains(name)?1:profile.endsWith("refused")&&name.equals("maxMp")?2:fields[i].getAccessor().invoke(row);}
                    rows.add((CharacterSummary)constructor.newInstance(copy));
                }
                return rows;
            }
            try{return method.invoke(original,values);}catch(InvocationTargetException ex){throw ex.getCause();}
        });
    }
    static String observe(Session s,GameHub hub)throws Exception {
        ClientSession c=s.owner();
        StringJoiner characters=new StringJoiner(",");
        for(Object row:(List<?>)field(c,"characters"))characters.add(quote(((CharacterSummary)row).name()));
        long roomId=(long)field(c,"currentRoomId");
        String roomJson="null";
        int siegeAttacker; synchronized(hub){siegeAttacker=hub.siegeAttackerForce(roomId);}
        Object timed=field(c,"timedStatus");Method poisonMethod=timed.getClass().getDeclaredMethod("poisonActive",long.class);poisonMethod.setAccessible(true);
        boolean poisonActive;long poisonExpires,poisonNext; synchronized(timed){poisonActive=(boolean)poisonMethod.invoke(timed,System.nanoTime());poisonExpires=(long)field(timed,"poisonExpiresAtNanos");poisonNext=(long)field(timed,"poisonNextTickAtNanos");}
        long gems=((RuntimeInventory)field(c,"inventory")).items().stream().filter(i->i.definition().catalogId()==40318).mapToLong(RuntimeInventory.Item::quantity).sum();
        Object buff=field(c,"weaponBuffStatus");
        Method bm=buff.getClass().getDeclaredMethod("modifier",long.class);bm.setAccessible(true);
        String buffJson=quote(String.valueOf(bm.invoke(buff,System.nanoTime())));
        Object hotel=field(c,"rentalHotelLease");
        String hotelJson=hotel==null?"null":"{\"token\":"+field(hotel,"token")+",\"keys\":"+field(hotel,"keyCount")+",\"map\":"+((NativeLitoRentalHotelCatalog.Room)field(hotel,"room")).mapId()+"}";
        StringJoiner nativeInventory=new StringJoiner(",");
        for(var item:((RuntimeInventory)field(c,"inventory")).items())nativeInventory.add("{\"id\":"+item.objectId()+",\"count\":"+item.quantity()+",\"catalog_id\":"+item.definition().catalogId()+",\"name\":"+quote(item.definition().name())+",\"equipped\":"+item.equipped()+",\"expires_at\":"+item.expiresAtEpochMillis()+"}");
        StringJoiner rentalInventory=new StringJoiner(",");
        for(var item:((RuntimeInventory)field(c,"rentalTownInventory")).items())rentalInventory.add("{\"id\":"+item.objectId()+",\"count\":"+item.quantity()+",\"catalog_id\":"+item.definition().catalogId()+",\"name\":"+quote(item.definition().name())+",\"equipped\":"+item.equipped()+",\"expires_at\":"+item.expiresAtEpochMillis()+"}");
        synchronized(hub){
            // Synthetic rental rooms are not GameHub lobby rooms; roomSettings otherwise returns a fallback.
            var settings=roomId>0 && !(boolean)field(c,"rentalTownRoomState")?hub.roomSettings(roomId):null;
            if(settings!=null){
                StringJoiner members=new StringJoiner(",");
                for(var member:hub.roomMembers(roomId))members.add(quote(member.litoDisplayName()));
                roomJson="{\"id\":"+roomId+",\"name\":"+quote(settings.name())+",\"capacity\":"+settings.capacity()+
                    ",\"password_required\":"+!settings.password().isEmpty()+",\"owner\":"+quote(hub.roomOwnerDisplayName(roomId))+",\"members\":["+members+"]}";
            }
        }
        BattleDoor.Snapshot door=hub.battleDoor((long)field(c,"currentRoomId"));
        String doorJson=door==null?"null":"{\"id\":"+door.objectId()+",\"x\":"+door.x()+",\"y\":"+door.y()+",\"action\":"+door.action()+",\"passable\":"+door.passable()+",\"open\":"+door.open()+"}";
        StringJoiner quest=new StringJoiner(",");
        for(var npc:hub.visibleQuestMonsters(roomId,c))quest.add(questNpcJson(npc));
        StringJoiner corpses=new StringJoiner(",");
        // Observe stored corpse records directly. questCorpse/findCorpse prune
        // expired authority state, so those methods must not be called here.
        synchronized(hub){
            Object room=((Map<?,?>)field(hub,"rooms")).get(roomId);
            Object encounter=room==null?null:field(room,"questEncounter");
            if(encounter!=null)synchronized(encounter){
                long observedNow=System.nanoTime();
                for(Object corpse:(List<?>)field(encounter,"corpses")){
                    if((long)field(corpse,"expiresAtNanos")>observedNow)
                        corpses.add(questNpcJson((BattleQuestEncounter.Snapshot)field(corpse,"snapshot")));
                }
            }
        }
        StringJoiner guardians=new StringJoiner(",");
        for(var tower:hub.guardianTowers(roomId))guardians.add("{\"id\":"+tower.objectId()+",\"force\":"+tower.force()+",\"x\":"+tower.x()+",\"y\":"+tower.y()+",\"gfx\":"+tower.gfx()+",\"hp\":"+tower.hp()+",\"maximum_hp\":"+tower.maxHp()+",\"action\":"+tower.action()+",\"destroyed\":"+tower.destroyed()+",\"placement_source\":"+quote(tower.placementSource())+"}");
        return "{\"siege_attacker_force\":"+siegeAttacker+",\"classic_world_map\":"+field(c,"classicWorldMapId")+",\"classic_world_instance\":"+field(c,"classicWorldInstanceId")+",\"quest_monsters\":["+quest+"],\"quest_corpses\":["+corpses+"],\"poison_active\":"+poisonActive+",\"poison_expires_at_nanos\":"+poisonExpires+",\"poison_next_tick_at_nanos\":"+poisonNext+",\"magic_gems\":"+gems+",\"weapon_buff\":"+buffJson+",\"session_identity\":"+System.identityHashCode(c)+",\"test_account\":"+quote(String.valueOf(field(c,"account")))+",\"guardian_towers\":["+guardians+"],\"object_id\":"+c.playerObjectId()+",\"phase\":"+quote(String.valueOf(field(c,"phase")))+",\"invisible_active\":"+field(c,"invisible")+
            ",\"room\":"+field(c,"currentRoomId")+",\"room_state\":"+roomJson+",\"rental_town\":"+field(c,"rentalTownRoomState")+",\"rental_map\":"+field(c,"rentalTownMapId")+",\"rental_set\":"+field(c,"rentalItemSetSelection")+",\"observer\":"+c.isObserver()+",\"channel\":"+field(c,"currentChannelId")+
            ",\"x\":"+field(c,"positionX")+",\"y\":"+field(c,"positionY")+",\"heading\":"+field(c,"playerHeading")+",\"door\":"+doorJson+
            ",\"hp\":"+field(c,"playerHp")+",\"mp\":"+field(c,"playerMp")+",\"shop_points\":"+field(c,"shopPoints")+",\"pending_transition\":"+quote(String.valueOf(field(c,"pendingTransition")))+",\"rental_gems\":"+field(c,"rentalTownLightGemCount")+",\"rental_inventory\":["+rentalInventory+"],\"native_inventory\":["+nativeInventory+"]"+
            ",\"rental_hotel\":"+hotelJson+",\"hotel_input_pending\":"+(field(c,"pendingRentalHotelInput")!=null)+",\"rental_citizen\":"+field(c,"rentalTownCitizen")+",\"rental_emblems\":"+field(c,"rentalTownEmblemsHeld")+
            ",\"available_characters\":["+characters+"],\"character\":"+quote(String.valueOf(c.selectedCharacterName()))+",\"source\":"+quote(String.valueOf(field(c,"selectedClassicServerName")))+"}";
    }
    public static void main(String[] args)throws Exception {
        Path root=Path.of(args[0]).toAbsolutePath().normalize();Files.createDirectories(root);
        String profile=args.length>1?args[1]:"melee";
        if(!Set.of("melee","heal","heal19","heal19-full","heal19-refused","heal35","heal35-full","heal35-refused","heal57","heal57-full","heal57-refused","bolt","bolt-refused","fire-arrow","fire-arrow-refused","wind-cutter","wind-cutter-refused","ice-arrow","ice-arrow-refused","holy-weapon","holy-weapon-missing","holy-weapon-refused","poison-cure","poison-missing","poison-refused","cure-refused","chill-touch","chill-touch-full","chill-touch-refused","vampiric-touch","vampiric-touch-full","vampiric-touch-refused","stalac","stalac-refused","lightning","lightning-refused","lightning-multi","turn-undead-quest","turn-undead-refused","turn-undead-player","guardian-attack","wall","wall-refused","ranged","item-heal","doors","movement","honor-respawn","room-admin","siege-native","detection-invisibility","detection-refused","invisibility-refused","invisibility-missing","world-entry","world-tree-disabled","classic-entry","rental-entry","rental-shop","rental-citizen","rental-hotel","rental-hotel-entry","rental-hotel-middle-entry","rental-hotel-expiry","rental-hotel-middle-expiry","rental-hotel-share","rental-merchants","rental-weight","rental-item-expiry","rental-equipped-expiry","rental-return").contains(profile))throw new IllegalArgumentException("profile");
        int peerCount=profile.equals("siege-native")?30:profile.equals("lightning-multi")?3:2;
        Path jar=Path.of(ClientSession.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        try(ServerSocket login=new ServerSocket(0,profile.equals("siege-native")?64:4,InetAddress.getByName("127.0.0.1"));
            ServerSocket game=new ServerSocket(0,profile.equals("siege-native")?64:4,InetAddress.getByName("127.0.0.2"))){
            login.setSoTimeout(45000);game.setSoTimeout(45000);
            Path configPath=root.resolve("server.properties");
            write(configPath,"bind=127.0.0.1\ngame.bind=127.0.0.2\nport="+game.getLocalPort()+"\nadvertised.host=127.0.0.2\nadvertised.port="+game.getLocalPort()+
                "\nserver.name=LiTo 重建測試\nlogin.classic.server.names=測試甲,測試乙\ndatabase.mode=memory\ndatabase.read.only=true\nstate.persistence.enabled=false\n"+
                "state.file=state.properties\ncapture.directory=captures\nmaps.directory=maps\ndatabase.legacy.properties=unused-sql.properties\nsource.servers.file=\n"+
                "login.bootstrap.sequence=login_ok,time_limit,classic_server_list,lito_server_list\nlito.rental.town.directory.enabled="+profile.startsWith("rental-")+"\nmax.frame=65535\n"+
                (profile.equals("world-entry")?"classic.world.compatibility.enabled=true\n":profile.equals("world-tree-disabled")?"classic.world.compatibility.enabled=false\n":"")+
                "battle.ai.enabled="+profile.equals("turn-undead-quest")+"\n"+
                "curse.poison.spell.fixture.enabled="+profile.startsWith("poison-")+"\n"+
                "lito.rental.town.directory.host=127.0.0.2\nlito.rental.town.directory.port="+game.getLocalPort()+"\nlito.rental.town.room.id=60001\nlito.rental.town.map=500\n");
            ServerConfig config=ServerConfig.load(configPath);
            if(profile.equals("siege-native")&&!config.nativeSiegeRosterEnforced())throw new IllegalStateException("native 15..50 roster must remain enforced");
            if(!config.databaseMode().equals("memory")||!config.databaseReadOnly()||config.statePersistenceEnabled())throw new IllegalStateException("isolation config");
            for(RecordComponent component:ServerConfig.class.getRecordComponents())if(component.getType()==Path.class){Path v=(Path)component.getAccessor().invoke(config);if(v!=null&&!v.toAbsolutePath().normalize().startsWith(root))throw new IllegalStateException("external path");}
            var routes=new LinkedHashMap<String,AccountRepository>();
            routes.put("測試甲",new MemoryAccountRepository("PeerFixture","TestOnlyPassword"));
            var memoryBacking=new MemoryAccountRepository("PeerFixture","TestOnlyPassword");
            AccountRepository second=fixture(profile,memoryBacking);routes.put("測試乙",second);
            if(profile.equals("turn-undead-quest")&&(config.turnUndeadFixtureDieRoll()!=0||config.turnUndeadFixturePercentileRoll()!=0))throw new IllegalStateException("natural TU probability required");
            GameStateStore state=new GameStateStore(config.stateFile(),false);
            if(visibilityFixture(profile)) {
                if(config.nativeTournamentEconomyEnabled())throw new IllegalStateException("ordinary memory economy required");
                for(String account:List.of("PeerFixture","PeerFixture2"))
                    for(var row:second.characters(account))
                        state.saveEconomyState(GameStateStore.sourceCharacter("測試乙",row.name()),1000L,
                            profile.equals("invisibility-missing")?Map.of():Map.of(40318L,8),Map.of());
            }
            if(profile.startsWith("poison-")){
                if(config.cursePoisonFixtureDieRoll()!=0||config.cursePoisonFixturePercentileRoll()!=0||config.nativeTournamentEconomyEnabled())throw new IllegalStateException("natural poison fixture required");
                for(var row:second.characters("PeerFixture2"))state.saveEconomyState(GameStateStore.sourceCharacter("測試乙",row.name()),1000L,profile.equals("poison-missing")?Map.of():Map.of(40318L,8),Map.of());
            }
            if(profile.equals("item-heal")){
                if(config.nativeTournamentEconomyEnabled())throw new IllegalStateException("normal fixture economy required");
                for(var row:second.characters("PeerFixture2"))state.saveEconomyState(GameStateStore.sourceCharacter("測試乙",row.name()),1000L,Map.of(40010L,2),Map.of());
            }
            // These two peers both select test source 1 (測試乙). Supply its
            // actual memory backing to the hub's explicit memory-fixture policy
            // branch; sessions retain the source router and initial overlays.
            // This is not a cross-source repository-routing qualification.
            try(AccountRepository accounts=new SourceRoutedAccountRepository(routes);CaptureLog capture=new CaptureLog(config.captureDirectory());GameHub hub=new GameHub(config,state,memoryBacking)){
                var sessions=new CopyOnWriteArrayList<Session>();
                var failures=new ConcurrentLinkedQueue<Throwable>();
                var acceptors=new ArrayList<Thread>();
                for(boolean isGame:new boolean[]{false,true}){
                    ServerSocket listener=isGame?game:login;
                    Thread t=new Thread(()->{try{
                        for(int i=0;i<peerCount;i++){
                            Socket socket=listener.accept();
                            ClientSession c=new ClientSession(socket,isGame?ClientSession.Endpoint.GAME:ClientSession.Endpoint.LOGIN,config,accounts,state,capture,hub);
                            Thread worker=new Thread(c,"real-peer-"+(isGame?"GAME":"LOGIN")+i);
                            worker.setUncaughtExceptionHandler((ignored,error)->failures.add(error));
                            Session s=new Session(socket,c,worker,isGame);sessions.add(s);worker.start();
                        }
                    }catch(Throwable e){failures.add(e);}},"peer-acceptor-"+isGame);
                    acceptors.add(t);t.start();
                }
                try {
                    write(root.resolve("ready.json"),"{\"port\":"+login.getLocalPort()+",\"game_port\":"+game.getLocalPort()+",\"jar_sha256\":"+quote(sha)+"}");
                    var previous=new IdentityHashMap<Session,String>();int changes=0;long remainingArrows=-1;
                    long start=System.nanoTime(),end=start+(profile.equals("siege-native")?600_000_000_000L:profile.endsWith("expiry")?8_000_000_000_000L:profile.equals("rental-merchants")?900_000_000_000L:profile.equals("turn-undead-quest")?630_000_000_000L:profile.equals("guardian-attack")?1_200_000_000_000L:profile.equals("honor-respawn")?390_000_000_000L:240_000_000_000L);
                    try(var trace=Files.newBufferedWriter(root.resolve("server-trace.jsonl"),StandardCharsets.UTF_8)){
                        while(System.nanoTime()<end){
                            if(!failures.isEmpty())throw new IllegalStateException("session failure",failures.peek());
                            for(Session s:sessions)if(s.game()){
                                String account=String.valueOf(field(s.owner(),"account"));
                                String row=profile.equals("siege-native")&&account.startsWith("Siege")?
                                    "{\"test_account\":"+quote(account)+",\"object_id\":"+s.owner().playerObjectId()+",\"phase\":"+quote(String.valueOf(field(s.owner(),"phase")))+",\"room\":"+field(s.owner(),"currentRoomId")+"}":observe(s,hub);
                                if(!row.equals(previous.put(s,row))){trace.write("{\"ms\":"+(System.nanoTime()-start)/1_000_000+",\"state\":"+row+"}\n");trace.flush();changes++;}
                                if(profile.equals("ranged") && s.owner().playerObjectId()==1 && String.valueOf(field(s.owner(),"phase")).equals("BATTLE")){
                                    Method method=ClientSession.class.getDeclaredMethod("stateCharacter");method.setAccessible(true);
                                    remainingArrows=state.sourceProjectileBalance((String)method.invoke(s.owner()),new SourceProjectilePhysicalStats(40007,-2,1,1,66,true,20)).remainingCount();
                                }
                            }
                            if(sessions.size()==peerCount*2 && acceptors.stream().noneMatch(Thread::isAlive) && sessions.stream().noneMatch(s->s.thread().isAlive()))break;
                            Thread.sleep(20);
                        }
                    }
                    if(sessions.size()!=peerCount*2 || sessions.stream().anyMatch(s->s.thread().isAlive()))throw new IllegalStateException("peer sessions did not finish");
                    for(int c=0;c<4;c++){
                        for(var room:hub.roomSnapshots(c))if(!hub.roomMembers(room.id()).isEmpty())throw new IllegalStateException("room membership leaked");
                        if(hub.channelSnapshot(c).users()!=0)throw new IllegalStateException("channel membership leaked");
                    }
                    if(hub.pendingClassicSourceHandoff("PeerFixture")!=null||hub.pendingClassicSourceHandoff("PeerFixture2")!=null||hub.pendingClassicSourceHandoff("PeerFixture5")!=null)throw new IllegalStateException("handoff leaked");
                    int classicMembers; synchronized(hub){classicMembers=((Map<?,?>)field(hub,"classicWorldMembership")).size();}
                    if(profile.equals("world-entry")&&classicMembers!=0)throw new IllegalStateException("classic WORLD membership leaked");
                    int hotelLeases=((RentalTownHotelRegistry)field(hub,"rentalTownHotels")).activeLeaseCount(System.nanoTime());
                    if(hotelLeases!=0)throw new IllegalStateException("hotel lease leaked after disconnect");
                    write(root.resolve("result.json"),"{\"status\":\"sessions_closed\",\"actual_client_sessions\":"+(peerCount*2)+",\"game_sessions\":"+peerCount+",\"jar_sha256\":"+quote(sha)+
                        ",\"classic_world_members_after_disconnect\":"+classicMembers+",\"authority_mutations\":0,\"sql_access\":false,\"state_persistent\":false,\"trace_changes\":"+changes+",\"memberships_empty\":true,\"profile\":"+quote(profile)+
                        ",\"active_hotel_leases_after_disconnect\":"+hotelLeases+",\"remaining_arrows\":"+remainingArrows+",\"initial_fixture\":"+quote(visibilityFixture(profile)?"initial ordinary memory characters; positive profile adds Knight learned13/60 only, Mage original book retained; eight initial magic gems per character, zero only in missing profile; refused profiles set Mage maxMP2; original remaining character/equipment fields retained; no runtime authority mutation":(profile.equals("world-entry")||profile.equals("world-tree-disabled"))?"initial learned skill131 added to both default memory characters; original class/stat/gfx/HP/MP retained":profile.equals("turn-undead-player")?"initial casterMP500 and INT40; original HP/class/level/book/equipment and target stats unchanged; natural player TU probability":profile.equals("turn-undead-quest")?"initial isolated characters: both requested HP60000 (character record preserved, game session cap32767); original Knight source INT40 and MP5000; original learned book unchanged; original Quest waves/NPC eligibility/dice/percentile/AI unchanged":profile.equals("rental-weight")?"initial memory characters strength/constitution/originalStrength/originalConstitution 1; default rental loadout retained":profile.equals("holy-weapon")?"player 1 initial source-equipment bow; player 3 uses its originally learned skill 8; no runtime authority mutation":profile.equals("ranged")?"player 1 bow and 20 arrows":profile.startsWith("poison-")?"player 3 initial formal skill11 memory bridge; natural dice/percentile; 8 magic gems except missing profile; low MP2 only refused profile":profile.equals("vampiric-touch-refused")?"player 3 max MP2; original skill28":profile.equals("chill-touch-refused")?"player 3 max MP2; original skill10":profile.equals("cure-refused")?"player 3 max MP2; original skill9":profile.equals("holy-weapon-refused")?"player 3 max MP 2; originally learned skill 8":profile.matches("heal(19|35|57)-refused")?"player 3 initial max MP2; original learned book, stats and HP retained; no runtime mutation":profile.endsWith("refused")?"player 1 max MP 2":profile.equals("item-heal")?"player 3 potion 40010 count 2":"default memory characters")+"}");
                }finally{
                    login.close();game.close();
                    for(Session s:sessions)s.socket().close();
                    for(Thread t:acceptors)t.join(3000);
                    for(Session s:sessions){s.thread().join(3000);if(s.thread().isAlive())throw new IllegalStateException("owned worker leaked");}
                }
            }
        }
    }
}
