"""Bounded, read-only PE dependency inventory for a retained release executable."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import struct


def inspect(path):
    data = path.read_bytes()
    if len(data) > 256 * 1024 * 1024:
        raise ValueError('PE exceeds inspection bound')

    def unpack(fmt, offset):
        size = struct.calcsize(fmt)
        if offset < 0 or offset + size > len(data):
            raise ValueError('PE field outside file')
        return struct.unpack_from(fmt, data, offset)

    if data[:2] != b'MZ':
        raise ValueError('missing DOS header')
    pe, = unpack('<I', 0x3c)
    if data[pe:pe + 4] != b'PE\0\0':
        raise ValueError('missing PE signature')
    machine, count, _, _, _, opt_size, characteristics = unpack('<HHIIIHH', pe + 4)
    opt = pe + 24
    magic, = unpack('<H', opt)
    if machine != 0x8664 or magic != 0x20b or count > 96 or opt_size < 240:
        raise ValueError('expected bounded AMD64 PE32+ executable')
    image_base, = unpack('<Q', opt + 24)
    headers, = unpack('<I', opt + 60)
    directories, = unpack('<I', opt + 108)
    subsystem, = unpack('<H', opt + 68)
    sections = []
    for index in range(count):
        at = opt + opt_size + index * 40
        size, address, raw_size, raw = unpack('<IIII', at + 8)
        sections.append((address, max(size, raw_size), raw, raw_size))

    def offset(rva):
        if 0 <= rva < headers and rva < len(data):
            return rva
        for address, size, raw, raw_size in sections:
            if address <= rva < address + size:
                delta = rva - address
                if delta >= raw_size or raw + delta >= len(data):
                    raise ValueError('RVA has no file-backed data')
                return raw + delta
        raise ValueError('unmapped RVA')

    def name(rva):
        begin = offset(rva)
        end = data.find(b'\0', begin, min(begin + 260, len(data)))
        if end < 0:
            raise ValueError('unterminated import name')
        return data[begin:end].decode('ascii')

    def imports(index, width, delay=False):
        if index >= directories:
            return []
        rva, size = unpack('<II', opt + 112 + index * 8)
        if not rva:
            return []
        if size > 1024 * 1024:
            raise ValueError('import directory exceeds bound')
        result = []
        for n in range(min(size // width, 4096)):
            words = unpack('<' + 'I' * (width // 4), offset(rva + n * width))
            if not any(words):
                return sorted(set(result), key=str.lower)
            name_rva = words[1] if delay else words[3]
            if delay and not words[0] & 1:
                name_rva -= image_base
            result.append(name(name_rva))
        raise ValueError('missing bounded import terminator')

    ordinary = imports(1, 20)
    delayed = imports(13, 32, True)
    system = Path(os.environ.get('SystemRoot', r'C:\Windows')) / 'System32'
    deps = []
    for dll in sorted(set(ordinary + delayed), key=str.lower):
        contract = dll.lower().startswith(('api-ms-', 'ext-ms-'))
        system_file = system / dll
        deps.append(dict(name=dll, api_set_contract=contract,
                         current_host_system_file_exists=system_file.is_file(),
                         current_host_system_path=str(system_file) if system_file.is_file() else None))
    return dict(status='inspected_PE_dependencies_only', executable=str(path),
                sha256=hashlib.sha256(data).hexdigest().upper(), size_bytes=len(data),
                machine='AMD64', optional_header='PE32+', subsystem=subsystem,
                executable_image=bool(characteristics & 2), ordinary_imports=ordinary,
                delayed_imports=delayed, dependencies=deps,
                runtime_dynamic_loads_verified=False, clean_machine_verified=False,
                portable_package_accepted=False, full_game_complete=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('executable', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    report = inspect(args.executable.resolve(strict=True))
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({k: report[k] for k in ['status', 'sha256', 'machine', 'ordinary_imports', 'delayed_imports']}, ensure_ascii=False))
