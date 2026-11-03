import java.nio.file.*;
import java.security.*;
import java.util.*;
import java.util.jar.*;
import org.objectweb.asm.*;
import org.objectweb.asm.tree.*;
/** Review candidate only; no deployment to the readonly formal source. */
public final class PatchSameEndpointRentalContext implements Opcodes {
 static final String OWNER="local/lito381/server/ClientSession",KEY="rentalItemSelectionContext()Z";
 public static void main(String[] args)throws Exception {
  Path jar=Path.of(args[0]);String sha=HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
  if(!sha.equals("6928a147df1e784304b9080632c56121b173bf60b172023ebd263e82fd448628"))throw new IllegalStateException("baseline drift");
  byte[] raw;try(var j=new JarFile(jar.toFile())){raw=j.getInputStream(j.getJarEntry(OWNER+".class")).readAllBytes();}
  var c=RentalReturnMethodCheck.read(raw);var before=new LinkedHashMap<String,String>();for(var m:c.methods)before.put(m.name+m.desc,RentalReturnMethodCheck.canonical(m));
  if(!c.fields.stream().anyMatch(f->f.name.equals("rentalBattleLoadoutActive")&&f.desc.equals("Z")))throw new IllegalStateException("field drift");
  var m=RentalReturnMethodCheck.method(c,KEY);var original=new LabelNode();var prefix=new InsnList();
  prefix.add(new VarInsnNode(ALOAD,0));prefix.add(new FieldInsnNode(GETFIELD,OWNER,"rentalBattleLoadoutActive","Z"));prefix.add(new JumpInsnNode(IFEQ,original));prefix.add(new InsnNode(ICONST_0));prefix.add(new InsnNode(IRETURN));prefix.add(original);prefix.add(new FrameNode(F_SAME,0,null,0,null));m.instructions.insert(prefix);
  var writer=new ClassWriter(0);c.accept(writer);byte[] changed=writer.toByteArray();var reread=RentalReturnMethodCheck.read(changed);int unchanged=0;
  for(var method:reread.methods){if((method.name+method.desc).equals(KEY))continue;if(!RentalReturnMethodCheck.canonical(method).equals(before.get(method.name+method.desc)))throw new IllegalStateException("unrelated method changed "+method.name);unchanged++;}
  if(RentalReturnMethodCheck.canonical(RentalReturnMethodCheck.method(reread,KEY)).equals(before.get(KEY)))throw new IllegalStateException("no patch");
  Files.write(Path.of(args[1]),changed);System.out.println("One rentalBattleLoadoutActive guard; unchanged methods="+unchanged);
 }
}
