"""Bounded preservation of the two fixed synthetic wire diagnostic directories.

No work at import. Call only in an authorized producer, after its exact child
and private Job are terminal before restoration. Never recursively delete.
"""
from pathlib import Path
import hashlib,os,re,stat,zipfile
from native_E149_recovery_contract import P,owned,require,sha,encoded
BASE=P/'target/current/verification/native-teleport-wire'

def scan(profile):
 require(profile in ['Debug','Release'],'unknown diagnostic profile')
 root=owned(BASE/profile,BASE);require(not root.exists() or root.is_dir(),'diagnostic root is not a directory')
 paths=sorted(root.iterdir()) if root.exists() else []
 require(len(paths)<=512,'diagnostic file-count bound exceeded')
 total=0;result={}
 for path in paths:
  path=owned(path,root);require(re.fullmatch('[a-z0-9-]+[.]json',path.name) and path.is_file(),'unknown diagnostic entry; preserve without changes')
  before=path.stat();total+=before.st_size
  require(before.st_nlink==1 and before.st_size<=8_388_608 and total<=33_554_432,'unreviewed diagnostic hardlink/size')
  require(before.st_file_attributes&~(stat.FILE_ATTRIBUTE_ARCHIVE|stat.FILE_ATTRIBUTE_NORMAL)==0,'unreviewed diagnostic attributes')
  with path.open('rb') as file:
   opened=os.fstat(file.fileno());blob=file.read(8_388_609);after=os.fstat(file.fileno())
  def fingerprint(info):return (info.st_dev,info.st_ino,info.st_size,info.st_mtime_ns,info.st_nlink,info.st_file_attributes)
  require(fingerprint(before)==fingerprint(opened)==fingerprint(after)==fingerprint(path.stat())
   and len(blob)==before.st_size,'diagnostic changed during capture')
  result[path.name]=dict(data=blob,sha256=hashlib.sha256(blob).hexdigest().upper(),
   metadata=dict(size=before.st_size,mtime_ns=before.st_mtime_ns,atime_ns=before.st_atime_ns,
    inode=before.st_ino,device=before.st_dev,mode=stat.S_IMODE(before.st_mode),attributes=before.st_file_attributes))
 return result

def metadata(snapshot):return {name:row['metadata'] for name,row in snapshot.items()}
def digests(snapshot):return {name:row['sha256'] for name,row in snapshot.items()}
def stable(before,after):
 return digests(before)==digests(after) and all(before[name]['metadata']['mtime_ns']==after[name]['metadata']['mtime_ns']
  and before[name]['metadata']['inode']==after[name]['metadata']['inode'] for name in before)

def archive(path,snapshot):
 with zipfile.ZipFile(path,'x',zipfile.ZIP_DEFLATED) as z:
  for name,row in snapshot.items():z.writestr(name,row['data'])
 with zipfile.ZipFile(path) as z:
  require(set(z.namelist())==set(snapshot) and all(hashlib.sha256(z.read(name)).hexdigest().upper()==row['sha256'] for name,row in snapshot.items()),
   'diagnostic archive readback differs')
 return dict(archive=str(path),archive_sha256=sha(path),files=digests(snapshot),metadata=metadata(snapshot))

class DiagnosticStore:
 def __init__(self,out,authorize_cleanup):
  self.out=owned(out);self.authorize_cleanup=authorize_cleanup;self.baseline={};self.descriptors={};self.pending=None;self.capture_attempted=False
  self.authorize_cleanup()
  for profile in ['Debug','Release']:
   snapshot=scan(profile);self.baseline[profile]=snapshot
   self.descriptors[profile]=dict(archive(self.out/(profile+'-diagnostics-before.zip'),snapshot),root=str(BASE/profile))

 def require_restored(self,profile):
  now=scan(profile);before=self.baseline[profile]
  require(digests(now)==digests(before) and all(now[name]['metadata']['mtime_ns']==before[name]['metadata']['mtime_ns'] for name in before),
   'original diagnostic bytes/timestamps not restored')

 def begin(self,profile,folder):
  self.authorize_cleanup();require(self.pending is None,'earlier diagnostic transaction is unfinished')
  self.require_restored(profile);self.pending=(profile,owned(folder,self.out));self.capture_attempted=False

 def preserve_restore(self):
  self.authorize_cleanup();require(self.pending is not None,'no original diagnostic transaction')
  require(not self.capture_attempted,'preservation already attempted; retain original archives for explicit recovery')
  self.capture_attempted=True
  profile,folder=self.pending;before=self.baseline[profile];after=scan(profile)
  proof=archive(folder/'diagnostics-after.zip',after)
  # Preserve all observed bytes before detecting unsafe deletion or drift.
  require(set(before)<=set(after),'prior diagnostic disappeared; retain archives and do not guess a writer')
  changed=[name for name,row in after.items() if name not in before or row['sha256']!=before[name]['sha256']
   or row['metadata']['mtime_ns']!=before[name]['metadata']['mtime_ns']]
  changes=[];artifacts=folder/'diagnostics';artifacts.mkdir();staging=folder/'restore';staging.mkdir()
  for name in changed:
   destination=owned(artifacts/name,self.out)
   with destination.open('xb') as file:file.write(after[name]['data'])
   require(sha(destination)==after[name]['sha256'],'changed diagnostic readback differs')
   changes.append(dict(name=name,artifact=str(destination),sha256=after[name]['sha256']))
  require(stable(after,scan(profile)),'concurrent diagnostic writer; preserve without overwriting')
  root=owned(BASE/profile,BASE)
  for name in changed:
   self.authorize_cleanup();target=owned(root/name,root)
   current=scan(profile)
   require(name in current and current[name]['sha256']==after[name]['sha256']
    and current[name]['metadata']['mtime_ns']==after[name]['metadata']['mtime_ns']
    and current[name]['metadata']['inode']==after[name]['metadata']['inode'],'diagnostic changed before single-file restoration')
   if name in before:
    original=before[name];temporary=owned(staging/name,self.out)
    with temporary.open('xb') as file:file.write(original['data'])
    require(sha(temporary)==original['sha256'],'restoration source differs from original archive')
    os.utime(temporary,ns=(original['metadata']['atime_ns'],original['metadata']['mtime_ns']))
    # Both absolute targets were checked inside the same owned rebuild root.
    os.replace(temporary,target)
   else:
    # Delete only this captured new JSON after its exact bytes were archived.
    # This is never recursive and never follows a reparse/hardlink target.
    require(sha(target)==after[name]['sha256'],'new diagnostic changed before removal')
    target.unlink()
  self.require_restored(profile)
  manifest=dict(profile=profile,after_snapshot=proof,changed_artifacts=changes,original_bytes_and_mtimes_restored=True)
  (folder/'diagnostic-restoration.json').write_bytes(encoded(manifest))
  self.pending=None
  return dict(diagnostic_changes=changes,diagnostic_after_snapshot=proof,diagnostic_restore_verified=True,
   diagnostic_restoration_manifest=str(folder/'diagnostic-restoration.json'))

 def all_restored(self):
  require(self.pending is None,'diagnostic transaction incomplete')
  for profile in self.baseline:self.require_restored(profile)
