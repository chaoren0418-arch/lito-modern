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
    record Session(Socket socket,ClientSession owner,Thread thread,boolean game){}
    static AccountRepository fixture(String profile) {
        AccountRepository original=new MemoryAccountRepository("PeerFixture","TestOnlyPassword");
        if(!profile.equals("ranged")&&!profile.endsWith("refused")&&!profile.equals("rental-weight"))return original;
        SourceProjectilePhysicalStats arrow=new SourceProjectilePhysicalStats(40007,-2,1,1,66,true,20);
        SourceWeaponPhysicalStats bow=new SourceWeaponPhysicalStats(19,4,1,1,15,0,0,0,0).withProjectile(arrow);
        SourceEquipmentMagicStats stats=new SourceEquipmentMagicStats(new SourceEquipmentPhysicalStats(bow,false,false),0,0);
        return (AccountRepository)java.lang.reflect.Proxy.newProxyInstance(AccountRepository.class.getClassLoader(),new Class<?>[]{AccountRepository.class},(proxy,method,values)->{
            if(profile.equals("ranged") && values!=null && values.length>0 && values[0] instanceof Number id && id.longValue()==1){
                if(method.getName().equals("equipmentMagicStats"))return stats;
                if(method.getName().equals("equipmentEvaluation"))return new SourceEquipmentEvaluation(stats,SourceEquipmentVitalStats.none());
            }
            if((profile.endsWith("refused")||profile.equals("rental-weight")) && method.getName().equals("characters") && (values[0].equals("PeerFixture")||(profile.equals("rental-weight")&&values[0].equals("PeerFixture2")))){
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
        Object hotel=field(c,"rentalHotelLease");
        String hotelJson=hotel==null?"null":"{\"token\":"+field(hotel,"token")+",\"keys\":"+field(hotel,"keyCount")+",\"map\":"+((NativeLitoRentalHotelCatalog.Room)field(hotel,"room")).mapId()+"}";
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
        return "{\"session_identity\":"+System.identityHashCode(c)+",\"test_account\":"+quote(String.valueOf(field(c,"account")))+",\"object_id\":"+c.playerObjectId()+",\"phase\":"+quote(String.valueOf(field(c,"phase")))+
            ",\"room\":"+field(c,"currentRoomId")+",\"room_state\":"+roomJson+",\"rental_town\":"+field(c,"rentalTownRoomState")+",\"rental_map\":"+field(c,"rentalTownMapId")+",\"rental_set\":"+field(c,"rentalItemSetSelection")+",\"observer\":"+c.isObserver()+",\"channel\":"+field(c,"currentChannelId")+
            ",\"x\":"+field(c,"positionX")+",\"y\":"+field(c,"positionY")+",\"heading\":"+field(c,"playerHeading")+",\"door\":"+doorJson+
            ",\"hp\":"+field(c,"playerHp")+",\"mp\":"+field(c,"playerMp")+",\"shop_points\":"+field(c,"shopPoints")+",\"pending_transition\":"+quote(String.valueOf(field(c,"pendingTransition")))+",\"rental_gems\":"+field(c,"rentalTownLightGemCount")+",\"rental_inventory\":["+rentalInventory+"]"+
            ",\"rental_hotel\":"+hotelJson+",\"hotel_input_pending\":"+(field(c,"pendingRentalHotelInput")!=null)+",\"rental_citizen\":"+field(c,"rentalTownCitizen")+",\"rental_emblems\":"+field(c,"rentalTownEmblemsHeld")+
            ",\"available_characters\":["+characters+"],\"character\":"+quote(String.valueOf(c.selectedCharacterName()))+",\"source\":"+quote(String.valueOf(field(c,"selectedClassicServerName")))+"}";
    }
    public static void main(String[] args)throws Exception {
        Path root=Path.of(args[0]).toAbsolutePath().normalize();Files.createDirectories(root);
        String profile=args.length>1?args[1]:"melee";
        if(!Set.of("melee","heal","bolt","bolt-refused","fire-arrow","fire-arrow-refused","wind-cutter","wind-cutter-refused","wall","wall-refused","ranged","item-heal","doors","movement","honor-respawn","room-admin","classic-entry","rental-entry","rental-shop","rental-citizen","rental-hotel","rental-hotel-entry","rental-hotel-middle-entry","rental-hotel-expiry","rental-hotel-middle-expiry","rental-hotel-share","rental-merchants","rental-weight","rental-item-expiry","rental-equipped-expiry","rental-return").contains(profile))throw new IllegalArgumentException("profile");
        Path jar=Path.of(ClientSession.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        try(ServerSocket login=new ServerSocket(0,4,InetAddress.getByName("127.0.0.1"));
            ServerSocket game=new ServerSocket(0,4,InetAddress.getByName("127.0.0.2"))){
            login.setSoTimeout(45000);game.setSoTimeout(45000);
            Path configPath=root.resolve("server.properties");
            write(configPath,"bind=127.0.0.1\ngame.bind=127.0.0.2\nport="+game.getLocalPort()+"\nadvertised.host=127.0.0.2\nadvertised.port="+game.getLocalPort()+
                "\nserver.name=LiTo 重建測試\nlogin.classic.server.names=測試甲,測試乙\ndatabase.mode=memory\ndatabase.read.only=true\nstate.persistence.enabled=false\n"+
                "state.file=state.properties\ncapture.directory=captures\nmaps.directory=maps\ndatabase.legacy.properties=unused-sql.properties\nsource.servers.file=\n"+
                "login.bootstrap.sequence=login_ok,time_limit,classic_server_list,lito_server_list\nlito.rental.town.directory.enabled="+profile.startsWith("rental-")+"\nmax.frame=65535\n"+
                "lito.rental.town.directory.host=127.0.0.2\nlito.rental.town.directory.port="+game.getLocalPort()+"\nlito.rental.town.room.id=60001\nlito.rental.town.map=500\n");
            ServerConfig config=ServerConfig.load(configPath);
            if(!config.databaseMode().equals("memory")||!config.databaseReadOnly()||config.statePersistenceEnabled())throw new IllegalStateException("isolation config");
            for(RecordComponent component:ServerConfig.class.getRecordComponents())if(component.getType()==Path.class){Path v=(Path)component.getAccessor().invoke(config);if(v!=null&&!v.toAbsolutePath().normalize().startsWith(root))throw new IllegalStateException("external path");}
            var routes=new LinkedHashMap<String,AccountRepository>();
            routes.put("測試甲",new MemoryAccountRepository("PeerFixture","TestOnlyPassword"));
            AccountRepository second=fixture(profile);routes.put("測試乙",second);
            GameStateStore state=new GameStateStore(config.stateFile(),false);
            if(profile.equals("item-heal")){
                if(config.nativeTournamentEconomyEnabled())throw new IllegalStateException("normal fixture economy required");
                for(var row:second.characters("PeerFixture2"))state.saveEconomyState(GameStateStore.sourceCharacter("測試乙",row.name()),1000L,Map.of(40010L,2),Map.of());
            }
            try(AccountRepository accounts=new SourceRoutedAccountRepository(routes);CaptureLog capture=new CaptureLog(config.captureDirectory());GameHub hub=new GameHub(config,state,accounts)){
                var sessions=new CopyOnWriteArrayList<Session>();
                var failures=new ConcurrentLinkedQueue<Throwable>();
                var acceptors=new ArrayList<Thread>();
                for(boolean isGame:new boolean[]{false,true}){
                    ServerSocket listener=isGame?game:login;
                    Thread t=new Thread(()->{try{
                        for(int i=0;i<2;i++){
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
                    long start=System.nanoTime(),end=start+(profile.endsWith("expiry")?8_000_000_000_000L:profile.equals("rental-merchants")?900_000_000_000L:profile.equals("honor-respawn")?390_000_000_000L:240_000_000_000L);
                    try(var trace=Files.newBufferedWriter(root.resolve("server-trace.jsonl"),StandardCharsets.UTF_8)){
                        while(System.nanoTime()<end){
                            if(!failures.isEmpty())throw new IllegalStateException("session failure",failures.peek());
                            for(Session s:sessions)if(s.game()){
                                String row=observe(s,hub);
                                // Optional negative-wire tests request fresh passive snapshots.
                                String account=String.valueOf(field(s.owner(),"account"));
                                String role=account.equals("PeerFixture")?"a":account.equals("PeerFixture2")?"b":null;
                                if(role!=null){
                                    Path request=root.resolve(role+"-observe-request");
                                    if(Files.exists(request)){
                                        String label=Files.readString(request,StandardCharsets.UTF_8);
                                        if(label.matches("[a-z-]{1,60}")){
                                            Path response=root.resolve(role+"-observe-"+label+".json");
                                            if(!Files.exists(response))write(response,row);
                                        }
                                    }
                                }
                                if(!row.equals(previous.put(s,row))){trace.write("{\"ms\":"+(System.nanoTime()-start)/1_000_000+",\"state\":"+row+"}\n");trace.flush();changes++;}
                                if(profile.equals("ranged") && s.owner().playerObjectId()==1 && String.valueOf(field(s.owner(),"phase")).equals("BATTLE")){
                                    Method method=ClientSession.class.getDeclaredMethod("stateCharacter");method.setAccessible(true);
                                    remainingArrows=state.sourceProjectileBalance((String)method.invoke(s.owner()),new SourceProjectilePhysicalStats(40007,-2,1,1,66,true,20)).remainingCount();
                                }
                            }
                            if(sessions.size()==4 && acceptors.stream().noneMatch(Thread::isAlive) && sessions.stream().noneMatch(s->s.thread().isAlive()))break;
                            Thread.sleep(20);
                        }
                    }
                    if(sessions.size()!=4 || sessions.stream().anyMatch(s->s.thread().isAlive()))throw new IllegalStateException("peer sessions did not finish");
                    for(int c=0;c<4;c++){
                        for(var room:hub.roomSnapshots(c))if(!hub.roomMembers(room.id()).isEmpty())throw new IllegalStateException("room membership leaked");
                        if(hub.channelSnapshot(c).users()!=0)throw new IllegalStateException("channel membership leaked");
                    }
                    if(hub.pendingClassicSourceHandoff("PeerFixture")!=null||hub.pendingClassicSourceHandoff("PeerFixture2")!=null)throw new IllegalStateException("handoff leaked");
                    int hotelLeases=((RentalTownHotelRegistry)field(hub,"rentalTownHotels")).activeLeaseCount(System.nanoTime());
                    if(hotelLeases!=0)throw new IllegalStateException("hotel lease leaked after disconnect");
                    write(root.resolve("result.json"),"{\"status\":\"sessions_closed\",\"actual_client_sessions\":4,\"game_sessions\":2,\"jar_sha256\":"+quote(sha)+
                        ",\"authority_mutations\":0,\"sql_access\":false,\"state_persistent\":false,\"trace_changes\":"+changes+",\"memberships_empty\":true,\"profile\":"+quote(profile)+
                        ",\"active_hotel_leases_after_disconnect\":"+hotelLeases+",\"remaining_arrows\":"+remainingArrows+",\"initial_fixture\":"+quote(profile.equals("rental-weight")?"initial memory characters strength/constitution/originalStrength/originalConstitution 1; default rental loadout retained":profile.equals("ranged")?"player 1 bow and 20 arrows":profile.endsWith("refused")?"player 1 max MP 2":profile.equals("item-heal")?"player 3 potion 40010 count 2":"default memory characters")+"}");
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
