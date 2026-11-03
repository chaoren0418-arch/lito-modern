"""Link the latest skill normal entry and full library harness; run nothing."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, json, os, re, subprocess, time, tomllib, traceback
from native_E149_recovery_contract import R, P, SOURCE, FORMAL, read, sha, encoded, owned, require, check_pins
from native_E149_owned_process import identity, kernel

V = R / '驗證紀錄/current'
T = R / '維護工具'


def build():
    w = read(R / '工作佇列.json')
    require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'], 'another workload is active')
    require(w['current_user_requested_goal']['test_execution_deferred_by_user'], 'development-only phase expected')
    ref = w['current_E149_shadow107_development']
    require(sha(ref['qualification']) == ref['qualification_sha256'], 'latest skill qualification changed')
    q = read(ref['qualification']); check_pins(q['source_pins'])
    require(q['compiler_exit_codes'] == [0] * 16 and q['tests_executed'] == 0, 'latest metadata scope changed')
    old_path = Path(q['compile_run']) / 'cohort.json'
    require(sha(old_path) == q['compile_cohort_sha256'], 'latest metadata cohort changed')
    old = read(old_path); check_pins(old['input_pins']); check_pins(old['normal_product_pins'])
    cp = Path(old['candidate']); require(sha(cp) == old['candidate_sha256'], 'latest candidate changed')
    candidate = read(cp); inputs = read(old['base_compiled_inputs'])
    require(len(inputs) == 1049, 'normal input count differs')
    binding_ref = w['current_E149_fixture_binding']
    require(binding_ref == q['current_fixture_binding_unchanged'] and sha(binding_ref['qualification']) == binding_ref['qualification_sha256'], 'current fixture differs')
    binding = read(binding_ref['qualification'])
    files = dict(candidate['effective_overlay_files']); paths = dict(candidate['effective_overlay_paths'])
    require(len(files) == 21 and files.keys() == paths.keys(), 'latest overlay differs')
    check_pins({paths[n]: h for n, h in files.items()})
    overlap_record = {n: dict(previous_path=binding['current_effective_overlay_paths'][n], previous_sha256=binding['current_effective_overlay_files'][n], replacement_path=paths[n], replacement_sha256=files[n])
                      for n in binding['current_effective_overlay_files'] if binding['current_effective_overlay_files'][n] != files[n]}
    runtime = sorted(n for n in files if '/src/' in n and not n.endswith('_tests.rs'))
    java = sorted(n for n in files if n.endswith('.java')); qa = sorted(set(files) - set(runtime) - set(java))
    require((len(runtime), len(qa), len(java)) == (8, 10, 3), 'latest source roles differ')
    for n in java: require(paths[n] == binding['current_effective_overlay_paths'][n] and files[n] == binding['current_effective_overlay_files'][n], 'latest Java context differs')
    paired = w['current_native_item_spells_development']['paired_runtime_sources']
    modern = Path(old['generated_mod']).read_text('utf-8-sig'); graph = list(re.finditer(r'#\[path = r#"([^"]+)"#\]', modern))
    require(len(graph) == 362, 'module graph changed'); runtime_graph = {}
    for n in runtime:
        pair = paired.get(n)
        matches = [m for m in graph if (Path(m[1]) == Path(pair['path']) if pair else Path(m[1]).name == Path(n).name)]
        require(len(matches) == 1, 'runtime module absent/ambiguous: ' + n); compiled = Path(matches[0][1])
        if pair:
            require(sha(compiled) == pair['sha256'] and paths[n] == pair['source'] and files[n] == pair['source_sha256'], 'paired runtime differs')
            restored = compiled.read_text('utf-8-sig')
            for row in pair['rewrites']:
                require(restored.count(row['compiled_attribute']) == 1, 'nested path differs'); restored = restored.replace(row['compiled_attribute'], row['original_attribute'])
            require(restored == Path(paths[n]).read_text('utf-8-sig'), 'runtime differs beyond nested paths')
        elif n.endswith('/world_raster.rs'):
            fixture = P / 'tests/fixtures/native-floor-gate-original.json'; original = Path(paths[n]).read_text('utf-8-sig')
            needle = 'include_bytes!("../../tests/fixtures/native-floor-gate-original.json")'
            require(original.count(needle) == 1 and original.replace(needle, 'include_bytes!(r#"' + str(fixture) + '"#)') == compiled.read_text('utf-8-sig'), 'raster differs beyond original fixture path')
            require(inputs[fixture.relative_to(R).as_posix()] == sha(fixture), 'raster fixture differs')
        else: require(sha(compiled) == files[n], 'runtime graph does not contain latest source: ' + n)
        runtime_graph[n] = dict(path=str(compiled), sha256=sha(compiled), raw_source=paths[n], raw_sha256=files[n])
    configuration = tomllib.loads((P / 'Cargo.toml').read_text('utf-8-sig'))
    require(configuration['package']['default-run'] == 'LiToModern', 'normal entry differs')
    require(tuple(configuration['profile']['release'][k] for k in ['opt-level', 'lto', 'codegen-units', 'strip']) ==
            ('z', 'thin', 16, 'symbols'), 'release settings differ')
    pins = dict(old['input_pins']) | {str(R / n): h for n, h in inputs.items()}
    pins.update({paths[n]: h for n, h in files.items()})
    for p in [Path(__file__).resolve(), T / 'native_E149_recovery_contract.py', T / 'native_E149_owned_process.py',
              Path(ref['qualification']), old_path, cp, Path(binding_ref['qualification']),
              P / 'src/main.rs', P / 'windows/app.manifest', P / 'build.rs', P / 'Cargo.toml']:
        pins[str(p)] = sha(p)
    external = dict(old['external_toolchain_dependency_pins'])
    rustc = Path(old['compiler_runs'][0]['command'][0])
    toolchain = rustc.parent.parent
    external[str(rustc)] = sha(rustc)
    for folder, patterns in [(toolchain / 'lib/rustlib/x86_64-pc-windows-msvc/lib', ['*.rlib']),
                             (toolchain / 'lib/rustlib/x86_64-pc-windows-msvc/codegen-backends', ['*.dll']),
                             (toolchain / 'bin', ['rustc_driver*.dll', 'LLVM*.dll'])]:
        for pattern in patterns:
            for p in folder.glob(pattern):
                external[str(p)] = sha(p)
    native = {}
    resource = {}
    for profile in ['Debug', 'Release']:
        log = Path(w['current_native_installed_resource_profile_normal_qualification']['run']) / profile / 'test-build/stdout.log'
        pins[str(log)] = sha(log)
        messages = [json.loads(s) for s in log.read_text('utf-8-sig').splitlines() if s.startswith('{')]
        rows = [m for m in messages if m.get('reason') == 'build-script-executed']
        native[profile] = sorted({s for m in rows for s in m.get('linked_paths', [])})
        own = [m for m in rows if 'lito-modern@' in m['package_id']]
        require(len(own) == 1 and own[0]['linked_libs'] == ['dylib=resource'], 'normal Windows resource contract differs')
        resource[profile] = own[0]['linked_libs']
        for flag in native[profile]:
            folder = Path(flag.split('=', 1)[1])
            require(folder.is_relative_to(R) or folder.is_relative_to(Path(os.environ['USERPROFILE']) / '.cargo/registry'),
                    'unapproved native link path')
            for p in folder.iterdir():
                if p.is_file() and p.suffix.lower() in ['.lib', '.a', '.rc']:
                    (pins if p.is_relative_to(R) else external)[str(p)] = sha(p)
        for m in messages:
            if m.get('reason') == 'compiler-artifact':
                for name in m.get('filenames', []):
                    p = Path(name)
                    if p.is_file() and p.suffix.lower() in ['.rlib', '.rmeta', '.dll', '.lib']:
                        owned(p, P / 'target/current'); pins[str(p)] = sha(p)

    def guard():
        check_pins(pins)
        require(all(sha(p) == h for p, h in external.items()), 'external compiler input changed')
        check_pins(old['normal_product_pins'])
        require(sha(w['current_formal_reference_identity']['formal']) == FORMAL, 'formal JAR changed')

    guard()
    stamp = datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
    out = owned(V / 'native-E149-skill-library-linked-build' / stamp); out.mkdir(parents=True)
    products = owned(P / 'target/current/verification/native-E149-skill-library-images' / stamp); products.mkdir(parents=True)
    (products / 'scratch').mkdir()
    (out / 'builder.py').write_bytes(Path(__file__).read_bytes())
    overlay = dict(schema='E149-unselected-skill-library-overlay-v1', base_source_manifest_sha256=SOURCE,
                   formal_target_sha256=FORMAL, candidate_files=files, candidate_paths=paths,
                   overlap_resolution=overlap_record, runtime_files=runtime, QA_files=qa, Java_files=java,
                   runtime_compiled_graph=runtime_graph, original_fixture_binding=binding_ref,
                   original_registry=w['current_isolated_development']['candidates'],
                   canonical_registry_selected=False, canonical_fixture_selected=False, production_applied=False,
                   qualification=ref, tests_executed=0, whole_goal_complete=False)
    (out / 'overlay.json').write_bytes(encoded(overlay)); pins[str(out / 'overlay.json')] = sha(out / 'overlay.json')
    state = dict(schema='E149-skill-library-linked-build-v1', status='compiling', source_manifest_sha256=SOURCE,
                 formal_target_sha256=FORMAL, overlay=str(out / 'overlay.json'), overlay_sha256=sha(out / 'overlay.json'),
                 original_metadata_cohort=str(old_path), original_metadata_cohort_sha256=sha(old_path),
                 generated_root=old['generated_root'], generated_mod=old['generated_mod'],
                 base_compiled_inputs=old['base_compiled_inputs'], input_pins=pins,
                 external_toolchain_dependency_pins=external, normal_product_pins=old['normal_product_pins'],
                 native_link_search_paths=native, Windows_resource_libraries=resource,
                 pipeline=identity(kernel().GetCurrentProcess(), os.getpid()), compiler_runs=[], owned_process=None,
                 products=str(products), tests_executed=0, outputs_executed=False, GUI_inputs=0,
                 database_started=False, server_started=False, candidate_applied=False, whole_goal_complete=False)

    def save():
        temp = out / 'cohort.json.writing'; temp.write_bytes(encoded(state)); os.replace(temp, out / 'cohort.json')

    save(); print(json.dumps(dict(run=str(out), mode='normal_entry_and_full_library_harness_link_only'), ensure_ascii=False), flush=True)
    env = {k: v for k, v in os.environ.items() if not k.startswith(('LITO_', 'RUST_')) and
           k not in ['RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER']}
    env.update(CARGO_MANIFEST_DIR=str(P), CARGO_PKG_VERSION=configuration['package']['version'],
               CARGO_BUILD_JOBS='1', RAYON_NUM_THREADS='1', TEMP=str(products / 'scratch'), TMP=str(products / 'scratch'))
    try:
        for profile in ['Debug', 'Release']:
            library = products / ('liblito_modern-' + profile + '.rlib')
            template = next(r['command'] for r in old['compiler_runs'] if r['profile'] == profile and r['kind'] == 'library')
            require('--test' not in template and '--cfg' not in template and '--emit=metadata' in template and
                    '--crate-type=rlib' in template, 'metadata library command differs')
            for kind in ['library', 'normal-entry', 'library-test']:
                guard(); command = list(template)
                output = library if kind == 'library' else products / (('LiToModern-' if kind == 'normal-entry' else 'full-library-') + profile + '-candidate.exe')
                command[command.index('--emit=metadata')] = '--emit=link'
                command[command.index('-o') + 1] = str(output)
                if kind == 'normal-entry':
                    command.remove('--crate-type=rlib')
                    command[command.index(old['generated_root'])] = str(P / 'src/main.rs')
                    command[command.index('--crate-name') + 1] = 'LiToModern'
                    command += ['--extern', 'lito_modern=' + str(library)]
                elif kind == 'library-test':
                    command.remove('--crate-type=rlib'); command += ['--test']
                if kind != 'library':
                    for lib in resource[profile]: command += ['-l', lib]
                for flag in native[profile]: command += ['-L', flag]
                folder = out / (profile + '-' + kind); folder.mkdir()
                with (folder / 'stdout.log').open('xb') as stdout, (folder / 'stderr.log').open('xb') as stderr:
                    child = subprocess.Popen(command, cwd=products, env=env, stdout=stdout, stderr=stderr,
                                             creationflags=subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS)
                    row = dict(profile=profile, kind=kind, command=command, output=str(output), exit_code=None,
                               original_process=identity(int(child._handle), child.pid))
                    state['compiler_runs'].append(row); state['owned_process'] = row['original_process']; save()
                    deadline = time.monotonic() + 1200
                    try:
                        while True:
                            try: child.wait(timeout=20); break
                            except subprocess.TimeoutExpired:
                                require(time.monotonic() < deadline, 'original compiler exceeded 1200 seconds')
                                print(json.dumps(dict(profile=profile, kind=kind, pid=child.pid, compiler_live=True)), flush=True)
                    except BaseException:
                        if child.poll() is None:
                            require(identity(int(child._handle), child.pid) == row['original_process'], 'compiler handle changed')
                            child.terminate(); child.wait(timeout=20); row['original_handle_cleanup'] = True
                        raise
                    finally:
                        row.update(exit_code=child.poll(), finished_utc=datetime.now(timezone.utc).isoformat(),
                                   logs={str(folder / n): sha(folder / n) for n in ['stdout.log', 'stderr.log']})
                        state['owned_process'] = None; save()
                require(child.returncode == 0, 'original compiler/linker failed; diagnostics preserved')
                require(output.is_file() and output.stat().st_size > 0, 'linked output missing')
                row.update(output_sha256=sha(output), output_bytes=output.stat().st_size)
                pins[str(output)] = sha(output); save(); guard()
        state['status'] = 'six_compilers_exited_no_output_executed'
    except BaseException:
        state['status'] = 'original_build_failure_preserved'
        (out / 'original-failure.txt').write_text(traceback.format_exc(), encoding='utf-8')
        raise
    finally:
        state['finished_utc'] = datetime.now(timezone.utc).isoformat(); save()
    print(json.dumps(dict(run=str(out), exit_codes=[r['exit_code'] for r in state['compiler_runs']],
                          tests_executed=0, outputs_executed=False), ensure_ascii=False), flush=True)


if __name__ == '__main__':
    argparse.ArgumentParser(description=__doc__).parse_args()
    build()
