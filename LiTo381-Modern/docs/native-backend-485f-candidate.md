# Explicit 485F player candidate

`LiToModern-485F-Candidate` is a repository Cargo binary. `default-run = "LiToModern"`
and the existing `src/main.rs` Modern profile routes remain unchanged.

Build from the repository with Rust/MSVC for `i686-pc-windows-msvc`:

```powershell
cargo build --offline --locked --target i686-pc-windows-msvc --bin LiToModern-485F-Candidate
```

To freeze a separate debug candidate, its actual linked library, source inputs and
only the two new CLI CPU checks, reuse the existing development cache:

```powershell
pwsh -NoProfile -File tools/build-r542-native-candidate.ps1
```

For a current-source compile without compiling or running any tests, opt in with
a fresh evidence name:

```powershell
pwsh -NoProfile -File tools/build-r542-native-candidate.ps1 -CompileOnly -EvidenceName r542-current-source-<unique-name>
```

Compile-only mode makes one offline, locked Cargo build with the existing
Cargo 1.95.0/MSVC dependency cache copied into a fresh evidence target directory.
The original cache is read-only; package and incremental artifacts are omitted
from the copy so the current package compiles in that single invocation. Scratch
files stay under project-local `target/build-scratch` with a unique ASCII basename.
It freezes verified source copies and their digest, raw Cargo JSON and verbose
compiler logs, exact candidate/library artifact records, and the two resource
manifest identities. It checks source drift before and after Cargo and stops on
the first failure, drift or output collision. Test compilation and execution are
explicitly accounted as `NOT RUN`; the resulting executable is never launched.
The receipt is compile evidence awaiting review, with no player acceptance.

The script preserves existing outputs. A later rebuild must select a fresh
`-EvidenceName r542-<name>`; it must not overwrite historical artifacts or replay
old test results. The finite new checks parse arguments and dispatch to a CPU
stub. They never call `Accepted::load`, read READY, claim a run, connect, or open UI.

The only accepted launch syntax is:

```text
LiToModern-485F-Candidate.exe --candidate-backend-485f <absolute-fresh-READY-path> <trusted-SHA256>
```

Launching retains the original `Accepted::load` / `run_backend_485f` entry.
`Accepted` validates the exact fresh READY, accepted identity, lease and one-use
claim. This document and building the binary do not supply a fresh acceptance.

The two resource manifests are the existing R532 inputs, pinned in source:

| Manifest under the build-time repository root | Bytes | SHA256 |
| --- | ---: | --- |
| `logs/continuation-20261003/r486-map-binding.json` | 208039 | `14805BC5D26A88EEAAE3B182A4A477D30AE7B0F26212B760F1CFA29B452DCA6E` |
| `logs/continuation-20261003/r506-native-menu-skin-binding.json` | 37911 | `27475914E201922FF0661EDA513B85E73C3E2CBF33FECB3CE71C7345CEF9D379` |

Cargo supplies `CARGO_MANIFEST_DIR`; no custom `R532_*` environment variables are
needed. This workspace candidate uses that exact resource root. Moving the EXE
does not relocate its resources or make it a portable release. Existing resource
adapters verify nested payload pins at resource loading; the build script verifies
the two manifest pins without starting those loaders or the UI.

The linked library reflects the current source, including any offline components
adopted after the original R542 build. Each fresh receipt identifies its actual
library; the original R542 library identity is historical. This entry calls only
the existing 485F application path; it imports no fixed trace, room/map/player
defaults, replay milestones, source generations or clocks. Building and CLI checks
do not establish new live ROOM, GUI, WORLD, movement or release acceptance.
