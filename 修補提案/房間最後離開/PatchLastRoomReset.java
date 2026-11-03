import java.nio.file.*;
import java.util.*;
import java.util.jar.*;
import java.security.*;
import org.objectweb.asm.*;
import org.objectweb.asm.tree.*;
/** Owned review candidate only: tolerate an already-removed reset row. */
public final class PatchLastRoomReset implements Opcodes {
 static final String OWNER="local/lito381/server/ClientSession",HUB="local/lito381/server/GameHub";
 public static void main(String[] args)throws Exception {
  Path base=Path.of(args[0]);String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(base)));
  if(!sha.equals("9719f45db324206617726ce06d1a9e2184e9a3b3aefab3d50f3ebbdfbf09cfac"))throw new IllegalStateException("baseline drift");
  byte[] bytes;try(var jar=new JarFile(base.toFile())){bytes=jar.getInputStream(jar.getJarEntry(OWNER+".class")).readAllBytes();}
  var c=RentalReturnMethodCheck.read(bytes);var old=new LinkedHashMap<String,String>();for(var m:c.methods)old.put(m.name+m.desc,RentalReturnMethodCheck.canonical(m));
  var m=RentalReturnMethodCheck.method(c,"broadcastDefaultRoomReset(J)V");
  if(!c.fields.stream().anyMatch(f->f.name.equals("hub")&&f.desc.equals("L"+HUB+";"))||!c.fields.stream().anyMatch(f->f.name.equals("currentChannelId")&&f.desc.equals("I")))throw new IllegalStateException("field contract drift");
  var original=new LabelNode();var prefix=new InsnList();prefix.add(new VarInsnNode(ALOAD,0));prefix.add(new FieldInsnNode(GETFIELD,OWNER,"hub","L"+HUB+";"));prefix.add(new VarInsnNode(ALOAD,0));prefix.add(new FieldInsnNode(GETFIELD,OWNER,"currentChannelId","I"));prefix.add(new VarInsnNode(LLOAD,1));prefix.add(new MethodInsnNode(INVOKEVIRTUAL,HUB,"roomExists","(IJ)Z",false));prefix.add(new JumpInsnNode(IFNE,original));prefix.add(new InsnNode(RETURN));prefix.add(original);prefix.add(new FrameNode(F_SAME,0,null,0,null));m.instructions.insert(prefix);m.maxStack=Math.max(m.maxStack,4);
  var writer=new ClassWriter(0);c.accept(writer);byte[] patched=writer.toByteArray();var check=RentalReturnMethodCheck.read(patched);int unchanged=0;
  for(var method:check.methods){if(method.name.equals("broadcastDefaultRoomReset")&&method.desc.equals("(J)V"))continue;if(!RentalReturnMethodCheck.canonical(method).equals(old.get(method.name+method.desc)))throw new IllegalStateException("unrelated method changed: "+method.name);unchanged++;}
  Files.write(Path.of(args[1]),patched);System.out.println("One roomExists guard; unrelated methods unchanged="+unchanged);
 }
}
