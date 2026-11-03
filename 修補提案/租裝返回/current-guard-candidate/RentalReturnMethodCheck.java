import java.nio.file.*;
import java.util.*;
import java.util.jar.*;
import org.objectweb.asm.*;
import org.objectweb.asm.tree.*;

/** Compares actual instructions and exception regions, ignoring only debug metadata. */
public final class RentalReturnMethodCheck {
 static ClassNode read(byte[] b){ClassNode n=new ClassNode();new ClassReader(b).accept(n,0);return n;}
 static int at(AbstractInsnNode n){while(n!=null && n.getOpcode()<0)n=n.getNext();if(n==null)return -1;int i=0;for(AbstractInsnNode p=n.getPrevious();p!=null;p=p.getPrevious())if(p.getOpcode()>=0)i++;return i;}
 static String canonical(MethodNode m){StringBuilder b=new StringBuilder(m.name+m.desc+":"+m.access+":"+m.exceptions+"\n");
  for(AbstractInsnNode n:m.instructions){if(n.getOpcode()<0)continue;b.append(n.getOpcode()).append(':');
   if(n instanceof VarInsnNode v)b.append(v.var);
   else if(n instanceof FieldInsnNode v)b.append(v.owner).append('/').append(v.name).append(v.desc);
   else if(n instanceof MethodInsnNode v)b.append(v.owner).append('/').append(v.name).append(v.desc).append(':').append(v.itf);
   else if(n instanceof JumpInsnNode v)b.append(at(v.label));
   else if(n instanceof LdcInsnNode v)b.append(v.cst.getClass().getName()).append(':').append(v.cst);
   else if(n instanceof TypeInsnNode v)b.append(v.desc);
   else if(n instanceof IntInsnNode v)b.append(v.operand);
   else if(n instanceof IincInsnNode v)b.append(v.var).append(':').append(v.incr);
   else if(n instanceof InvokeDynamicInsnNode v)b.append(v.name).append(v.desc).append(v.bsm).append(Arrays.deepToString(v.bsmArgs));
   else if(n instanceof TableSwitchInsnNode v){b.append(v.min).append(':').append(v.max).append(':').append(at(v.dflt));for(LabelNode l:v.labels)b.append(':').append(at(l));}
   else if(n instanceof LookupSwitchInsnNode v){b.append(v.keys).append(':').append(at(v.dflt));for(LabelNode l:v.labels)b.append(':').append(at(l));}
   else if(n instanceof MultiANewArrayInsnNode v)b.append(v.desc).append(':').append(v.dims);
   else if(!(n instanceof InsnNode))throw new IllegalArgumentException("Unhandled instruction "+n.getClass());
   b.append('\n');
  }
  for(TryCatchBlockNode t:m.tryCatchBlocks)b.append("catch:").append(at(t.start)).append(':').append(at(t.end)).append(':').append(at(t.handler)).append(':').append(t.type).append('\n');
  return b.toString();
 }
 static MethodNode method(ClassNode c,String key){return c.methods.stream().filter(m->(m.name+m.desc).equals(key)).findFirst().orElseThrow();}
 public static void main(String[] a)throws Exception{
  Path jar=Path.of(a[0]),before=Path.of(a[1]),out=Path.of(a[2]);Files.createDirectories(out);
  byte[] formalBytes;try(JarFile j=new JarFile(jar.toFile())){formalBytes=j.getInputStream(j.getJarEntry("local/lito381/server/ClientSession.class")).readAllBytes();}
  ClassNode formal=read(formalBytes),donor=read(Files.readAllBytes(before));
  String[] keys={"quitFight()V","exitRoom(Z)V"};
  for(String key:keys){String f=canonical(method(formal,key)),d=canonical(method(donor,key));String name=key.startsWith("quit")?"quitFight":"exitRoom";Files.writeString(out.resolve(name+"-formal.txt"),f);Files.writeString(out.resolve(name+"-donor.txt"),d);if(!f.equals(d))throw new IllegalStateException("Donor differs from formal instructions: "+key);System.out.println("EXACT instructions/exception regions: "+key);}
  if(a.length==5){ClassNode changed=read(Files.readAllBytes(Path.of(a[3])));Map<String,String> baseline=new LinkedHashMap<>();for(MethodNode m:formal.methods)baseline.put(m.name+m.desc,canonical(m));
   for(String key:keys){MethodNode replacement=method(changed,key);if(canonical(replacement).equals(baseline.get(key)))throw new IllegalStateException("Patch did not change "+key);formal.methods.set(formal.methods.indexOf(method(formal,key)),replacement);}
   ClassWriter writer=new ClassWriter(0);formal.accept(writer);byte[] result=writer.toByteArray();ClassNode reread=read(result);int unchanged=0;
   for(MethodNode m:reread.methods){String key=m.name+m.desc;if(Arrays.asList(keys).contains(key))continue;if(!canonical(m).equals(baseline.get(key)))throw new IllegalStateException("Unrelated method changed "+key);unchanged++;}
   Files.write(Path.of(a[4]),result);System.out.println("PATCHED exactly 2 methods; unchanged methods="+unchanged);
  }
 }
}
