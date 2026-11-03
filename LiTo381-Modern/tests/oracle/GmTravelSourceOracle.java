package local.lito381.server;
import java.nio.file.*;
import java.nio.*;
import java.sql.*;
import java.lang.reflect.*;
import java.util.*;

/** Reads one static fixture row through a SELECT-only account before any game session. */
public final class GmTravelSourceOracle {
 static ServerConfig withSource(ServerConfig template, Path config)throws Exception {
  var components=ServerConfig.class.getRecordComponents();
  Class<?>[] types=new Class<?>[components.length];Object[] values=new Object[components.length];
  for(int i=0;i<components.length;i++){types[i]=components[i].getType();values[i]=components[i].getName().equals("source")?config:components[i].getAccessor().invoke(template);}
  return ServerConfig.class.getDeclaredConstructor(types).newInstance(values);
 }
 public static void main(String[] args)throws Exception {
  Path root=Path.of(args[0]).toAbsolutePath().normalize();int port=Integer.parseInt(args[1]);
  Class.forName("com.mysql.cj.jdbc.Driver");
  String url="jdbc:mysql://127.0.0.1:"+port+"/travel_static?useSSL=false&serverTimezone=UTC&characterEncoding=UTF-8";
  try(Connection c=DriverManager.getConnection(url,"travel_read","TravelFixtureOnly")){
   c.setReadOnly(true);try(PreparedStatement s=c.prepareStatement("SELECT item_id,name,classname,use_type,invgfx FROM etcitem WHERE item_id=?")){
    s.setInt(1,42501);try(ResultSet r=s.executeQuery()){
     if(!r.next()||r.getInt(1)!=42501||!r.getString(2).equals("GM瞬間移動")||!r.getString(3).equals("wand.Storm_Walk")||!r.getString(4).equals("spell_long")||r.getInt(5)!=828)throw new IllegalStateException("original static row mismatch");
     if(ClassicWorldDropCatalog.nativeUseType("wand","spell_long")!=5)throw new IllegalStateException("use type mapping drift");
    }
   }
  }
  Path memoryConfig=root.resolve("memory-config.properties"),config=root.resolve("travel-oracle.properties");
  Files.writeString(memoryConfig,"bind=127.0.0.1\nport=60000\ndatabase.mode=memory\ndatabase.read.only=true\nstate.persistence.enabled=false\nstate.file=state.properties\ncapture.directory=captures\nmaps.directory=maps\ndatabase.legacy.properties=unused.properties\nsource.servers.file=\n");
  Files.writeString(config,"bind=127.0.0.1\ngame.bind=127.0.0.1\nport=60000\nserver.name=TravelSourceOracle\ndatabase.lito.database=travel_unused\nweight.rate=1\ncapture.directory=captures\nlogs.directory=logs\nbattle.ai.enabled=false\nbattle.physical.hit.enabled=true\nbattle.natural.mp.regen.enabled=false\nnative.siege.roster.enforced=false\nnative.tournament.allowance=0\nmemory.initial=64m\nmemory.maximum=256m\ncpu.limit=1\nstartup.timeout.seconds=30\ndatabase.static.database=travel_static\ndatabase.jdbc.base.url=jdbc:mysql://127.0.0.1:"+port+"/\ndatabase.jdbc.options=?useSSL=false&serverTimezone=UTC&characterEncoding=UTF-8\ndatabase.user=travel_read\ndatabase.password=TravelFixtureOnly\n");
  // Assemble an immutable test configuration before any game session. Only the
  // source file points at the current unified static DB configuration; no
  // account or character repository is opened by this source-only oracle.
  GmTravel.load(withSource(ServerConfig.load(memoryConfig), config));
  Field field=GmTravel.class.getDeclaredField("stormItems");field.setAccessible(true);
  Set<?> ids=(Set<?>)field.get(null);if(!ids.equals(Set.of(42501L)))throw new IllegalStateException("formal source classifier did not load exact SQL row");
  byte[] packet=ByteBuffer.allocate(13).order(ByteOrder.LITTLE_ENDIAN).put((byte)0x1c).putInt(9100001).putInt(0).putShort((short)32778).putShort((short)32820).array();
  var parsed=Protocol.targetLocationInventoryUse(packet);
  if(parsed.inventoryObjectId()!=9100001||parsed.targetObjectId()!=0||parsed.x()!=32778||parsed.y()!=32820)throw new IllegalStateException("target-location codec");
  int rejected=0;for(int n=0;n<packet.length;n++){try{Protocol.targetLocationInventoryUse(Arrays.copyOf(packet,n));throw new IllegalStateException("truncation accepted");}catch(IllegalArgumentException expected){rejected++;}}
  try{Protocol.targetLocationInventoryUse(Arrays.copyOf(packet,14));throw new IllegalStateException("trailing byte accepted");}catch(IllegalArgumentException expected){rejected++;}
  Files.writeString(root.resolve("gm-travel-source-oracle.json"),"{\"status\":\"verified_actual_isolated_SQL_formal_GmTravel_classifier_and_target_codec\",\"catalog_id\":42501,\"use_type\":5,\"invgfx\":828,\"packet\":\""+HexFormat.of().formatHex(packet)+"\",\"malformed_rejected\":"+rejected+",\"static_row_only\":true,\"player_tables_accessed\":false,\"runtime_authority_mutations\":0,\"gameplay_or_GUI_verified\":false}\n");
  System.out.println("Formal GmTravel loaded SQL42501; native target-location13-byte codec and14 malformed variants qualified");
 }
}
