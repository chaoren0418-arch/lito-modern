"""Plan an owned E149 static-resource DB; serve is explicitly deferred by default.

No process is started on import or by `plan`. Only future `serve --execute`
may initialize a fresh datadir. This is fixture infrastructure, not acceptance.
"""
from pathlib import Path
from datetime import datetime, timezone
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, re, secrets, socket, stat
import subprocess, sys, time, traceback

R = Path(__file__).resolve().parents[1]
P = R / 'LiTo381-Modern'
CASES = P / 'target/current/verification/native-E149-owned-mysql'
FORMAL = 'E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498'
SQL = 'C1E29B0479FE40476AD53B4E52498451C3A1E469A27483EABE67FA880EBCA7DD'
DUMP = 'D4111D8AEFF50EA5D9D36630C7381132812F7F727E8432FFAF035B89C9CBB075'
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
utc = lambda: datetime.now(timezone.utc).isoformat()


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def within(path, root=R):
    """Reject reparse ancestors before resolving; allow a not-yet-created leaf."""
    path = Path(os.path.abspath(path))
    require(path.is_relative_to(root), 'path outside owned root')
    for part in [path, *path.parents]:
        if part.exists():
            require(not part.lstat().st_file_attributes & stat.FILE_ATTRIBUTE_REPARSE_POINT,
                    'reparse path forbidden: ' + str(part))
    require(path.resolve().is_relative_to(root.resolve()), 'resolved path escaped root')
    return path


def write_json(path, value):
    path = within(path)
    blob = (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
    temp = path.with_name(path.name + '.writing')
    with temp.open('xb') as file:
        file.write(blob)
    os.replace(temp, path)


def properties(path, values):
    # Java uses a UTF-8 Reader. Escape the Properties metacharacters explicitly.
    def esc(value):
        return str(value).replace('\\', '\\\\').replace('\n', '\\n').replace('\r', '\\r').replace('=', '\\=').replace(':', '\\:')
    with within(path).open('x', encoding='utf-8', newline='\n') as file:
        file.write(''.join(esc(k) + '=' + esc(v) + '\n' for k, v in values.items()))


def authorize_execution():
    w = read(R / '工作佇列.json')
    goal = w['current_user_requested_goal']
    require(not goal.get('test_execution_deferred_by_user', True),
            'User deferred tests: serve is disabled; plan/static development only')
    require(not w['active_workloads'] and not w['source_and_corresponding_QA_frozen'],
            'another workload or source freeze is active')
    ref = w['current_formal_reference_identity']
    require(ref['formal_sha256'] == sha(ref['formal']) == FORMAL, 'formal target changed')


def validate_plan(path):
    path = within(path, CASES)
    p = read(path)
    root = within(p['case_root'], CASES)
    require(path == root / 'plan.json', 'plan location mismatch')
    require(p['status'] == 'development_plan_only' and re.fullmatch('[0-9a-f]{12}', p['nonce']), 'invalid plan')
    require(p['launcher_sha256'] == sha(__file__), 'launcher differs from planned source')
    require(30 <= p['lease_seconds'] <= 1800, 'invalid explicit lease')
    require(p['schema'] == 'lito_rebuild_e149_' + p['nonce'], 'schema mismatch')
    require(p['reader_user'] == 'lg06_' + p['nonce'][:8], 'reader mismatch')
    for name, digest in p['input_pins'].items():
        require(sha(within(name)) == digest, 'input changed: ' + name)
    bundle = read(p['bundle_manifest'])
    base = within(bundle['bundle_root'], P / 'target/current/owned-database-toolchains')
    for name, digest in bundle['files'].items():
        require(sha(within(base / name, base)) == digest, 'private toolchain changed: ' + name)
    require(bundle['formal_target_sha256'] == FORMAL and bundle['machine'] == '0x8664', 'toolchain identity differs')
    m = read(p['static_manifest'])
    require(m['SQL_sha256'] == SQL and m['source_dump_sha256'] == DUMP, 'static source differs')
    require(m['table_count'] == 19 and m['rows'] == 6277 and m['statement_count'] == 6296, 'static counts differ')
    require(sha(within(m['SQL'])) == SQL, 'static SQL changed')
    return p, root, base, bundle, m


def plan(bundle_path, static_path, lease):
    bundle_path, static_path = within(bundle_path), within(static_path)
    bundle, m = read(bundle_path), read(static_path)
    nonce = secrets.token_hex(6)
    root = within(CASES / ('case-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f') + '-' + nonce))
    root.mkdir(parents=True, exist_ok=False)
    pins = {str(p): sha(p) for p in [bundle_path, static_path, within(m['SQL']), within(bundle['JDBC']['cached'])]}
    p = dict(status='development_plan_only', created_utc=utc(), case_root=str(root), nonce=nonce,
             schema='lito_rebuild_e149_' + nonce, reader_user='lg06_' + nonce[:8], lease_seconds=lease,
             launcher=str(Path(__file__).resolve()), launcher_sha256=sha(__file__),
             bundle_manifest=str(bundle_path), static_manifest=str(static_path), input_pins=pins,
             database_started=False, SQL_executed=False, tests_executed=0, desktop_inputs=0,
             future_execution_requires_user_resumption=True)
    write_json(root / 'plan.json', p)
    validate_plan(root / 'plan.json')
    print(json.dumps(dict(plan=str(root / 'plan.json'), database_started=False, tests_executed=0), ensure_ascii=False))


def ps_json(code):
    command = ['pwsh', '-NoProfile', '-NonInteractive', '-Command',
               "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';" + code]
    out = subprocess.run(command, capture_output=True, timeout=8, creationflags=subprocess.CREATE_NO_WINDOW)
    require(out.returncode == 0, 'OS ownership observation failed')
    return json.loads(out.stdout.decode('utf-8-sig')) if out.stdout.strip() else None


def kernel():
    k = ctypes.WinDLL('kernel32', use_last_error=True)
    k.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    k.GetProcessTimes.restype = wintypes.BOOL
    k.QueryFullProcessImageNameW.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD)]
    k.QueryFullProcessImageNameW.restype = wintypes.BOOL
    k.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    k.OpenProcess.restype = wintypes.HANDLE
    k.CloseHandle.argtypes = [wintypes.HANDLE]
    return k


def handle_identity(handle, pid):
    k = kernel()
    times = [wintypes.FILETIME() for _ in range(4)]
    require(k.GetProcessTimes(handle, *[ctypes.byref(t) for t in times]), 'process birth unavailable')
    ticks = (times[0].dwHighDateTime << 32) | times[0].dwLowDateTime
    size = wintypes.DWORD(32768)
    text = ctypes.create_unicode_buffer(size.value)
    require(k.QueryFullProcessImageNameW(handle, 0, text, ctypes.byref(size)), 'process image unavailable')
    return dict(pid=pid, creation_filetime_ticks=ticks,
                start_epoch_millis=(ticks - 116444736000000000) // 10000,
                executable=str(Path(text.value).resolve()), executable_sha256=sha(text.value))


def identity_for_pid(pid):
    k = kernel()
    h = k.OpenProcess(0x1000, False, pid)  # QUERY_LIMITED_INFORMATION only
    require(h, 'recorded process is not queryable')
    try:
        return handle_identity(h, pid)
    finally:
        k.CloseHandle(h)


def command_args(command):
    api = ctypes.WinDLL('shell32', use_last_error=True)
    api.CommandLineToArgvW.argtypes = [wintypes.LPCWSTR, ctypes.POINTER(ctypes.c_int)]
    api.CommandLineToArgvW.restype = ctypes.POINTER(wintypes.LPWSTR)
    count = ctypes.c_int()
    ptr = api.CommandLineToArgvW(command, ctypes.byref(count))
    require(ptr, 'command line unavailable')
    k = kernel()
    k.LocalFree.argtypes = [ctypes.c_void_p]
    k.LocalFree.restype = ctypes.c_void_p
    try:
        return [ptr[i] for i in range(count.value)]
    finally:
        k.LocalFree(ctypes.cast(ptr, ctypes.c_void_p))


def verify_live(owner, require_listener=True):
    actual = identity_for_pid(owner['pid'])
    require(actual == owner['identity'], 'owned process birth/image changed')
    pid, port = int(owner['pid']), int(owner['port'])
    row = ps_json(f'Get-CimInstance Win32_Process -Filter "ProcessId={pid}"|Select-Object CommandLine|ConvertTo-Json -Compress')
    require(row and command_args(row['CommandLine']) == owner['command'], 'owned command/datadir/bind differs')
    if require_listener:
        rows = ps_json(f'@(Get-NetTCPConnection -State Listen -LocalPort {port} -ErrorAction SilentlyContinue|Select-Object LocalAddress,OwningProcess)|ConvertTo-Json -Compress')
        rows = [rows] if isinstance(rows, dict) else rows
        require(rows and all(x['LocalAddress'] == '127.0.0.1' and x['OwningProcess'] == pid for x in rows),
                'listener is absent or belongs to a different process')
    require(identity_for_pid(pid) == actual, 'owned identity changed during observation')
    return actual


class Runner:
    def __init__(self, root, bundle, base, history_name='process-history.json'):
        self.root, self.bundle, self.base = root, bundle, base
        self.history_path = within(root / history_name, root)
        self.history = []

    def spawn(self, name, args, stdin=None):
        binary = within(args[0], self.base)
        relative = binary.relative_to(self.base).as_posix()
        require(sha(binary) == self.bundle['files'][relative], 'executable changed')
        require(not (self.root / (name + '.stdout.log')).exists(), 'command name reused')
        out = (self.root / (name + '.stdout.log')).open('xb')
        err = (self.root / (name + '.stderr.log')).open('xb')
        try:
            environment = {k: v for k, v in os.environ.items() if not k.upper().startswith('MYSQL')}
            environment.update(TEMP=str(self.root / 'tmp'), TMP=str(self.root / 'tmp'))
            proc = subprocess.Popen(args, cwd=self.root, stdin=stdin or subprocess.DEVNULL,
                                    stdout=out, stderr=err, env=environment, creationflags=subprocess.CREATE_NO_WINDOW)
            record = dict(name=name, command=args, identity=handle_identity(int(proc._handle), proc.pid), started_utc=utc())
            self.history.append(record)
            write_json(self.history_path, self.history)
            return proc, record
        except BaseException:
            # A created child may not leak merely because identity/receipt failed.
            if 'proc' in locals() and proc.poll() is None:
                proc.terminate()
                proc.wait(timeout=10)
            raise
        finally:
            out.close()
            err.close()

    def finish(self, proc, record, timeout):
        try:
            code = proc.wait(timeout=timeout)
        except BaseException:
            if proc.poll() is None:
                proc.terminate()  # exact original Popen handle
                proc.wait(timeout=10)
            raise
        finally:
            record.update(exit_code=proc.poll(), finished_utc=utc())
            write_json(self.history_path, self.history)
        require(code == 0, record['name'] + ' failed; inspect preserved original logs')


def serve(plan_path):
    authorize_execution()  # before creating datadir, binding port, or starting child
    p, root, base, bundle, m = validate_plan(plan_path)
    require(not (root / 'state.json').exists() and not (root / 'data').exists(), 'case already attempted; preserve it and make a fresh plan')
    state = dict(status='initializing', plan_sha256=sha(plan_path), nonce=p['nonce'],
                 pipeline=identity_for_pid(os.getpid()), started_utc=utc(), database_started=False,
                 SQL_executed=False, tests_executed=0, static_SQL_access=False, player_SQL_access=False,
                 static_resource_reader_invoked=False)
    write_json(root / 'state.json', state)
    runner = Runner(root, bundle, base)
    daemon = daemon_record = owner = None
    reservation = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    primary = None
    try:
        reservation.setsockopt(socket.SOL_SOCKET, socket.SO_EXCLUSIVEADDRUSE, 1)
        reservation.bind(('127.0.0.1', 0))
        port = reservation.getsockname()[1]
        require(49152 <= port <= 65535, 'OS did not allocate a private ephemeral port')
        datadir = within(root / 'data', root)
        datadir.mkdir()
        (datadir / 'mysql').mkdir()
        (root / 'tmp').mkdir()
        root_password, reader_password = secrets.token_hex(24), secrets.token_hex(24)
        bootstrap = root / 'bootstrap.sql'
        with bootstrap.open('xb') as file:
            file.write(b'CREATE DATABASE IF NOT EXISTS mysql;\nUSE mysql;\n')
            for name in ['mysql_system_tables.sql', 'mysql_system_tables_data.sql', 'fill_help_tables.sql']:
                file.write((base / 'share' / name).read_bytes() + b'\n')
            file.write(("DELETE FROM mysql.user WHERE User <> 'root' OR Host NOT IN ('localhost','127.0.0.1');\n"
                        "DELETE FROM mysql.db;\nDELETE FROM mysql.proxies_priv;\n"
                        "UPDATE mysql.user SET Password=PASSWORD('" + root_password + "');\n").encode('ascii'))
        common = [str(base / 'bin/mysqld.exe'), '--no-defaults', '--basedir=' + base.as_posix(),
                  '--datadir=' + datadir.as_posix(), '--tmpdir=' + (root / 'tmp').as_posix(),
                  '--character-set-server=utf8mb4', '--collation-server=utf8mb4_general_ci',
                  '--innodb-buffer-pool-size=32M', '--innodb-log-file-size=8M']
        authorize_execution()
        with bootstrap.open('rb') as file:
            child, record = runner.spawn('bootstrap', common + ['--bootstrap', '--skip-networking'], file)
            state.update(SQL_executed=True)
            runner.finish(child, record, 90)
        state.update(status='starting', bootstrap_exit_code=0)
        write_json(root / 'state.json', state)
        config = root / 'root-client.ini'
        with config.open('x', encoding='ascii', newline='\n') as file:
            file.write('[client]\nuser=root\npassword=' + root_password + '\nprotocol=TCP\nhost=127.0.0.1\nport=' + str(port) + '\ndefault-character-set=utf8mb4\n')
        args = common + ['--console', '--port=' + str(port), '--bind-address=127.0.0.1', '--skip-name-resolve',
                         '--pid-file=' + (root / 'mysqld.pid').as_posix(), '--log-error=' + (root / 'mysqld.error.log').as_posix(),
                         '--max-connections=12', '--event-scheduler=OFF']
        authorize_execution()
        reservation.close()  # only release immediately before starting our daemon
        daemon, daemon_record = runner.spawn('daemon', args)
        owner = dict(pid=daemon.pid, identity=daemon_record['identity'], command=args, port=port, datadir=str(datadir),
                     root_client_sha256=sha(config), plan_sha256=sha(plan_path))
        write_json(root / 'owned-process.json', owner)
        deadline = time.monotonic() + 30
        while True:
            require(daemon.poll() is None, 'owned daemon exited during startup')
            try:
                verify_live(owner)
                break
            except Exception:
                require(time.monotonic() < deadline, 'no verified owned listener before startup deadline')
                time.sleep(.2)
        state.update(status='importing', database_started=True, process=owner)
        write_json(root / 'state.json', state)

        def client(name, sql, database=None, admin=False):
            if not admin:
                authorize_execution()
            verify_live(owner)  # each command, including shutdown, requires actual ownership
            require(sha(within(config, root)) == owner['root_client_sha256'], 'root client configuration changed')
            exe = 'mysqladmin.exe' if admin else 'mysql.exe'
            command = [str(base / 'bin' / exe), '--defaults-file=' + config.as_posix()]
            if admin:
                command += ['shutdown']
            else:
                command += ['--batch', '--raw', '--skip-column-names', '--default-character-set=utf8mb4']
                if database:
                    require(database == p['schema'], 'client schema mismatch')
                    command += ['--database=' + database]
            payload = root / (name + '.sql')
            with payload.open('xb') as file:
                file.write(sql)
            with payload.open('rb') as file:
                child, row = runner.spawn(name, command, file)
                runner.finish(child, row, 90)
            if not admin:
                verify_live(owner)
            return (root / (name + '.stdout.log')).read_bytes().decode('utf-8-sig')

        client('create-schema', ('CREATE DATABASE `' + p['schema'] + '` CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;\n').encode('ascii'))
        original_sql = Path(m['SQL']).read_bytes()
        require(hashlib.sha256(original_sql).hexdigest().upper() == SQL, 'SQL changed before import')
        client('import-static', original_sql, p['schema'])
        state['static_SQL_access'] = True
        rows = client('list-tables', ('SELECT TABLE_NAME FROM information_schema.TABLES WHERE TABLE_SCHEMA=\'' + p['schema'] + "' AND TABLE_TYPE='BASE TABLE' ORDER BY TABLE_NAME;\n").encode('ascii'))
        require(set(rows.splitlines()) == set(m['selected_tables']) and len(rows.splitlines()) == 19, 'actual imported table set differs')
        names = sorted(m['selected_tables'])
        query = '\n'.join('SELECT COUNT(*) FROM `' + n + '`;' for n in names) + '\n'
        counts = [int(x) for x in client('count-static', query.encode('utf-8'), p['schema']).splitlines()]
        require(counts == [m['selected_tables'][n]['rows'] for n in names], 'actual imported row counts differ')
        # Database-level grant names use patterns, even inside backticks.
        # Escape underscores so the reader is limited to this exact schema.
        grant_schema = p['schema'].replace('_', '\\_')
        grant = ("CREATE USER '" + p['reader_user'] + "'@'127.0.0.1' IDENTIFIED BY '" + reader_password + "';\n"
                 "GRANT SELECT ON `" + grant_schema + "`.* TO '" + p['reader_user'] + "'@'127.0.0.1';\n")
        client('create-reader', grant.encode('ascii'))
        privileges = client('read-reader-grants', ("SHOW GRANTS FOR '" + p['reader_user'] + "'@'127.0.0.1';\n").encode('ascii'))
        grants = privileges.splitlines()
        require(len(grants) == 2 and any(x.startswith('GRANT USAGE ON *.* TO ') for x in grants)
                and any(x.startswith('GRANT SELECT ON `' + grant_schema + '`.* TO ') for x in grants),
                'reader grant scope differs from exact static schema')
        verify_live(owner)
        common_proof = {'process.pid': daemon.pid, 'process.start.epoch.millis': owner['identity']['start_epoch_millis'], 'port': port, 'schema': p['schema']}
        properties(root / 'static-import.properties', dict(common_proof, status='imported', **{
            'sql.sha256': SQL, 'source.dump.sha256': DUMP, 'tables': 19, 'rows': sum(counts), 'statements': 6296}))
        properties(root / 'database-owner.properties', dict(common_proof, nonce=p['nonce'], user=p['reader_user'], password=reader_password,
            executable=Path(owner['identity']['executable']).as_posix(), **{'executable.sha256': owner['identity']['executable_sha256']}, datadir=datadir.as_posix()))
        state.update(status='serving_static_only', actual_table_rows=dict(zip(names, counts)), import_exit_code=0,
                     proof_pins={str(root / n): sha(root / n) for n in ['database-owner.properties', 'static-import.properties']})
        write_json(root / 'state.json', state)
        print(json.dumps(dict(status=state['status'], case_root=str(root), owner_proof=str(root / 'database-owner.properties'), import_proof=str(root / 'static-import.properties')), ensure_ascii=False), flush=True)
        end = time.monotonic() + p['lease_seconds']
        while time.monotonic() < end:
            require(daemon.poll() is None, 'owned daemon ended during lease')
            request = root / 'stop-request.json'
            if request.exists():
                stop = read(within(request, root))
                require(stop == dict(nonce=p['nonce'], process=owner['identity']), 'stop request identity mismatch')
                state['stop_reason'] = 'matching_explicit_request'
                break
            if read(R / '工作佇列.json')['current_user_requested_goal'].get('test_execution_deferred_by_user', True):
                state['stop_reason'] = 'user_deferred_tests'
                break
            time.sleep(.25)
        else:
            state['stop_reason'] = 'lease_expired'
    except BaseException as error:
        primary = error
        state.update(status='failed', error_type=type(error).__name__, error=str(error))
        (root / 'original-failure.txt').write_text(traceback.format_exc(), encoding='utf-8')
    finally:
        reservation.close()
        cleanup = []
        if daemon is not None:
            if daemon.poll() is None:
                try:
                    verify_live(owner)
                    require(sha(within(root / 'root-client.ini', root)) == owner['root_client_sha256'], 'root client configuration changed')
                    command = [str(base / 'bin/mysqladmin.exe'), '--defaults-file=' + (root / 'root-client.ini').as_posix(), 'shutdown']
                    child, row = runner.spawn('shutdown', command)
                    runner.finish(child, row, 15)
                    daemon.wait(timeout=15)
                except BaseException as error:
                    cleanup.append(type(error).__name__ + ': ' + str(error))
                    if daemon.poll() is None:
                        # This handle still names exactly the child we created,
                        # even if its PID was later reused by another process.
                        try:
                            daemon.terminate()
                            daemon.wait(timeout=15)
                        except BaseException as stop_error:
                            cleanup.append(type(stop_error).__name__ + ': ' + str(stop_error))
            daemon_record.update(exit_code=daemon.poll(), finished_utc=utc())
            write_json(root / 'process-history.json', runner.history)
        state.update(finished_utc=utc(), cleanup_errors=cleanup,
                     owned_daemon_terminal=daemon is None or daemon.poll() is not None,
                     daemon_exit_code=None if daemon is None else daemon.poll())
        if primary is None:
            state['status'] = 'stopped' if state['owned_daemon_terminal'] and not cleanup else 'cleanup_incomplete'
        write_json(root / 'state.json', state)
        print(json.dumps(dict(status=state['status'], owned_daemon_terminal=state['owned_daemon_terminal'], case_root=str(root)), ensure_ascii=False), flush=True)
    if primary is not None:
        raise primary
    require(state['status'] == 'stopped', 'cleanup requires review; no terminal success published')


def status_or_stop(plan_path, stop=False):
    p, root, _, _, _ = validate_plan(plan_path)
    state_path = root / 'state.json'
    require(state_path.exists(), 'plan has never been executed')
    state = read(state_path)
    owner = read(within(root / 'owned-process.json', root))
    actual = verify_live(owner)
    require(state['pipeline'] == identity_for_pid(state['pipeline']['pid']), 'producer is absent; preserve for explicit orphan recovery')
    if stop:
        request = dict(nonce=p['nonce'], process=actual)
        path = root / 'stop-request.json'
        if path.exists():
            require(read(path) == request, 'different stop request already exists')
        else:
            write_json(path, request)
    print(json.dumps(dict(status='matching_stop_requested' if stop else 'owned_live_observed',
                         case_root=str(root), pid=actual['pid'], terminal_verified=False), ensure_ascii=False))


def recover(plan_path):
    """Explicit future cleanup for an orphan. Never starts a DB or uses kill-by-PID."""
    p, root, base, bundle, _ = validate_plan(plan_path)
    state = read(within(root / 'state.json', root))
    owner = read(within(root / 'owned-process.json', root))
    require(owner['plan_sha256'] == sha(plan_path), 'orphan plan differs')
    try:
        producer = identity_for_pid(state['pipeline']['pid'])
    except RuntimeError:
        producer = None
    require(producer != state['pipeline'], 'producer is live; use matching stop request')
    require(not (root / 'orphan-recovery.json').exists(), 'preserve original recovery attempt; no implicit retry')
    k = kernel()
    # Hold one exact object for query, wait and last-resort termination.
    # PID is used only to acquire this handle, before any shutdown command.
    handle = k.OpenProcess(0x1000 | 0x100000 | 0x0001, False, owner['pid'])
    require(handle, 'orphan absent/unqueryable; no process was touched')
    record = dict(status='orphan_cleanup_attempt', started_utc=utc(), plan_sha256=sha(plan_path),
                  process=owner['identity'], database_started=False, SQL_executed=False)
    runner = Runner(root, bundle, base, 'orphan-recovery-process-history.json')
    k.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    k.WaitForSingleObject.restype = wintypes.DWORD
    k.TerminateProcess.argtypes = [wintypes.HANDLE, wintypes.UINT]
    k.TerminateProcess.restype = wintypes.BOOL
    try:
        require(handle_identity(handle, owner['pid']) == owner['identity'], 'orphan PID reused; no action')
        verify_live(owner)
        require(sha(within(root / 'root-client.ini', root)) == owner['root_client_sha256'], 'orphan client configuration changed')
        try:
            child, row = runner.spawn('orphan-shutdown', [str(base / 'bin/mysqladmin.exe'),
                '--defaults-file=' + (root / 'root-client.ini').as_posix(), 'shutdown'])
            runner.finish(child, row, 15)
            require(k.WaitForSingleObject(handle, 15000) == 0, 'orphan shutdown did not terminate')
            record['method'] = 'verified_mysqladmin_shutdown'
        except Exception as error:
            record['shutdown_error'] = type(error).__name__ + ': ' + str(error)
            if k.WaitForSingleObject(handle, 0) != 0:
                require(handle_identity(handle, owner['pid']) == owner['identity'], 'orphan handle changed')
                require(k.TerminateProcess(handle, 1), 'owned orphan handle termination failed')
                require(k.WaitForSingleObject(handle, 15000) == 0, 'owned orphan is not terminal')
                record['method'] = 'exact_original_handle_termination'
            else:
                record['method'] = 'original_handle_observed_terminal'
        record.update(status='owned_orphan_terminal', terminal_verified=True)
    except BaseException as error:
        record.update(status='orphan_recovery_failed', error=type(error).__name__ + ': ' + str(error), terminal_verified=False)
        raise
    finally:
        k.CloseHandle(handle)
        record['finished_utc'] = utc()
        write_json(root / 'orphan-recovery.json', record)
    print(json.dumps(dict(status=record['status'], case_root=str(root), terminal_verified=True), ensure_ascii=False))


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    commands = ap.add_subparsers(dest='command', required=True)
    prepare = commands.add_parser('plan')
    prepare.add_argument('--bundle', type=Path, required=True)
    prepare.add_argument('--static', type=Path, required=True)
    prepare.add_argument('--lease-seconds', type=int, default=900)
    for name in ['serve', 'status', 'stop', 'recover']:
        parser = commands.add_parser(name)
        parser.add_argument('--plan', type=Path, required=True)
        if name in ['serve', 'recover']:
            parser.add_argument('--execute', action='store_true', required=True)
    args = ap.parse_args()
    if args.command == 'plan':
        require(30 <= args.lease_seconds <= 1800, 'lease outside bounded range')
        plan(args.bundle, args.static, args.lease_seconds)
    elif args.command == 'serve':
        serve(args.plan)
    elif args.command == 'recover':
        recover(args.plan)
    else:
        status_or_stop(args.plan, args.command == 'stop')


if __name__ == '__main__':
    main()
