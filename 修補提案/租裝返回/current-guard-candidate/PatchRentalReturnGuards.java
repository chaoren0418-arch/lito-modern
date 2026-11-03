import java.nio.file.*;
import java.util.*;
import java.util.jar.*;
import org.objectweb.asm.*;
import org.objectweb.asm.tree.*;
/** Review-only guard insertion into the exact fixed class, preserving every existing body. */
public final class PatchRentalReturnGuards implements Opcodes {
 static final String OWNER="local/lito381/server/ClientSession";
 static String canon(MethodNode m){return RentalReturnMethodCheck.canonical(m);}
 public static void main(String[] args)throws Exception{
  byte[] bytes;try(JarFile jar=new JarFile(args[0])){bytes=jar.getInputStream(jar.getJarEntry(OWNER+".class")).readAllBytes();}
  ClassNode c=RentalReturnMethodCheck.read(bytes);Map<String,String> old=new LinkedHashMap<>();for(MethodNode m:c.methods)old.put(m.name+m.desc,canon(m));
  for(String key:List.of("quitFight()V","exitRoom(Z)V")){
   MethodNode m=RentalReturnMethodCheck.method(c,key);List<AbstractInsnNode> n=new ArrayList<>();for(AbstractInsnNode i:m.instructions)if(i.getOpcode()>=0)n.add(i);
   int[] expected={ALOAD,GETFIELD,GETSTATIC,IF_ACMPNE,ALOAD,INVOKEVIRTUAL,RETURN};for(int i=0;i<expected.length;i++)if(n.get(i).getOpcode()!=expected[i])throw new IllegalStateException("Unexpected guard "+key+":"+i);
   FieldInsnNode phase=(FieldInsnNode)n.get(1),battle=(FieldInsnNode)n.get(2);MethodInsnNode settle=(MethodInsnNode)n.get(5);
   if(!phase.owner.equals(OWNER)||!phase.name.equals("phase")||!battle.name.equals("BATTLE")||!settle.owner.equals(OWNER)||!settle.name.equals("settleRequestedBattleExit")||!settle.desc.equals("()V"))throw new IllegalStateException("Guard contract drift");
   LabelNode after=((JumpInsnNode)n.get(3)).label,original=new LabelNode();InsnList prefix=new InsnList();
   prefix.add(new VarInsnNode(ALOAD,0));prefix.add(new FieldInsnNode(GETFIELD,OWNER,"rentalTownRoomState","Z"));prefix.add(new JumpInsnNode(IFEQ,original));
   if(key.startsWith("quit")){prefix.add(new VarInsnNode(ALOAD,0));prefix.add(new FieldInsnNode(GETFIELD,OWNER,"rentalTownTransferPending","Z"));prefix.add(new JumpInsnNode(IFNE,after));}
   prefix.add(new VarInsnNode(ALOAD,0));prefix.add(new FieldInsnNode(GETFIELD,OWNER,"pendingTransition","L"+OWNER+"$PendingTransition;"));
   prefix.add(new FieldInsnNode(GETSTATIC,OWNER+"$PendingTransition",key.startsWith("quit")?"RESTART_AWAIT_QUIT_FIGHT":"RESTART_AWAIT_RETURN","L"+OWNER+"$PendingTransition;"));prefix.add(new JumpInsnNode(IF_ACMPEQ,after));
   prefix.add(original);prefix.add(new FrameNode(F_SAME,0,null,0,null));m.instructions.insertBefore(n.get(0),prefix);
  }
  ClassWriter w=new ClassWriter(0);c.accept(w);byte[] changed=w.toByteArray();ClassNode check=RentalReturnMethodCheck.read(changed);int preserved=0;
  for(MethodNode m:check.methods){String key=m.name+m.desc;if(key.equals("quitFight()V")||key.equals("exitRoom(Z)V"))continue;if(!canon(m).equals(old.get(key)))throw new IllegalStateException("Unrelated instructions changed "+key);preserved++;}
  Files.write(Path.of(args[1]),changed);System.out.println("Exactly 2 guarded prefixes inserted; original instructions retained; other methods unchanged="+preserved);
 }
}
