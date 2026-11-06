package local.lito381.server;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.lang.reflect.*;
import java.security.MessageDigest;
import java.util.*;

/** Two real deployed ClientSessions share memory authorities and handoff state. */
public final class NativeGameHarness {
    static void write(Path p,String text)throws Exception{Files.writeString(p,text,StandardCharsets.UTF_8);}
    static Object field(Object owner,String name)throws Exception{Field f=owner.getClass().getDeclaredField(name);f.setAccessible(true);return f.get(owner);}
    static void join(Thread thread,Socket socket)throws Exception{join(thread,socket,25000);}
    static void join(Thread thread,Socket socket,long timeout)throws Exception{thread.join(timeout);if(thread.isAlive()){socket.close();thread.join(3000);throw new IllegalStateException("session did not stop");}}
    static String quoted(String s){StringBuilder b=new StringBuilder("\"");for(char c:s.toCharArray()){if(c=='"'||c=='\\')b.append('\\').append(c);else if(c<32)b.append(String.format("\\u%04x",(int)c));else b.append(c);}return b.append('"').toString();}
    static String wireJson(Protocol.InventoryWireItem i){return "{\"id\":"+i.objectId()+",\"use_type\":"+i.useType()+",\"charge\":"+i.chargeCount()+",\"icon\":"+i.icon()+",\"blessing\":"+i.blessing()+",\"count\":"+i.count()+",\"identified\":"+i.identified()+",\"name\":"+quoted(i.name())+",\"detail\":"+java.util.stream.IntStream.range(0,i.detail().length).map(n->Byte.toUnsignedInt(i.detail()[n])).boxed().toList()+"}";}
    // Packet fixture only: uses the deployed generators and actual encrypted writer.
    // Files coordinate capture timing; state is always delivered by the TCP session.
    static void actorStates(Path root,ClientSession session,Thread thread)throws Exception {
        Method send=ClientSession.class.getDeclaredMethod("sendEncryptedFrame",byte[].class,String.class);send.setAccessible(true);
        long own=session.playerObjectId();
        for(int step=0;step<10;step++){
            if(step>0){
                byte[] packet=switch(step){
                    case 1 -> Protocol.poison(own,1);
                    case 2 -> Protocol.poison(own,2);
                    case 3 -> Protocol.invisible(own,1);
                    case 4 -> Protocol.poison(own,0);
                    case 5 -> Protocol.poison(2,1);
                    case 6 -> Protocol.invisible(2,1);
                    case 7 -> Protocol.invisible(own,0);
                    case 8 -> Protocol.invisible(2,0);
                    case 9 -> Protocol.poison(2,0);
                    default -> throw new AssertionError();
                };
                send.invoke(session,packet,"isolated_actor_state_"+step);
            }
            write(root.resolve("state-step"),Integer.toString(step));
            long until=System.nanoTime()+30_000_000_000L;
            while(!Files.exists(root.resolve("state-ack-"+step)) && thread.isAlive() && System.nanoTime()<until)Thread.sleep(20);
            if(!Files.exists(root.resolve("state-ack-"+step)))throw new IllegalStateException("state capture missing "+step);
        }
        write(root.resolve("state-fixture.json"),"{\"steps\":10,\"packets\":9,\"transport\":\"actual_deployed_encrypted_writer\",\"natural_spell_gameplay\":false}");
    }
    static void labelUpdates(Path root,ClientSession session,Thread thread)throws Exception {
        Method send=ClientSession.class.getDeclaredMethod("sendEncryptedFrame",byte[].class,String.class);send.setAccessible(true);
        long own=session.playerObjectId();
        int x=(int)field(session,"positionX"),y=(int)field(session,"positionY");
        send.invoke(session,Protocol.battleActorWithHeading(2,x+1,y+1,61,0,0,0,0,"ArenaBot","",0,0,"",0,100,1),"isolated_label_setup");
        for(int step=0;step<11;step++){
            if(step>0){
                byte[] packet=switch(step){
                    case 1 -> Protocol.title(2,"稱號更新");
                    case 2 -> Protocol.pledgeMetadata(2,"測試盟","盟員稱號",0xffffffffL);
                    case 3 -> Protocol.battleClanMetadata(2,0x80000000L,"新血盟",true);
                    case 4 -> Protocol.title(2,"");
                    case 5 -> Protocol.battleClanMetadata(2,0,"",false);
                    case 6 -> Protocol.battleClanMetadata(own,7,"自身血盟",true);
                    case 7 -> Protocol.battleClanMetadata(2,0,"12345678901234567890",true);
                    case 8 -> Protocol.pledgeMetadata(2,"","",0);
                    case 9 -> Protocol.battleClanMetadata(2,0,"",false);
                    case 10 -> Protocol.battleClanMetadata(own,0,"",false);
                    default -> throw new AssertionError();
                };
                send.invoke(session,packet,"isolated_label_update_"+step);
            }
            write(root.resolve("label-step"),Integer.toString(step));
            long until=System.nanoTime()+30_000_000_000L;
            while(!Files.exists(root.resolve("label-ack-"+step)) && thread.isAlive() && System.nanoTime()<until)Thread.sleep(20);
            if(!Files.exists(root.resolve("label-ack-"+step)))throw new IllegalStateException("label capture missing "+step);
        }
        write(root.resolve("label-fixture.json"),"{\"steps\":11,\"packets\":10,\"setup_packets\":1,\"transport\":\"actual_deployed_encrypted_writer\",\"natural_clan_gameplay\":false}");
    }
    static void emblems(Path root,ClientSession session,Thread thread,GameHub hub)throws Exception {
        Method send=ClientSession.class.getDeclaredMethod("sendEncryptedFrame",byte[].class,String.class);send.setAccessible(true);
        for(int n=0;n<2;n++){
            String identity="isolated-emblem-owner-"+n,clan="徽章盟"+n;
            hub.registerOriginalClan(identity,clan);
            byte[] pixels=new byte[384];int[] words={0,0x8000,0x7fff,0xffff,0x7c00,0x03e0,0x001f,0x4210};
            for(int i=0;i<192;i++){int v=words[(i+n)%words.length];pixels[2*i]=(byte)v;pixels[2*i+1]=(byte)(v>>8);}
            if(!hub.registerClanEmblem(identity,clan,new ClanEmblemData(n==0?0xffffffffL:0x80000000L,pixels)))throw new IllegalStateException("emblem seed");
        }
        long own=session.playerObjectId();int x=(int)field(session,"positionX"),y=(int)field(session,"positionY");
        for(int step=0;step<6;step++){
            byte[] packet=switch(step){
                case 0 -> Protocol.battleActorWithHeading(2,x+3,y+3,61,0,0,0,0,"ArenaBot","",0,0xffffffffL,"徽章盟0",0,100,1);
                case 1 -> Protocol.battleClanMetadata(own,0xffffffffL,"",false);
                case 2 -> Protocol.battleClanMetadata(2,0x80000000L,"",false);
                case 3 -> Protocol.battleClanMetadata(2,7,"",false);
                case 4 -> Protocol.battleClanMetadata(2,0,"",false);
                case 5 -> Protocol.battleClanMetadata(own,0,"",false);
                default -> throw new AssertionError();
            };
            send.invoke(session,packet,"isolated_emblem_step_"+step);
            write(root.resolve("emblem-step"),Integer.toString(step));
            long until=System.nanoTime()+20_000_000_000L;
            while(!Files.exists(root.resolve("emblem-ack-"+step)) && thread.isAlive() && System.nanoTime()<until)Thread.sleep(20);
            if(!Files.exists(root.resolve("emblem-ack-"+step)))throw new IllegalStateException("emblem capture missing "+step);
        }
        write(root.resolve("emblem-fixture.json"),"{\"steps\":6,\"seeded_clan_emblems\":2,\"natural_clan_gameplay\":false,\"response_handler\":\"actual_ClientSession_C30_GameHub\"}");
    }
    public static void main(String[] args)throws Exception {
        Path root=Path.of(args[0]).toAbsolutePath().normalize();String mode=args[1];
        if(!Set.of("room-world-emblems","room-world-labels","room-world-states","room-world-pickup","room-world-pickup-stale","room-world-stream","room-world-stream-no-reagent","room-world-stream-no-mana","room-world-item-drop","room-world-item-drop-refused","room-world-item-heal","room-world-item-heal-194","room-world-item-heal-197","inventory-actions","inventory-actions-refused","inventory","inventory-reselect","select","notices","empty","unauthorized","enter","switch","return","reselect","restricted","room-join","room-ready","room-leave","room-rejoin","room-reject","room-retry","room-world","room-world-move","room-world-correct","room-world-route","room-world-attack","room-world-victory","room-world-defeat","room-world-refused","room-world-cycle","room-world-ranged","room-world-heal","room-world-shield","room-world-bolt","room-world-bolt-refused","room-world-wall","room-world-wall-refused").contains(mode))throw new IllegalArgumentException("mode");
        long potionId = mode.endsWith("-194") ? 40011L : mode.endsWith("-197") ? 40012L : 40010L;
        Files.createDirectories(root);
        Path jar=Path.of(ClientSession.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        try(ServerSocket login=new ServerSocket(0,1,InetAddress.getByName("127.0.0.1"));ServerSocket game=new ServerSocket(0,1,InetAddress.getByName("127.0.0.2"))){
            login.setSoTimeout(20000);game.setSoTimeout(20000);
            Path configPath=root.resolve("server.properties");
            write(configPath,"bind=127.0.0.1\ngame.bind=127.0.0.2\nport="+game.getLocalPort()+"\nadvertised.host=127.0.0.2\nadvertised.port="+game.getLocalPort()+
                "\nserver.name=LiTo 重建測試\nlogin.classic.server.names=測試甲,測試乙\ndatabase.mode=memory\ndatabase.read.only=true\nstate.persistence.enabled=false\n"+
                "state.file=state.properties\ncapture.directory=captures\nmaps.directory=maps\ndatabase.legacy.properties=unused-sql.properties\nsource.servers.file=\n"+
                "login.bootstrap.sequence=login_ok,time_limit,classic_server_list,lito_server_list\nlito.rental.town.directory.enabled=false\nmax.frame=65535\n"+
                ((mode.equals("room-world-item-drop") || mode.startsWith("room-world-pickup"))?"native.player.drop.enabled=true\n":"")+
                (mode.equals("notices")?"login.notice.message.ids=45,438\nlogin.common.news=請選擇角色。\\n重建測試。\n":""));
            ServerConfig config=ServerConfig.load(configPath);
            if(!config.databaseMode().equals("memory")||!config.databaseReadOnly()||config.statePersistenceEnabled())throw new IllegalStateException("isolation config");
            for(RecordComponent component:ServerConfig.class.getRecordComponents())if(component.getType()==Path.class){Path value=(Path)component.getAccessor().invoke(config);if(value!=null&&!value.toAbsolutePath().normalize().startsWith(root))throw new IllegalStateException("external path");}
            LinkedHashMap<String,AccountRepository> routes=new LinkedHashMap<>();
            routes.put("測試甲",new MemoryAccountRepository("重建帳號","測試密碼"));
            MemoryAccountRepository memoryBacking=new MemoryAccountRepository("重建帳號",mode.equals("unauthorized")?"不同密碼":"測試密碼");
            AccountRepository second=memoryBacking;
            if(mode.equals("empty") || mode.equals("restricted")){
                AccountRepository original=second;
                second=(AccountRepository)java.lang.reflect.Proxy.newProxyInstance(AccountRepository.class.getClassLoader(),new Class<?>[]{AccountRepository.class},(proxy,method,values)->{
                    if(method.getName().equals("characters")){
                        if(mode.equals("empty"))return List.of();
                        List<?> rows=(List<?>)method.invoke(original,values);List<Object> changed=new ArrayList<>();
                        for(Object row:rows){RecordComponent[] components=row.getClass().getRecordComponents();Class<?>[] types=new Class<?>[components.length];Object[] fields=new Object[components.length];
                            for(int i=0;i<components.length;i++){types[i]=components[i].getType();fields[i]=components[i].getAccessor().invoke(row);if(components[i].getName().equals("level"))fields[i]=14;}
                            changed.add(row.getClass().getDeclaredConstructor(types).newInstance(fields));
                        }return changed;
                    }
                    try{return method.invoke(original,values);}catch(InvocationTargetException ex){throw ex.getCause();}
                });
            }
            if(mode.equals("room-world-ranged")){
                AccountRepository original=second;
                SourceProjectilePhysicalStats arrow=new SourceProjectilePhysicalStats(40007,-2,1,1,66,true,20);
                SourceWeaponPhysicalStats bow=new SourceWeaponPhysicalStats(19,4,1,1,15,0,0,0,0).withProjectile(arrow);
                SourceEquipmentMagicStats stats=new SourceEquipmentMagicStats(new SourceEquipmentPhysicalStats(bow,false,false),0,0);
                second=(AccountRepository)java.lang.reflect.Proxy.newProxyInstance(AccountRepository.class.getClassLoader(),new Class<?>[]{AccountRepository.class},(proxy,method,values)->{
                    if(method.getName().equals("equipmentMagicStats"))return stats;
                    if(method.getName().equals("equipmentEvaluation"))return new SourceEquipmentEvaluation(stats,SourceEquipmentVitalStats.none());
                    try{return method.invoke(original,values);}catch(InvocationTargetException ex){throw ex.getCause();}
                });
            }
            if(mode.equals("room-world-heal") || mode.equals("room-world-shield") || mode.startsWith("room-world-bolt") || mode.startsWith("room-world-wall") || mode.startsWith("room-world-stream")){
                AccountRepository original=second;
                second=(AccountRepository)java.lang.reflect.Proxy.newProxyInstance(AccountRepository.class.getClassLoader(),new Class<?>[]{AccountRepository.class},(proxy,method,values)->{
                    if(method.getName().equals("learnedSkills"))return mode.startsWith("room-world-stream")?Set.of(63):mode.startsWith("room-world-wall")?Set.of(58):mode.startsWith("room-world-bolt")?Set.of(4):Set.of(1,3);
                    if(method.getName().equals("characters")){
                        var fields=CharacterSummary.class.getRecordComponents();
                        var types=Arrays.stream(fields).map(RecordComponent::getType).toArray(Class<?>[]::new);
                        var constructor=CharacterSummary.class.getDeclaredConstructor(types);
                        List<CharacterSummary> result=new ArrayList<>();
                        for(Object row:(List<?>)method.invoke(original,values)){
                            Object[] copy=new Object[fields.length];
                            for(int i=0;i<fields.length;i++)copy[i]=fields[i].getName().equals("maxMp")?(mode.startsWith("room-world-stream")?(mode.endsWith("no-mana")?2:80):mode.startsWith("room-world-wall")?(mode.endsWith("refused")?2:120):(mode.equals("room-world-bolt-refused")?2:10)):fields[i].getAccessor().invoke(row);
                            result.add((CharacterSummary)constructor.newInstance(copy));
                        }
                        return result;
                    }

                    try{return method.invoke(original,values);}catch(InvocationTargetException ex){throw ex.getCause();}
                });
            }
            routes.put("測試乙",second);GameStateStore state=new GameStateStore(config.stateFile(),false);
            if(mode.startsWith("inventory") || mode.startsWith("room-world-item-") || mode.startsWith("room-world-stream") || mode.startsWith("room-world-pickup")){
                if(config.nativeTournamentEconomyEnabled())throw new IllegalStateException("inventory fixture requires normal memory economy");
                for(var row:second.characters("重建帳號"))state.saveEconomyState(GameStateStore.sourceCharacter("測試乙",row.name()),1000L,(mode.startsWith("room-world-stream")?(mode.endsWith("no-reagent")?Map.of(40010L,2):Map.of(40318L,2,40010L,2)):mode.startsWith("inventory-actions")?Map.of(24L,1,20304L,1,40318L,4,40010L,2):Map.of(40318L,4,potionId,2)),Map.of());
            }
            try(AccountRepository accounts=new SourceRoutedAccountRepository(routes);CaptureLog capture=new CaptureLog(config.captureDirectory());GameHub hub=new GameHub(config,state,memoryBacking)){
                write(root.resolve("ready.json"),"{\"port\":"+login.getLocalPort()+",\"game_port\":"+game.getLocalPort()+",\"jar_sha256\":\""+sha+"\"}");
                try(Socket loginSocket=login.accept()){
                    ClientSession first=new ClientSession(loginSocket,ClientSession.Endpoint.LOGIN,config,accounts,state,capture,hub);
                    Thread loginThread=new Thread(first,"isolated-LOGIN");loginThread.start();
                    String inventoryJson="[]";String gamePhase="not_connected",source="",character="",displayCharacter="";int count=0,channel=-1;long joinedRoom=0,worldObject=0,forcedPickupRemoval=0;int worldX=0,worldY=0,worldGfx=0;long remainingArrows=-1;int fixtureBotHp=-1, finalPlayerHp=-1, finalPlayerMp=-1; boolean consumed=false,readyState=false,movementLock=false;
                    if(!mode.equals("unauthorized")){
                        try(Socket gameSocket=game.accept()){
                            ClientSession next=new ClientSession(gameSocket,ClientSession.Endpoint.GAME,config,accounts,state,capture,hub);
                            Thread gameThread=new Thread(next,"isolated-GAME");gameThread.start();
                            // Current formal builds no longer create the training bot.
                            // Visual-only fixtures explicitly inject a non-authoritative display actor.
                            // Combat modes must use a real opponent; never fake their hit/HP authority.
                            if(Set.of("room-world-emblems","room-world-labels","room-world-states").contains(mode)){
                                long until=System.nanoTime()+40_000_000_000L;
                                while(!String.valueOf(field(next,"phase")).equals("BATTLE") && gameThread.isAlive() && System.nanoTime()<until)Thread.sleep(20);
                                if(!String.valueOf(field(next,"phase")).equals("BATTLE"))throw new IllegalStateException("visual fixture battle not ready");
                                Method send=ClientSession.class.getDeclaredMethod("sendEncryptedFrame",byte[].class,String.class);send.setAccessible(true);
                                int x=(int)field(next,"positionX"),y=(int)field(next,"positionY");
                                send.invoke(next,Protocol.battleActorWithHeading(2,x+3,y+3,61,0,0,0,0,"ArenaBot","",0,0,"",0,100,1),"isolated_visual_actor_not_combat_authority");
                            }
                            if(mode.equals("room-world-emblems")){
                                long until=System.nanoTime()+40_000_000_000L;
                                while(!Files.exists(root.resolve("emblem-ready")) && gameThread.isAlive() && System.nanoTime()<until)Thread.sleep(20);
                                if(!Files.exists(root.resolve("emblem-ready")) || !String.valueOf(field(next,"phase")).equals("BATTLE"))throw new IllegalStateException("emblem world not ready");
                                emblems(root,next,gameThread,hub);
                            }
                            if(mode.equals("room-world-labels")){
                                long until=System.nanoTime()+40_000_000_000L;
                                while(!Files.exists(root.resolve("label-ready")) && gameThread.isAlive() && System.nanoTime()<until)Thread.sleep(20);
                                if(!Files.exists(root.resolve("label-ready")) || !String.valueOf(field(next,"phase")).equals("BATTLE"))throw new IllegalStateException("label world not ready");
                                labelUpdates(root,next,gameThread);
                            }
                            if(mode.equals("room-world-states")){
                                long until=System.nanoTime()+40_000_000_000L;
                                while(!Files.exists(root.resolve("state-ready")) && gameThread.isAlive() && System.nanoTime()<until)Thread.sleep(20);
                                if(!Files.exists(root.resolve("state-ready")) || !String.valueOf(field(next,"phase")).equals("BATTLE"))throw new IllegalStateException("state world not ready");
                                actorStates(root,next,gameThread);
                            }
                            if(mode.equals("room-world-pickup-stale")){
                                long until=System.nanoTime()+45_000_000_000L;
                                while(forcedPickupRemoval==0 && gameThread.isAlive() && System.nanoTime()<until){
                                    long room=(long)field(next,"currentRoomId");
                                    if(room!=0 && String.valueOf(field(next,"phase")).equals("BATTLE")){
                                        var ground=hub.groundItems(room);
                                        if(ground.size()==1){var item=ground.get(0);if(item.quantity()!=4 || hub.takeGroundItem(room,next,item.objectId(),4)==null)throw new IllegalStateException("stale pickup fixture removal");forcedPickupRemoval=item.objectId();}
                                    }
                                    if(forcedPickupRemoval==0)Thread.sleep(10);
                                }
                                if(forcedPickupRemoval==0)throw new IllegalStateException("stale pickup fixture never acquired ground object");
                            }
                            if(mode.equals("room-world-victory")){
                                long until=System.nanoTime()+40_000_000_000L;
                                while(!String.valueOf(field(next,"phase")).equals("BATTLE") && gameThread.isAlive() && System.nanoTime()<until)Thread.sleep(10);
                                if(!String.valueOf(field(next,"phase")).equals("BATTLE"))throw new IllegalStateException("fixture never entered battle");
                                long battleRoom=(long)field(next,"currentRoomId");
                                fixtureBotHp=hub.damageBot(battleRoom,99);
                                if(fixtureBotHp!=1)throw new IllegalStateException("fixture bot HP preparation");
                            }
                            join(gameThread,gameSocket,mode.startsWith("room-world")?100000:25000);
                            if(mode.startsWith("inventory") || mode.startsWith("room-world-item-") || mode.startsWith("room-world-stream") || mode.startsWith("room-world-pickup")){
                                RuntimeInventory actual=(RuntimeInventory)field(next,"inventory");
                                if(!actual.catalogCounts().equals(mode.equals("room-world-pickup-stale")?Map.of(40010L,2):mode.startsWith("room-world-stream")?(mode.endsWith("no-mana")?Map.of(40318L,2,40010L,2):Map.of(40010L,2)):mode.equals("room-world-item-drop")?Map.of(40010L,2):mode.startsWith("room-world-item-heal")?Map.of(40318L,4):mode.startsWith("inventory-actions")?Map.of(24L,1,20304L,1,40318L,4,40010L,2):Map.of(40318L,4,40010L,2)))throw new IllegalStateException("inventory fixture authority changed");
                                if(mode.startsWith("inventory-actions") && actual.items().stream().anyMatch(RuntimeInventory.Item::equipped))throw new IllegalStateException("action fixture did not unequip");
                                inventoryJson=actual.items().stream().map(item->wireJson(item.wireItem())).toList().toString();
                            }
                            finalPlayerHp=(int)field(next,"playerHp");finalPlayerMp=(int)field(next,"playerMp");
                            if(mode.startsWith("room-world-pickup") && finalPlayerHp!=50)throw new IllegalStateException("pickup HP changed");
                            if(mode.startsWith("room-world-stream") && (finalPlayerHp!=50 || finalPlayerMp!=(mode.endsWith("no-mana")?2:mode.endsWith("no-reagent")?80:40)))throw new IllegalStateException("reagent HP/MP authority");
                            if(mode.equals("room-world-ranged")){
                                Method stateCharacter=ClientSession.class.getDeclaredMethod("stateCharacter");stateCharacter.setAccessible(true);
                                String owner=(String)stateCharacter.invoke(next);
                                remainingArrows=state.sourceProjectileBalance(owner,new SourceProjectilePhysicalStats(40007,-2,1,1,66,true,20)).remainingCount();
                                if(remainingArrows!=19)throw new IllegalStateException("one real ranged attack must consume one source arrow");
                            }
                            worldObject=next.playerObjectId();worldX=(int)field(next,"positionX");worldY=(int)field(next,"positionY");worldGfx=next.battleGfx();movementLock=(boolean)field(next,"movementLockPending");
                            if(mode.startsWith("room-world") && movementLock)throw new IllegalStateException("movement lock not released");
                            gamePhase=String.valueOf(field(next,"phase"));source=(String)field(next,"selectedClassicServerName");character=next.selectedCharacterName();
                            count=((List<?>)field(next,"characters")).size();
                            if(count==1){Method display=ClientSession.class.getDeclaredMethod("litoDisplayName",String.class);display.setAccessible(true);displayCharacter=(String)display.invoke(next,character);}
                            consumed=(boolean)field(next,"litoCharacterSelectionConsumed");
                            channel=(int)field(next,"currentChannelId");
                            boolean inChannel=Set.of("inventory-actions","inventory-actions-refused","inventory","inventory-reselect","enter","switch","reselect","room-leave","room-reject").contains(mode);
                            boolean inRoom=Set.of("room-join","room-ready","room-rejoin","room-retry").contains(mode);
                            joinedRoom=(long)field(next,"currentRoomId");readyState=(boolean)field(next,"ready");
                            boolean noSelection=Set.of("empty","return").contains(mode);
                            if(!gamePhase.equals(Set.of("room-world-victory","room-world-defeat","room-world-cycle").contains(mode)?"CHANNEL":mode.startsWith("room-world")?"BATTLE":inRoom?"ROOM":inChannel?"CHANNEL":"LOBBY")||!source.equals("測試乙")||((count==0)!=mode.equals("empty"))||consumed==noSelection)throw new IllegalStateException("GAME authority/selection mismatch");
                            if(inChannel && channel!=(Set.of("inventory-actions","inventory-actions-refused","inventory","enter","room-leave").contains(mode)?0:3))throw new IllegalStateException("wrong channel");
                            if(inRoom && (channel!=0 || joinedRoom==0 || readyState))throw new IllegalStateException("room did not settle");
                            if(Set.of("room-leave","room-world-victory","room-world-defeat","room-world-cycle").contains(mode) && joinedRoom!=0)throw new IllegalStateException("room membership not cleared");
                            if(mode.startsWith("room-world")){
                                long until=System.nanoTime()+3_000_000_000L;
                                while(!hub.roomMembers(joinedRoom).isEmpty() && System.nanoTime()<until)Thread.sleep(25);
                            }
                            for(int c=0;c<4;c++)for(var room:hub.roomSnapshots(c))if(!hub.roomMembers(room.id()).isEmpty())throw new IllegalStateException("room members leaked after disconnect");
                            if(mode.equals("return") && (!field(next,"pendingTransition").toString().equals("NONE") || channel!=-1))throw new IllegalStateException("return not settled");
                            for(int c=0;c<4;c++)if(hub.channelSnapshot(c).users()!=0)throw new IllegalStateException("channel membership leaked after disconnect");
                            if(hub.pendingClassicSourceHandoff("重建帳號")!=null)throw new IllegalStateException("source handoff not consumed");
                        }
                    }
                    join(loginThread,loginSocket);
                    String loginPhase=String.valueOf(field(first,"phase"));
                    if(mode.equals("unauthorized")&&(!loginPhase.equals("AUTHENTICATED")||!field(first,"selectedClassicServerName").equals("測試甲")))throw new IllegalStateException("unauthorized source advanced");
                    write(root.resolve("result.json"),"{\"status\":\"verified\",\"mode\":\""+mode+"\",\"login_phase\":\""+loginPhase+"\",\"game_phase\":\""+gamePhase+"\",\"source\":\""+source+"\",\"character\":\""+character+"\",\"display_character\":\""+displayCharacter+"\",\"count\":"+count+",\"channel\":"+channel+",\"joined_room\":"+joinedRoom+",\"world_object\":"+worldObject+",\"world_x\":"+worldX+",\"world_y\":"+worldY+",\"movement_lock_pending\":"+movementLock+",\"world_gfx\":"+worldGfx+",\"remaining_arrows\":"+remainingArrows+",\"fixture_bot_hp\":"+fixtureBotHp+",\"final_player_hp\":"+finalPlayerHp+",\"final_player_mp\":"+finalPlayerMp+",\"capture_file\":"+quoted(capture.path().toString())+",\"forced_pickup_removal\":"+forcedPickupRemoval+",\"inventory\":"+inventoryJson+",\"ready\":"+readyState+",\"room_ids_0\":"+hub.roomSnapshots(0).stream().map(GameHub.RoomSnapshot::id).toList()+",\"room_ids_3\":"+hub.roomSnapshots(3).stream().map(GameHub.RoomSnapshot::id).toList()+",\"selection_consumed\":"+consumed+",\"jar_sha256\":\""+sha+"\",\"actual_client_sessions\":true,\"sql_access\":false,\"state_persistent\":false}");
                }
            }
        }
    }
}
