package local.lito381.server;
import java.net.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
public final class R509NativeFixtureMain {
 static Path root;
 static void write(String name,Object value)throws Exception{Files.writeString(root.resolve(name),NativeLetterAuthenticatedWireMain.json(value),StandardCharsets.UTF_8);}
 static void character(NativeLetterAuthenticatedWireMain base,ModernSettlementMysqlFixture mysql,String db,String account,String name,int id,int level)throws Exception{
  base.sql(mysql,db,"INSERT INTO characters(account_name,char_name,objid,level,HighLevel,MaxHp,CurHp,MaxMp,CurMp,Ac,Str,Con,Dex,Cha,Intel,Wis,Class,Sex,Type,Food,OriginalStr,OriginalCon,OriginalDex,OriginalCha,OriginalInt,OriginalWis) VALUES('"+account+"','"+name+"',"+id+","+level+","+level+",200,200,100,100,10,18,18,13,12,8,9,61,0,1,100,18,18,13,12,8,9)");
  base.sql(mysql,db,"INSERT INTO character_other(char_obj_id) VALUES("+id+")");base.sql(mysql,db,"INSERT INTO lito_character_origin(char_obj_id,account_name,char_name,created_at) VALUES("+id+",'"+account+"','"+name+"',NOW())");
 }
 public static void main(String[]args)throws Exception{
  root=NativeLetterAuthenticatedWireMain.checkedOwnedRoot(Path.of(args[0]));Path work=Path.of(args[1]).toRealPath();if(!root.getParent().equals(Path.of("D:/lito-dev-runtime/20261003-supervision/modern").toRealPath())||!root.getFileName().toString().matches("nw-[a-f0-9]{32}")||!Path.of(System.getProperty("java.io.tmpdir")).toRealPath().equals(root))throw new IllegalArgumentException("owned namespace");
  var base=new NativeLetterAuthenticatedWireMain(root,work);if(!Path.of(AccountRepository.class.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath().equals(base.jar.toRealPath())||!NativeLetterAuthenticatedWireMain.hash(base.jar).equals("AD368F49CF47E37A76A954AAFAAE1962AB2AB761057AAD4EE3145F7BBFD71219"))throw new IllegalStateException("actual AD368 repository required");
  long pid=-1;var result=new LinkedHashMap<String,Object>();
  try(var mysql=ModernSettlementMysqlFixture.open(Path.of("C:/Program Files/MySQL/MySQL Server 5.5"))){
   pid=mysql.processId();base.schema(mysql);int port;try(var s=new ServerSocket(0,1,InetAddress.getByName("127.0.0.1"))){port=s.getLocalPort();}Path profile=base.profile(mysql,port);base.seed(mysql,profile);
   try(var routes=AccountRepository.open(ServerConfig.load(profile))){for(String account:List.of("RosterEmpty","RosterMulti","RosterLow"))if(routes.createAccount(account,"WireOnly1")!=AccountMutationResult.SUCCESS)throw new IllegalStateException("new owned synthetic account");}
   for(String db:List.of(NativeLetterAuthenticatedWireMain.A,NativeLetterAuthenticatedWireMain.B)){character(base,mysql,db,"RosterMulti","測試甲",10201,52);character(base,mysql,db,"RosterMulti","測試乙",10202,52);character(base,mysql,db,"RosterLow","LowRow",10203,10);}
   String physical;int sqlPort;try(var c=mysql.openConnection(ModernSettlementMysqlFixture.deadline(10));var s=c.createStatement();var rs=s.executeQuery("SELECT @@datadir,@@port")){if(!rs.next())throw new IllegalStateException("identity");physical=rs.getString(1);sqlPort=rs.getInt(2);}
   if(!Path.of(physical).toRealPath().equals(mysql.dataDirectory().toRealPath())||!mysql.dataDirectory().startsWith(root))throw new IllegalStateException("datadir identity");
   result.put("state","READY_OWNED_NATIVE_SQL");result.put("profile",profile.toString());result.put("port",port);result.put("sql_port",sqlPort);result.put("datadir",mysql.dataDirectory().toString());result.put("physical_datadir",Path.of(physical).toRealPath().toString());result.put("mysql_pid",pid);result.put("helper",NativeLetterAuthenticatedWireMain.process(ProcessHandle.current()));result.put("retained_data",true);result.put("old_acceptance_flows_executed",0);result.put("GUI",0);result.put("new_cases",List.of("zero","two MS950 rows","single alias plus inventory","low-level channel refusal"));write("r509-ready.json",result);
   long end=System.nanoTime()+java.util.concurrent.TimeUnit.SECONDS.toNanos(240);while(!Files.exists(root.resolve("r509-stop.request"))){if(System.nanoTime()>end)throw new IllegalStateException("owned deadline");Thread.sleep(50);}result.put("state","OWNED_SQL_STOP_REQUESTED");
  }finally{result.put("mysql_retired",pid>0&&!ProcessHandle.of(pid).map(ProcessHandle::isAlive).orElse(false));write("r509-sql-retirement.json",result);}
 }
}
