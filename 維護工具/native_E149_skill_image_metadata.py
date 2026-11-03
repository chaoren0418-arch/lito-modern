"""Raw-file x64 PE metadata reader; never loads or executes an image."""
from pathlib import Path
import hashlib,struct
from native_E149_recovery_contract import require
# Derived from qualify-native-E149-integrated-client.py SHA256 53FAD9006B10D11F381303075E2ACE588E19F32E7ABD3A85FBF69A273FC29274
def image_metadata(path,expected_subsystem):
    """Bounded raw-file PE reader; no LoadLibrary, process, or code execution."""
    require(expected_subsystem in [2,3], 'unreviewed PE subsystem'); data = Path(path).read_bytes()
    def u16(at):
        require(0 <= at <= len(data) - 2, 'PE u16 outside file'); return struct.unpack_from('<H', data, at)[0]
    def u32(at):
        require(0 <= at <= len(data) - 4, 'PE u32 outside file'); return struct.unpack_from('<I', data, at)[0]
    require(data[:2] == b'MZ', 'DOS signature absent')
    pe = u32(0x3c); require(data[pe:pe + 4] == b'PE\0\0', 'PE signature absent')
    machine = u16(pe + 4); count = u16(pe + 6); size = u16(pe + 20); optional = pe + 24
    require(machine == 0x8664 and 0 < count <= 96 and size >= 240 and u16(optional) == 0x20b, 'not Windows x64 PE32+')
    require(u16(optional + 68) == expected_subsystem and u16(pe + 22) & 2, 'expected Windows subsystem absent')
    table = optional + size; sections = []
    for i in range(count):
        at = table + 40 * i
        sections.append((u32(at + 12), u32(at + 16), u32(at + 20)))
    def file_at(rva, length):
        require(length >= 0, 'negative mapped length')
        if rva + length <= u32(optional + 60):
            require(rva + length <= len(data), 'header RVA outside file'); return rva
        matches = [raw + rva - base for base, rawsize, raw in sections if base <= rva and rva + length <= base + rawsize]
        require(len(matches) == 1 and matches[0] + length <= len(data), 'resource RVA not uniquely backed by file bytes')
        return matches[0]
    require(u32(optional + 108) >= 3, 'resource data directory absent')
    root = u32(optional + 128); root_size = u32(optional + 132)
    require(root and 16 <= root_size <= 16 * 1024 * 1024, 'resource directory size invalid')
    base = file_at(root, root_size); leaves = {}; seen = set()
    def walk(offset, keys):
        require(len(keys) <= 2 and offset not in seen and 0 <= offset <= root_size - 16, 'resource directory invalid or cyclic')
        seen.add(offset); entries = u16(base + offset + 12) + u16(base + offset + 14)
        require(entries <= 4096 and offset + 16 + entries * 8 <= root_size, 'resource directory entries exceed section')
        for i in range(entries):
            at = base + offset + 16 + i * 8; name = u32(at); value = u32(at + 4)
            # Only integer type/name/language resources are relevant here.
            if name & 0x80000000: continue
            key = (*keys, name)
            if not keys and name not in [16, 24]: continue
            if value & 0x80000000:
                walk(value & 0x7fffffff, key)
            else:
                require(len(key) == 3 and value <= root_size - 16, 'resource data entry outside section')
                rva = u32(base + value); length = u32(base + value + 4)
                require(0 < length <= 1024 * 1024 and key not in leaves, 'resource payload invalid')
                at = file_at(rva, length); leaves[key] = data[at:at + length]
    walk(0, ())
    manifest = [(k, v) for k, v in leaves.items() if k[0] == 24 and k[1] == 1]
    require(len(manifest) == 1, 'exactly one normal app manifest required')
    version = {str(k): hashlib.sha256(v).hexdigest().upper() for k, v in leaves.items() if k[0] == 16}
    require(version, 'Windows version resource missing')
    key, xml = manifest[0]
    return dict(machine=hex(machine), optional_magic='0x20b', subsystem=expected_subsystem, subsystem_name={2:'Windows GUI',3:'Windows console'}[expected_subsystem],
                manifest_resource_key=list(key), manifest_sha256=hashlib.sha256(xml).hexdigest().upper(),
                manifest_bytes=len(xml), version_resources=version)
