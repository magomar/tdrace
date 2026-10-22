---
type: validation_receipt
schema_version: "1.0"
spec: "specs/136_migrate_to_standalone_cabinet_repository.md"
epic: "tdrace-sp136-knofa"
candidate_commit: "c1e6e7821204105ef162b29042e2cae7fb068aed"
verifier: "local-user"
evaluated_at: "2026-10-22T19:51:24Z"
command: "cargo tree -p cabinet && cargo test -p tdrace-app --test font_tests && cargo build -p tdrace-app --target wasm32-unknown-unknown"
exit_code: 0
duration_ms: 215
status: passed
---

# 🧾 Validation Receipt: Spec 136

- **Candidate Commit**: `c1e6e7821204105ef162b29042e2cae7fb068aed`
- **Spec**: `specs/136_migrate_to_standalone_cabinet_repository.md`
- **Command**: `cargo tree -p cabinet && cargo test -p tdrace-app --test font_tests && cargo build -p tdrace-app --target wasm32-unknown-unknown`
- **Result**: `passed` (exit code: 0)

### Verified Criteria
This receipt records only the explicit command result. It does not assert coverage of every Pseudo-Gherkin scenario.

### Execution Log Summary
```
stdout:
cabinet v0.2.0 (/home/mario/workspace/games/cabinet/crates/cabinet)
├── cabinet-core v0.2.0 (/home/mario/workspace/games/cabinet/crates/cabinet-core)
│   ├── chrono v0.4.45
│   │   ├── iana-time-zone v0.1.65
│   │   └── num-traits v0.2.19
│   │       [build-dependencies]
│   │       └── autocfg v1.5.1
│   ├── glam v0.33.7
│   │   ├── mint v0.5.9
│   │   └── serde_core v1.0.229
│   ├── serde v1.0.229
│   │   ├── serde_core v1.0.229
│   │   └── serde_derive v1.0.229 (proc-macro)
│   │       ├── proc-macro2 v1.0.107
│   │       │   └── unicode-ident v1.0.24
│   │       ├── quote v1.0.47
│   │       │   └── proc-macro2 v1.0.107 (*)
│   │       └── syn v3.0.3
│   │           ├── proc-macro2 v1.0.107 (*)
│   │           ├── quote v1.0.47 (*)
│   │           └── unicode-ident v1.0.24
│   └── serde_json v1.0.151
│       ├── itoa v1.0.18
│       ├── memchr v2.8.3
│       ├── serde_core v1.0.229
│       └── zmij v1.0.23
├── chrono v0.4.45 (*)
├── gilrs v0.11.2
│   ├── fnv v1.0.7
│   ├── gilrs-core v0.6.8
│   │   ├── inotify v0.11.5
│   │   │   ├── bitflags v2.13.1
│   │   │   ├── inotify-sys v0.1.8
│   │   │   │   └── libc v0.2.189
│   │   │   └── libc v0.2.189
│   │   ├── libc v0.2.189
│   │   ├── libudev-sys v0.1.4
│   │   │   └── libc v0.2.189
│   │   │   [build-dependencies]
│   │   │   └── pkg-config v0.3.34
│   │   ├── log v0.4.34
│   │   ├── nix v0.31.3
│   │   │   ├── bitflags v2.13.1
│   │   │   ├── cfg-if v1.0.4
│   │   │   └── libc v0.2.189
│   │   │   [build-dependencies]
│   │   │   └── cfg_aliases v0.2.2
│   │   ├── uuid v1.25.0
│   │   └── vec_map v0.8.2
│   ├── log v0.4.34
│   ├── uuid v1.25.0
│   └── vec_map v0.8.2
├── glam v0.29.3
│   └── serde v1.0.229 (*)
├── kira v0.12.4
│   ├── atomic-arena v0.1.2
│   ├── cpal v0.18.2
│   │   ├── alsa v0.11.0
│   │   │   ├── alsa-sys v0.4.0
│   │   │   │   └── libc v0.2.189
│   │   │   │   [build-dependencies]
│   │   │   │   └── pkg-config v0.3.34
│   │   │   ├── bitflags v2.13.1
│   │   │   ├── cfg-if v1.0.4
│   │   │   └── libc v0.2.189
│   │   ├── alsa-sys v0.4.0 (*)
│   │   ├── audio_thread_priority v0.35.1
│   │   │   ├── cfg-if v1.0.4
│   │   │   ├── dbus v0.6.5
│   │   │   │   ├── libc v0.2.189
│   │   │   │   └── libdbus-sys v0.2.7
│   │   │   │       [build-dependencies]
│   │   │   │       └── pkg-config v0.3.34
│   │   │   ├── libc v0.2.189
│   │   │   └── log v0.4.34
│   │   ├── dasp_sample v0.11.0
│   │   └── libc v0.2.189
│   ├── glam v0.33.7 (*)
│   ├── mint v0.5.9
│   ├── pastey v0.2.3 (proc-macro)
│   ├── rtrb v0.3.5
│   ├── symphonia v0.6.1
│   │   ├── lazy_static v1.5.0
│   │   ├── symphonia-bundle-flac v0.6.1
│   │   │   ├── log v0.4.34
│   │   │   ├── symphonia-common v0.6.1
│   │   │   │   ├── log v0.4.34
│   │   │   │   ├── symphonia-core v0.6.1
│   │   │   │   │   ├── bitflags v2.13.1
│   │   │   │   │   ├── bytemuck v1.25.2
│   │   │   │   │   ├── lazy_static v1.5.0
│   │   │   │   │   ├── log v0.4.34
│   │   │   │   │   ├── num-complex v0.4.6
│   │   │   │   │   │   └── num-traits v0.2.19 (*)
│   │   │   │   │   └── smallvec v1.15.2
│   │   │   │   └── symphonia-metadata v0.6.1
│   │   │   │       ├── lazy_static v1.5.0
│   │   │   │       ├── log v0.4.34
│   │   │   │       ├── regex-lite v0.1.9
│   │   │   │       ├── smallvec v1.15.2
│   │   │   │       └── symphonia-core v0.6.1 (*)
│   │   │   ├── symphonia-core v0.6.1 (*)
│   │   │   └── symphonia-metadata v0.6.1 (*)
│   │   ├── symphonia-bundle-mp3 v0.6.1
│   │   │   ├── lazy_static v1.5.0
│   │   │   ├── log v0.4.34
│   │   │   └── symphonia-core v0.6.1 (*)
│   │   ├── symphonia-codec-pcm v0.6.1
│   │   │   ├── log v0.4.34
│   │   │   └── symphonia-core v0.6.1 (*)
│   │   ├── symphonia-codec-vorbis v0.6.1
│   │   │   ├── log v0.4.34
│   │   │   ├── symphonia-common v0.6.1 (*)
│   │   │   └── symphonia-core v0.6.1 (*)
│   │   ├── symphonia-core v0.6.1 (*)
│   │   ├── symphonia-format-ogg v0.6.1
│   │   │   ├── log v0.4.34
│   │   │   ├── symphonia-common v0.6.1 (*)
│   │   │   ├── symphonia-core v0.6.1 (*)
│   │   │   └── symphonia-metadata v0.6.1 (*)
│   │   ├── symphonia-format-riff v0.6.1
│   │   │   ├── extended v0.1.0
│   │   │   ├── log v0.4.34
│   │   │   ├── symphonia-core v0.6.1 (*)
│   │   │   └── symphonia-metadata v0.6.1 (*)
│   │   └── symphonia-metadata v0.6.1 (*)
│   └── triple_buffer v9.0.0
│       └── crossbeam-utils v0.8.23
├── macroquad v0.4.16
│   ├── fontdue v0.9.4
│   │   ├── hashbrown v0.15.5
│   │   │   ├── allocator-api2 v0.2.21
│   │   │   ├── equivalent v1.0.2
│   │   │   └── foldhash v0.1.5
│   │   └── ttf-parser v0.25.1
│   │       └── core_maths v0.1.1
│   │           └── libm v0.2.16
│   ├── glam v0.27.0
│   ├── image v0.24.9
│   │   ├── bytemuck v1.25.2
│   │   ├── byteorder v1.5.0
│   │   ├── color_quant v1.1.0
│   │   ├── num-traits v0.2.19 (*)
│   │   └── png v0.17.16
│   │       ├── bitflags v1.3.2
│   │       ├── crc32fast v1.5.0
│   │       │   └── cfg-if v1.0.4
│   │       ├── fdeflate v0.3.7
│   │       │   └── simd-adler32 v0.3.10
│   │       ├── flate2 v1.1.9
│   │       │   ├── crc32fast v1.5.0 (*)
│   │       │   └── miniz_oxide v0.8.9
│   │       │       ├── adler2 v2.0.1
│   │       │       └── simd-adler32 v0.3.10
│   │       └── miniz_oxide v0.8.9 (*)
│   ├── macroquad_macro v0.1.8 (proc-macro)
│   ├── miniquad v0.4.11
│   │   └── libc v0.2.189
│   └── quad-rand v0.2.3
├── serde v1.0.229 (*)
└── serde_json v1.0.151 (*)

running 5 tests
test test_embedded_font_bytes_integrity ... ok
test test_format_lap_time_precision ... ok
test test_fonts_load_embedded_headless_resilience ... ok
test test_font_text_wrapping_multiline ... ok
test test_modality_modal_wrapped_lines_fit_component_space ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


stderr:
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/font_tests.rs (target/debug/deps/font_tests-871ef7f102915ecf)
warning: unused import: `PerformanceSummaryMetrics`
 --> crates/tdrace-app/src/ai/library_storage.rs:7:41
  |
7 |     BotBehaviour, BotBehaviourRegistry, PerformanceSummaryMetrics,
  |                                         ^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `ChampionshipRecord`
  --> crates/tdrace-app/src/db/mod.rs:10:24
   |
10 |     ChampionshipAward, ChampionshipRecord, ClassicAcademyProgress, Licen...
   |                        ^^^^^^^^^^^^^^^^^^

warning: `tdrace-app` (lib) generated 2 warnings (run `cargo fix --lib -p tdrace-app` to apply 2 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s

```
