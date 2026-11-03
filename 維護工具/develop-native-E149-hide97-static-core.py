"""Readonly native suffix review and standalone metadata-only lifecycle draft."""
from pathlib import Path
from datetime import datetime, timezone
import json, os, re, subprocess
from native_E149_recovery_contract import R, P, read, sha, encoded, require, check_pins
from native_E149_owned_process import identity, kernel

T = R / '維護工具'
V = R / '驗證紀錄/current'

def main():
    w = read(R / '工作佇列.json')
    require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and
            w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and
            not w['active_workloads'] and not w['active_workload_sessions'] and
            not w['source_and_corresponding_QA_frozen'], 'development-only phase differs')
    ref = w['current_E149_hide97_request_venom98_finite_development']
    require(sha(ref['qualification']) == ref['qualification_sha256'], 'base qualification differs')
    q = read(ref['qualification'])
    check_pins(q['normal_product_pins'])
    old = read(Path(q['compile_run']) / 'cohort.json')
    rustc = Path(old['compiler_runs'][0]['command'][0])
    require(sha(rustc) == old['compiler_runs'][0]['original_process']['executable_sha256'], 'rustc differs')
    prior = V / 'native-E149-hide97-world-order/20261024-192511-648815/world-order.json'
    require(sha(prior) == '1343A1C373470AB675B1799B4FC97D7E943BFA4B4C0C4BD39AF5AD1EEABA91D6', 'world source differs')
    j = read(prior)
    check_pins(j['source_pins'])
    require(all(sha(p) == h for p, h in j['external_readonly_pins'].items()), 'native source differs')
    asm = next(Path(p) for p in j['external_readonly_pins'] if p.endswith('.asm'))
    s = asm.read_text('utf-8-sig')
    stamp = datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
    out = V / 'native-E149-hide97-static-core-development' / stamp
    out.mkdir(parents=True)
    source = T / 'templates/native_hide97_static_lifecycle_draft.rs'
    # Compile the immutable snapshot, not a subsequently editable template.
    snapshot = out / source.name
    snapshot.write_bytes(source.read_bytes())
    functions = {}
    for name in ['4E2150', '4857B0', '4ADDA0']:
        a = s.index('sub_' + name + ' proc near')
        b = s.index('sub_' + name + ' endp', a) + len('sub_' + name + ' endp')
        p = out / (name + '.asm.txt')
        p.write_text(s[a:b] + '\n', encoding='utf-8', newline='\n')
        functions[name] = dict(path=str(p), sha256=sha(p))
    actor = (out / '4E2150.asm.txt').read_text('utf-8')
    compact = re.sub(r';[^\n]*', '', actor)
    compact = '\n'.join(line.strip() for line in compact.splitlines() if line.strip())
    checks = [
        'mov     byte_834EB3, 0',
        'loc_4E3207:\nmov     ecx, dword_834EE4\nmov     al, [ecx+13h]\ntest    al, al\njnz     loc_4E3429',
        'mov     al, byte_834F1D\ntest    al, al\njz      short loc_4E325C',
        'loc_4E325C:\nmov     al, byte_834EB2\ntest    al, al\njz      loc_4E339B',
        'loc_4E339B:\nmov     al, byte_834EB3\ntest    al, al\njz      short loc_4E3420\nmov     al, byte_641498\ntest    al, al\njnz     short loc_4E33B2\ncall    sub_488860',
        'loc_4E3429:\nmov     al, byte_641498\ntest    al, al\njnz     short loc_4E3437\ncall    sub_488860',
        'loc_4E307E:\nmov     ecx, [ebp+var_24]\npush    0\nmov     edx, offset sub_4E2150\nmov     byte_834EB3, 1\ncall    sub_4D65F0',
    ]
    require(all(v in compact for v in checks), 'reviewed local branch suffix differs')
    require(compact.count('mov     byte_834EB3, 1') == 1 and
            actor.count('call    sub_488860') == 2, 'store/call count differs')
    pump = (out / '4857B0.asm.txt').read_text('utf-8')
    require('call    sub_4D6570\n\ncall    sub_488860' in pump and
            'cmp     edi, 1F4h' in pump, 'nested drain/draw source differs')
    packet = (out / '4ADDA0.asm.txt').read_text('utf-8')
    require('mov     edi, 0A9h' in packet and packet.count('call    sub_4857B0') == 1,
            'S37 special draw source differs')
    review = dict(
        schema='E149-hide97-local-redraw-suffix-and-static-core-v1',
        base_qualification=ref,
        original_world_order=str(prior),
        original_world_order_sha256=sha(prior),
        full_asm_functions=functions,
        predicate_scope='Local suffix reached at 4E3207, not all prior CFG paths or live globals',
        redraw_4E33AD=dict(own_byte13=0, byte834F1D=0, byte834EB2=0, byte834EB3='nonzero', byte641498=0),
        redraw_4E3432=dict(own_byte13='nonzero', byte641498=0),
        byte834EB3_reset_at_entry=True,
        sole_direct_set_one_in_function='4E307E: enqueue 4E2150 at var24 delay; upstream path still conditional',
        special_S37_169_own_actor_can_call_4857B0=True,
        special_S37_2941_does_not_directly_take_169_branch=True,
        function_4857B0_drains_and_draws_in_relative_500ms_loop=True,
        nested_global_drains_and_transitive_message_reentrancy_qualified=False,
        exact_original_clock_initialization_qualified=False,
        draft_core=dict(source=str(snapshot), template=str(source),
                        borrowed_fixed_drain=True, equal_deadline_stable_order=True,
                        pop_before_callback=True, zero_delay_enqueued=True,
                        retained_admission_tombstones=True, no_draw_advances_time=True,
                        standalone_conditional_only=True, native_clock_wrap_refused=True,
                        queue_capacity_explicit=True, full_global_queue_bound=False),
        hide97_2941_visual_and_retirement_integrated=False,
        assets_decoded=False, pixels_qualified=False, tests_written=0,
        tests_executed=0, desktop_inputs=0, PEs_linked=False,
        whole_goal_complete=False,
    )
    (out / 'developer.py').write_bytes(Path(__file__).read_bytes())
    pins = {str(p): sha(p) for p in out.iterdir() if p.is_file()}
    pins[str(source)] = sha(source)
    pins[str(prior)] = sha(prior)
    state = dict(review=review, input_pins=pins, normal_product_pins=q['normal_product_pins'],
                 external_toolchain_pins={str(rustc): sha(rustc)},
                 pipeline=identity(kernel().GetCurrentProcess(), os.getpid()),
                 compiler_runs=[], status='standalone_metadata_compiling',
                 tests_executed=0, outputs_executed=False, PEs_linked=False)
    ep = out / 'evidence.json'
    def save(): ep.write_bytes(encoded(state))
    save()
    products = P / 'target/current/verification/native-E149-hide97-static-core' / stamp
    products.mkdir(parents=True)
    env = dict(os.environ)
    env['TMP'] = env['TEMP'] = str(products)
    for profile, opt in [('Debug', '0'), ('Release', 'z')]:
        product = products / (profile + '.rmeta')
        command = [str(rustc), str(snapshot), '--edition=2021', '--crate-name=hide97_static_core_draft',
                   '--crate-type=lib', '--target=x86_64-pc-windows-msvc', '--emit=metadata',
                   '-Copt-level=' + opt, '-o', str(product)]
        check_pins(pins)
        with (out / (profile + '.stdout')).open('wb') as stdout, (out / (profile + '.stderr')).open('wb') as stderr:
            child = subprocess.Popen(command, cwd=products, env=env, stdout=stdout, stderr=stderr,
                                     creationflags=subprocess.CREATE_NO_WINDOW | subprocess.BELOW_NORMAL_PRIORITY_CLASS)
            row = dict(profile=profile, command=command, original_process=identity(int(child._handle), child.pid),
                       exit_code=None, product=str(product))
            state['compiler_runs'].append(row)
            save()
            # Preserve the original handle; no deadline/restart/termination policy.
            row['exit_code'] = child.wait()
            row['original_handle_signaled'] = kernel().WaitForSingleObject(int(child._handle), 0) == 0
        row['stdout_sha256'] = sha(out / (profile + '.stdout'))
        row['stderr_sha256'] = sha(out / (profile + '.stderr'))
        save()
        require(row['exit_code'] == 0 and row['original_handle_signaled'] and product.is_file(), 'metadata compiler failed')
        row['product_sha256'] = sha(product)
        save()
    check_pins(pins)
    check_pins(q['normal_product_pins'])
    state['status'] = 'two_standalone_metadata_complete_no_runtime_integration'
    save()
    print(json.dumps(dict(evidence=str(ep), sha256=sha(ep), compiler_exit_codes=[r['exit_code'] for r in state['compiler_runs']],
                          tests_executed=0, integrated_into_runtime=False), ensure_ascii=False))

if __name__ == '__main__': main()
