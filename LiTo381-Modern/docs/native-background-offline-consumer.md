# BIN background offline consumer, 2026-10-07

`native_mode_menu_skin::background` consumes the already delivered 640×480,
top-to-bottom RGBA8 references for Native resource IDs 6314 and 7482. It returns
the existing `Img` representation without extracting or converting the assets
again. The bounded, confined `native-resource-adapter::read_pinned` reader checks
the opened length and exact SHA. Caller-provided buffers are checked against the
same fixed identities.

`select(Some(0))` selects 6314; every nonzero byte selects 7482. `select(None)`
and `load_selected(root, None)` preserve unknown state, and the latter performs
no file access. This models the source branch at VA 00496F05 / RVA 00096F05,
not an initialized runtime flag. `BackgroundAsset::blit_preview` accepts an
explicit caller-chosen origin and uses the existing CPU `Canvas` compositor.

The governing inputs are BIN `modern-contracts-20261007.json`, its
`native_background_assets` record, `Native-background-scene-contract.json`,
and the delivered RGBA manifest. The complete adoption pins, frozen source,
diff and build/test records are in the Modern parent directory's
`modern-bin-consumer-20261007.json`.

The three new `modern_bin_20261007_` checks cover all 256 source byte values,
unknown input without I/O, both real pinned resources through `Img`/`Canvas`,
every pixel of a clipped caller-defined preview, and malformed/incorrect
identity or path inputs. `tools/build-modern-bin-consumer-20261007.ps1` freezes
the source, compiles the i686 library, and executes only those three checks.
It intentionally rejects reuse of its evidence output. Equivalent compilation
uses `cargo build --offline --locked --lib --target i686-pc-windows-msvc`;
test compilation uses the same selection with `cargo test --no-run`, followed
by the produced test executable with filter `modern_bin_20261007_`.

This is an offline resource/selector/compositor increment. No default skin,
Directory scene, Native owner, actual flag publication, background origin,
renderer mode or Native draw parity is established. The call to 495520 at
496F28 is a factory boundary; it does not by itself establish a drawing
primitive. Production scene binding still requires independently qualified
existing owner/lifecycle/draw evidence, without extending the stopped private
backing analysis. No Java, SQL, socket, READY, claim or GUI action is needed.
