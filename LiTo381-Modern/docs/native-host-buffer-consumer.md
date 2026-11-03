# Host / alias / port byte consumer, 2026-10-07

`native_city_runtime::startup_host::bytes` implements the accepted
`saved_host_64AD74` raw-copy contract as pure library data. `HostBuffer` has
256 byte positions and `AliasBuffer` has 28. `PortWord` independently stores
two little-endian bytes, with unsigned bits and a signed read view. No socket
is qualified by the numeric contents of this data object.

`NulBuffer::copy_c_bytes` scans at most its capacity and copies through the
first NUL, including that byte. Bytes after that terminator are ignored.
Repeated shorter copies preserve the old destination tail. `known_bytes`
exposes only the prefix that has actually been written; an uninitialized
buffer has no C string, while a copied single NUL has an initialized empty
payload. The safety bounds permit at most 255 host payload bytes and 27 alias
payload bytes. The original code's own local capacity enforcement is unproved.

`copy_payload` adapts an already length-delimited payload by appending NUL,
without decoding or normalizing bytes. It rejects embedded NUL and excess
length before modifying the destination. `Buffers::copy_data` validates all
optional inputs on an owned copy, then commits together; omission leaves a
field unchanged. It represents only byte writes, with no initialization,
thread, epoch or owner authority.

The existing `FixtureStartupHost::parse` still requires its explicit
synthetic kind, canonical numeric IPv4 loopback and existing JSON bounds.
Its validated ASCII bytes now go into a `HostBuffer`. `target` reads that
buffer's payload for the same numeric conversion and fixed port2000. This
synthetic adapter is not the Native 050104CD return or a saved-host event.
`configure_source_startup_host_fixture` retains the original immutable,
before-owned-connection gate verbatim.

`entry_transfer::State::start` now copies the existing qualified alias payload
into `Key::byte_storage.alias`. `State::alias`, used by the current alternate
roster C9 replay path, reads the buffer rather than the presentation Vec.
`selected_name` remains the same payload projection for existing snapshots.
Known transfer/return port writes update independent `PortWord` storage.
Transfer `byte_storage.host` stays uninitialized: no profile, alias, peer,
SocketAddr or synthetic fixture is promoted to Native saved-host authority.
The port projection records only these explicit source-semantic steps; it is
not an assertion of the global port after unrelated fixture I/O/timer paths.
Existing Modern transfer retirement clears its owned key; that does not
claim the Native global buffers are cleared.

Four new `modern_host_20261007_` CPU checks cover both capacity boundaries,
missing NUL, raw non-ASCII bytes, source ownership, ignored source tails,
preserved destination tails, independent fields, atomic multi-field errors,
all 65536 port bit patterns, unchanged synthetic startup restrictions and
the affected transfer start/alias/return/revision-error paths. Only these
new checks execute. The build script freezes the library source and retains
compile/test logs under `target/modern-host-buffer-consumer-20261007`.

Actual initialized host bytes, codepage/IDN treatment, private050104CD success,
Native owner/epoch and indirect clearing remain unknown. This increment opens
no socket, provider, Java/SQL, game or GUI and does not extend private analysis.
