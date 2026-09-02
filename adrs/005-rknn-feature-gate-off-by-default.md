# ADR-005. Put the RKNN link behind a feature and default it off

*[한국어 원문](005-rknn-feature-gate-off-by-default.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-004](004-backend-abstraction-mock-first.md), [ADR-006](006-crate-split-unsafe-isolation.md) |

---

## In one line

> `npuforge-rknn` is a workspace member but **does not link RKNN in the default
> build.** So `cargo build --workspace` passes on a Windows/x86 development PC
> and in CI. Only real-hardware builds turn on `--features rknn`.

## Context

The RKNN Runtime (`librknnrt.so`) is **an ARM64 Linux-only shared library**. It
is distributed by Rockchip and is not included in this repository.

Simply putting this crate in the workspace leads to:

```text
cargo build --workspace on a Windows development PC
  -> npuforge-rknn looks for librknnrt.so
  -> it is not there
  -> the whole workspace build fails
  -> no code can be written at all
```

[ADR-004](004-backend-abstraction-mock-first.md) established the principle that
"everything has to run without hardware", and this breaks it at the link stage.

## Decision

**1. Create an `rknn` feature and leave the default empty.**

```toml
[features]
default = []
rknn = []
```

**2. Only real-hardware builds turn it on explicitly.**

```bash
cargo build --release --target aarch64-unknown-linux-gnu \
      -p npuforge-node --features rknn
```

It is registered in the repository as the `cargo build-node` alias.

**3. The types exist even in a build with the feature off.** A placeholder
implementation compiles and returns a clear error if inference is attempted.

```rust
async fn infer(&self, _input: InferenceInput) -> Result<InferenceOutput> {
    Err(NpuForgeError::new(
        ErrorCode::BackendError,
        "this binary was built without RKNN support",
    ))
}
```

**4. A mismatch between build and configuration dies at startup.**

```rust
pub const fn is_rknn_enabled() -> bool { cfg!(feature = "rknn") }
```

The node agent checks this value at startup. Giving a `[backend] type = "rknn"`
configuration to a binary built without RKNN stops it **before it takes its
first request.**

## Rationale

### It structurally blocks one mistake

**Deploying a Mock-only binary to a real node** is the most frightening
accident. It would run and the node would produce fake results — with better
throughput, since the Mock does not use the NPU.

Without the `is_rknn_enabled()` check, that accident is only discovered **after
the benchmark results are all in.** Dying at startup makes it immediately known.

This is a type already encountered in this project — the shared context and the
remote execution failure were both "failures that looked like success".

### `publish = false` is for the same reason

It guards against accidental publication from an environment without
`librknnrt.so`.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Default it on (`default = ["rknn"]`) | Windows and CI builds break. ADR-004's premise collapses |
| Take `npuforge-rknn` out of the workspace | It becomes a separate build and CI stops even checking that this crate compiles |
| Auto-detect with `#[cfg(target_arch)]` | Being aarch64 Linux is no guarantee the RKNN SDK is present. The build environment and the run environment can differ |
| Load dynamically at runtime with `dlopen` | Solves the link problem but loses compile-time verification of the FFI signatures. It would reopen a risk already caught by checking against the real headers |

## Consequences

**Gained**

- `cargo test --workspace` passes on Windows/x86 (209 tests)
- CI runs fmt, clippy, test and the aarch64 cross-build without the RKNN SDK
- Misdeployment of a Mock binary is caught at startup

**Lost / the cost**

- **The `--features rknn` path cannot be execution-verified in CI.** It
  cross-compiles but is never run. Hence the need for separate real-hardware
  integration tests (`crates/npuforge-rknn/tests/real_device.rs`)
- The build command forks in two. The feature must not be forgotten when
  deploying to real hardware (hence the `is_rknn_enabled()` check)

**New constraint introduced**

- The two `#[cfg(feature = "rknn")]` paths **have to keep the same interface.**
  Fixing one leaves the other failing to compile, or worse, silently diverging

## What would overturn this

- **If every development PC becomes ARM64 Linux**, the reason for this
  separation weakens. Though the CI runners would have to change too, so it is
  unlikely
- **If another NPU backend is added**, the feature naming and structure need
  re-examination. It is written assuming `rknn` alone
