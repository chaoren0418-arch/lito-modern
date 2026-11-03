package local.lito381.server;
import java.net.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import java.sql.*;

/** Modern-owned setup only. No old letter/battle acceptance is executed. */
public final class R507NativeFixtureMain {
 static Path root;
 static void write(String name,Object value)throws Exception{
  Files.writeString(root.resolve(name),NativeLetterAuthenticatedWireMain.json(value),StandardCharsets.UTF_8);
 }
 static String wireHex(byte[] body)throws Exception{
  var out=new java.io.ByteArrayOutputStream();
  Protocol.writeFrame(out,new Protocol.Writer().c(Protocol.S_KEY).d(0x12345678L).bytes());
  Protocol.writeFrame(out,new Protocol.CipherState(0x12345678).encrypt(body));
  return HexFormat.of().formatHex(out.toByteArray());
 }
 public static void main(String[] args)throws Exception{
  if(args.length!=2)throw new IllegalArgumentException("root work");
  root=Path.of(args[0]).toRealPath();Path work=Path.of(args[1]).toRealPath();
  if(!root.getParent().equals(Path.of("D:/lito-dev-runtime/20261003-supervision/modern").toRealPath())||!root.getFileName().toString().matches("nw-[a-f0-9]{32}")||!Path.of(System.getProperty("java.io.tmpdir")).toRealPath().equals(root))throw new IllegalArgumentException("owned modern namespace required");
  var base=new NativeLetterAuthenticatedWireMain(root,work);
  if(!Path.of(AccountRepository.class.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath().equals(base.jar.toRealPath()))throw new IllegalStateException("repository must load from pinned candidate JAR");
  if(!NativeLetterAuthenticatedWireMain.hash(base.jar).equals("AD368F49CF47E37A76A954AAFAAE1962AB2AB761057AAD4EE3145F7BBFD71219"))throw new IllegalStateException("candidate drift");
  long mysqlPid=-1;Map<String,Object> result=new LinkedHashMap<>();
  try(var mysql=ModernSettlementMysqlFixture.open(Path.of("C:/Program Files/MySQL/MySQL Server 5.5"))){
   mysqlPid=mysql.processId();base.schema(mysql);
   int port;try(var listen=new ServerSocket(0,1,InetAddress.getByName("127.0.0.1"))){port=listen.getLocalPort();}
   Path profile=base.profile(mysql,port);base.seed(mysql,profile);
   String physical;int sqlport;
   try(var c=mysql.openConnection(ModernSettlementMysqlFixture.deadline(10));var s=c.createStatement();var rows=s.executeQuery("SELECT @@datadir,@@port")){
    if(!rows.next())throw new IllegalStateException("identity row");physical=rows.getString(1);sqlport=rows.getInt(2);
   }
   if(!Path.of(physical).toRealPath().equals(mysql.dataDirectory().toRealPath())||!mysql.dataDirectory().startsWith(root))throw new IllegalStateException("physical datadir mismatch");
   var dirs=new ArrayList<Object>();
   for(int count:new int[]{6,10}){
    var servers=new ArrayList<Protocol.LitoVsServerDirectoryEntry>();
    for(int i=0;i<count;i++)servers.add(new Protocol.LitoVsServerDirectoryEntry("OwnedServer"+i+"-"+"x".repeat(20),0x0100007fL,2000,i,100));
    byte[] body=Protocol.litoServerDirectory(List.of(),servers);
    dirs.add(Map.of("case","legal_"+count+"_server_rows","rooms",0,"servers",count,"bytes",body.length,"body_hex",HexFormat.of().formatHex(body),"seeded_wire_hex",wireHex(body)));
   }
   var rooms=new ArrayList<Protocol.LitoRoomDirectoryEntry>();
   var servers=new ArrayList<Protocol.LitoVsServerDirectoryEntry>();
   for(int i=0;i<127;i++)rooms.add(new Protocol.LitoRoomDirectoryEntry("r".repeat(256),0x0100007fL,2000,i+1,0));
   for(int i=0;i<10;i++)servers.add(new Protocol.LitoVsServerDirectoryEntry("s".repeat(256),0x0100007fL,2000,0,100));
   byte[] max=Protocol.litoServerDirectory(rooms,servers);
   dirs.add(Map.of("case","legal_java_maximum_directory","rooms",127,"servers",10,"bytes",max.length,"body_hex",HexFormat.of().formatHex(max),"seeded_wire_hex",wireHex(max)));
   write("java-directory-vectors.json",dirs);
   result.put("state","READY_OWNED_NATIVE_SQL");result.put("profile",profile.toString());result.put("port",port);result.put("sql_port",sqlport);result.put("datadir",mysql.dataDirectory().toString());result.put("physical_datadir",Path.of(physical).toRealPath().toString());result.put("mysql_pid",mysqlPid);result.put("helper",NativeLetterAuthenticatedWireMain.process(ProcessHandle.current()));result.put("retained_data",true);result.put("old_acceptance_flows_executed",0);result.put("GUI",0);write("r507-ready.json",result);
   long end=System.nanoTime()+java.util.concurrent.TimeUnit.SECONDS.toNanos(240);
   while(!Files.exists(root.resolve("r507-stop.request"))){if(System.nanoTime()>end)throw new IllegalStateException("owned deadline");Thread.sleep(50);}
   result.put("state","OWNED_SQL_STOP_REQUESTED");
   // Deliberately do not call successful(): retain own data, no recursive deletion.
  }finally{
   result.put("mysql_retired",mysqlPid>0&&!ProcessHandle.of(mysqlPid).map(ProcessHandle::isAlive).orElse(false));write("r507-sql-retirement.json",result);
  }
 }
}
