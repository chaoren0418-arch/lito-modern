package local.lito381.server;
import java.nio.file.*;
import java.nio.*;
import java.nio.charset.*;
import java.security.*;
import java.util.*;
public final class NativeItemActionsOracle {
 static byte[] use(long id,String argument){byte[] text=argument.getBytes(Charset.forName("MS950"));return ByteBuffer.allocate(6+text.length).order(ByteOrder.LITTLE_ENDIAN).put((byte)28).putInt((int)id).put(text).put((byte)0).array();}
 static byte[] drop(int x,int y,long id,long n){return ByteBuffer.allocate(13).order(ByteOrder.LITTLE_ENDIAN).put((byte)12).putShort((short)x).putShort((short)y).putInt((int)id).putInt((int)n).array();}
 static void rejected(byte[] b,boolean drop){try{if(drop)Protocol.inventoryDrop(b);else Protocol.defaultInventoryUse(b);throw new AssertionError("accepted malformed action");}catch(IllegalArgumentException expected){}}
 public static void main(String[] args)throws Exception {
  LinkedHashMap<String,byte[]> vectors=new LinkedHashMap<>();
  vectors.put("use",use(0xfedcba98L,""));vectors.put("use_max",use(0xffffffffL,""));vectors.put("use_argument",use(7,"測試"));vectors.put("drop",drop(32768,65535,0xfedcba98L,0xffffffffL));
  for(var e:vectors.entrySet()){
   byte[] b=e.getValue();if(e.getKey().equals("drop")){var p=Protocol.inventoryDrop(b);if(p.inventoryObjectId()!=0xfedcba98L||p.quantity()!=0xffffffffL)throw new AssertionError();}
   else {var p=Protocol.defaultInventoryUse(b);if(p.inventoryObjectId()==0)throw new AssertionError();}
   for(int end=0;end<b.length;end++)rejected(Arrays.copyOf(b,end),e.getKey().equals("drop"));rejected(Arrays.copyOf(b,b.length+1),e.getKey().equals("drop"));
  }
  rejected(use(0,""),false);rejected(drop(0,0,0,1),true);rejected(drop(0,0,1,0),true);
  vectors.put("ground_gem",Protocol.groundItem(0xfedcba98L,32768,65535,InventoryCatalog.require(40318).groundGfx(),2,"測試 (2)"));
  vectors.put("ground_potion",Protocol.groundItem(0xfffffffeL,32768,65535,InventoryCatalog.require(40010).groundGfx(),2,"藥水 (2)"));
  Path jar=Path.of(Protocol.class.getProtectionDomain().getCodeSource().getLocation().toURI());String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  StringBuilder out=new StringBuilder("{\"schema\":1,\"jar_sha256\":\"").append(sha).append("\",\"all_truncations_and_trailing_rejected\":true,\"packets\":{");boolean comma=false;
  for(var v:vectors.entrySet()){if(comma)out.append(',');comma=true;out.append('"').append(v.getKey()).append("\":\"").append(HexFormat.of().formatHex(v.getValue())).append('"');}
  out.append("},\"catalog\":[");comma=false;
  for(long id:new long[]{InventoryCatalog.SHORT_SWORD,InventoryCatalog.RING_OF_FIRE_RESISTANCE,40010,40318}){var d=InventoryCatalog.require(id);if(comma)out.append(',');comma=true;out.append("{\"catalog_id\":").append(id).append(",\"use_type\":").append(d.useType()).append(",\"weapon\":").append(d.weapon()).append(",\"armor\":").append(d.armor()).append(",\"effect\":\"").append(d.effect()).append("\"}");}
  System.out.println(out.append("]}"));
 }
}
