package local.lito381.server;
import java.nio.file.*;
import java.security.MessageDigest;
import java.util.*;
/** Current deployed pickup parser, ground authority and action15 vectors. */
public final class NativeGroundPickupOracle {
 static byte[] request(int x,int y,long id,long quantity,int auxiliary){return new Protocol.Writer().c(11).h(Protocol.groundPickupWireX(x)).h(y).d(id).d(quantity).c(auxiliary).bytes();}
 static void reject(byte[] b){try{Protocol.groundPickup(b);throw new AssertionError("bad pickup accepted");}catch(IllegalArgumentException expected){}}
 public static void main(String[] args)throws Exception {
  LinkedHashMap<String,byte[]> p=new LinkedHashMap<>();long id=0xf1234567L;
  p.put("request",request(32729,32797,id,2,0));p.put("boundary",request(65535,0,0xffffffffL,0xffffffffL,0));p.put("zero_x",request(0,65535,id,1,0));
  var digest=MessageDigest.getInstance("SHA-256");
  for(int x=0;x<=65535;x++){int wire=Protocol.groundPickupWireX(x);digest.update((byte)wire);digest.update((byte)(wire>>8));}
  String xsha=HexFormat.of().formatHex(digest.digest());
  for(var b:p.values()){var r=Protocol.groundPickup(b);if(r.worldObjectId()==0 || r.quantity()==0 || r.auxiliary()!=0)throw new AssertionError();for(int n=0;n<b.length;n++)reject(Arrays.copyOf(b,n));reject(Arrays.copyOf(b,b.length+1));}
  reject(request(0,0,0,1,0));reject(request(0,0,id,0,0));
  var gem=InventoryCatalog.require(40318);
  p.put("ground",Protocol.groundItem(id,32729,32797,gem.groundGfx(),4,"Magic Gem (4)"));
  p.put("partial",Protocol.groundItem(id,32729,32797,gem.groundGfx(),2,"Magic Gem (2)"));
  p.put("removed",new Protocol.Writer().c(21).d(id).bytes());
  p.put("action",Protocol.objectAction(1,15));p.put("direction",new Protocol.Writer().c(28).d(1).c(0).bytes());
  p.put("inventory",Protocol.addInventory(new Protocol.InventoryWireItem(0xe1234567L,0,0,80,0,2,1,"Magic Gem (2)",new byte[0])));
  var ground=new BattleGroundItems();var item=ground.put(gem,4,32729,32797);long actualId=item.objectId();
  if(ground.take(actualId,5)!=null || ground.item(actualId).quantity()!=4)throw new AssertionError("overdraw");
  if(ground.take(actualId,2)==null || ground.item(actualId).quantity()!=2 || ground.take(actualId,2)==null || ground.item(actualId)!=null || ground.take(actualId,1)!=null)throw new AssertionError("partial/full/stale take");
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"wire_x_all_u16_sha256\":\"").append(xsha).append("\",\"packets\":{");boolean comma=false;
  for(var v:p.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(v.getKey()).append("\":\"").append(HexFormat.of().formatHex(v.getValue())).append('"');}
  System.out.println(out.append("},\"ground_take_partial_full_stale_overdraw_verified\":true}"));
 }
}
