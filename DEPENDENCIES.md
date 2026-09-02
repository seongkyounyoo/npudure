# Third-party dependencies

*[한국어 원문](DEPENDENCIES.ko.md)*

NPUDure is Apache-2.0. This document sets out the licenses of the third-party
components used alongside it.

Models and datasets follow a separate document →
[`MODEL_LICENSES.md`](MODEL_LICENSES.md)

---

# 1. Rust crates

**101** crates as of 2026-08-11 (including transitive dependencies).
`sha2` (model hash verification), `tokio-stream` (a listener for tests) and
their transitive dependencies were added in M2.
All are of the MIT / Apache-2.0 family, with no copyleft.

## 1.1 License distribution

| Count | License |
|---:|---|
| 57 | MIT OR Apache-2.0 |
| 19 | MIT |
| 5 | Apache-2.0 |
| 4 | Apache-2.0 OR MIT |
| 3 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| 2 | Unlicense OR MIT |
| 2 | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| 2 | BSD-2-Clause OR Apache-2.0 OR MIT |
| 1 | (MIT OR Apache-2.0) AND Unicode-3.0 |

**All are permissive, with no GPL- or AGPL-only dependency.** There is no
problem distributing under Apache-2.0.

Where a crate offers several options with `OR`, Apache-2.0 is chosen. The two
that include `LGPL-2.1-or-later` can also be taken as MIT or Apache-2.0, so no
LGPL obligation arises.

## 1.2 How it is verified

The allowlist is defined in [`deny.toml`](deny.toml) and checked in CI.

```bash
cargo install cargo-deny
cargo deny check licenses
```

For a manual count:

```bash
cargo metadata --format-version 1 --all-features \
  | python3 -c "
import json,sys,collections
m=json.load(sys.stdin); c=collections.Counter()
for p in m['packages']: c[p.get('license') or 'UNKNOWN'] += 1
for lic,n in c.most_common(): print(f'{n:4d}  {lic}')
"
```

**Checked whenever a new dependency is added.** CI fails if a license not on
`deny.toml`'s allowlist appears.

## 1.3 Direct dependencies

Those declared in `Cargo.toml`.

| Crate | Purpose |
|---|---|
| `tokio` | the async runtime |
| `serde` / `serde_json` / `toml` | serialization, configuration parsing |
| `thiserror` | error types |
| `uuid` | Request IDs |
| `bytes` | payload buffers |
| `async-trait` | the backend interface |
| `tracing` / `tracing-subscriber` | structured logs |
| `parking_lot` | registry locking |
| `rand` | the Mock backend's deterministic randomness |
| `libc` | RKNN FFI (unix only) |
| `cc` | building the C wrapper |

---

# 2. RKNN software

**None of it is included in this repository.**

| Component | Source | Terms |
|---|---|---|
| RKNN Runtime (`librknnrt.so`) | pre-installed in the board OS image | Rockchip's own terms |
| `rknn_api.h`, `rknn_matmul_api.h`, `rknn_custom_op.h` | pre-installed in the board OS image | the same |
| RKNN-Toolkit2 | PyPI (`rknn-toolkit2==2.3.0`) | the same |
| `rknn_model_zoo` | GitHub, **Apache-2.0** | code may be quoted |

The `npuforge-rknn` crate links dynamically against `librknnrt.so` only when the
`rknn` feature is on. It defaults to off, so the workspace builds without the
SDK.

**Note.** The `rknn_model_zoo` repository is Apache-2.0, but **the model files it
distributes carry their own original licenses.** A repository's license and a
datum's license are separate things. → `MODEL_LICENSES.md` §2

---

# 3. Container image

What `tools/model-converter/Dockerfile` uses.

| Component | License |
|---|---|
| the `ubuntu:22.04` base | per package (mostly GPL/LGPL/MIT) |
| `rknn-toolkit2` | Rockchip's own terms |
| `torch`, `onnx` (transitive) | BSD-3-Clause / Apache-2.0 |
| `opencv-python-headless` | Apache-2.0 |
| `pillow` | MIT-CMU |

This image is **a conversion tool, not a distributed artefact.** Users build it
themselves. Publishing the image itself to a registry would require separately
reviewing the redistribution terms of the packages it contains.

---

# 4. Open items

| Item | Status |
|---|---|
| `cargo-deny` CI integration | `deny.toml` written, a job defined in `.github/workflows/ci.yml`. Execution not yet verified |
| Confirming the RKNN Runtime redistribution terms in the original text | incomplete |
| Updating the `NOTICE` file | after the dependencies settle |
| Deciding whether to publish the container image | undecided |
