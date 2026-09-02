# ADR-008. Internal communication uses gRPC (tonic + Protocol Buffers)

*[한국어 원문](008-grpc-tonic-protobuf.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-003](003-central-simple-scheduler.md), [ADR-012](012-want-float-zero-blob-v2.md), [ADR-024](024-error-code-scheme.md), `docs/01-TECHSPEC.md` §5.3, §7 |

---

## In one line

> Client↔scheduler and scheduler↔node communication uses **gRPC**. The schema
> lives in one place as `.proto` and Rust code is generated from it. The
> management API and the dashboard's REST/JSON are kept separate.

## Context

Most of what moves through this system is **a large binary blob**.

```text
request   raw RGB 640x640x3   = 1,228,800 byte
response  9 raw tensor blobs  = 1,218,000 byte  (want_float=0)
```

And although there are only three nodes, hundreds of these move per second
(the INT8 3-node target is 471 inf/s).

There were three protocol candidates.

| | |
|---|---|
| REST + JSON | works everywhere and is easy to debug. Inflates binary with base64 |
| gRPC | binary as-is, schema enforcement, code generation |
| A hand-rolled binary protocol | could be fastest. Everything has to be built by hand |

## Decision

**1. Internal RPC is gRPC + Protocol Buffers.** Implemented with `tonic`.

**2. The schema lives in one place, the `npuforge-proto` crate.** Rust types
are generated from `.proto` at build time.

**3. The services are split in two.**

| Service | Direction | Purpose |
|---|---|---|
| `SchedulerService` | client → scheduler | `Infer`, `BatchInfer`, `ListNodes` |
| `NodeService` | scheduler → node | inference delegation, status queries |

Node registration and heartbeats also travel over gRPC.

**4. The management API and dashboard are separate, on REST/JSON + axum.**
They are called directly from the browser, so putting them on gRPC would add
another gateway.

**5. The payload arrives as a single `bytes` field.** The tensor structure is
described not by protobuf but by our own blob format
([ADR-012](012-want-float-zero-blob-v2.md)).

## Rationale

### 1. Avoid base64

Sending a 1.23 MB image over REST/JSON requires base64 encoding. That makes it
**about 1.33× larger** and adds encode/decode CPU on both ends.

Both are damaging in this project. The network is already close to saturation
at aggregation
([ADR-014](014-10g-aggregation-separate-scheduler.md)), and CPU is already a
bottleneck under sustained load.

### 2. The schema has to be in one place or three nodes drift apart

The three nodes run **the same binary**, but the scheduler runs on a separate
host. If message definitions are scattered through the code, one side gets
updated without the other.

With `.proto` as the single source, both sides are generated from the same
definition.

### 3. The timing breakdown fields have to travel structured

Eleven timing fields (`TimingBreakdown`) come back with each response. They are
this project's central output, so **a field must not silently disappear.** The
protobuf schema enforces that.

### 4. The Rust ecosystem is ready

`tonic` runs on Tokio and comes with streaming, timeouts and connection reuse
built in. Reusing a per-node channel to reduce connection cost also works
directly.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| REST + JSON (internally too) | 1.33× base64 inflation plus encoding CPU. Both network and CPU are already tight |
| REST + multipart/octet-stream | Avoids the inflation but loses schema enforcement. The timing fields would have to be kept in sync by hand |
| A hand-rolled binary protocol | Could be fastest, but reconnection, streaming and error propagation all have to be built. That time is time not spent measuring |
| Extending gRPC to the management API | Cannot be called directly from a browser. Adds a grpc-web gateway |

## Consequences

**Gained**

- Binary carried without inflation
- A single source for message definitions
- The mock 3-node integration test **runs over real gRPC** — it is one process,
  but the transport path is the same as on real hardware

**Lost / the cost**

- Cannot be poked directly with `curl`. Another debugging tool is needed
- Changing `.proto` involves the build pipeline (`build.rs`)
- There are now two protocols (gRPC + REST). Error representation has to agree
  across both → [ADR-024](024-error-code-scheme.md)'s `NPF-xxxx` is that glue

**New constraint introduced**

- Message size limits have to be managed explicitly. A 1.23 MB request fits
  under the default 4 MB limit, but experiments that increase input size (S6)
  will need to check

## What would overturn this

- **If the input becomes JPEG and payloads drop to the 100 KB class**, the
  absolute cost of base64 inflation shrinks. The schema-enforcement reason
  still stands
- **If a public external API becomes a requirement**, consider putting a REST
  gateway in front of gRPC. That is not a reason to change the internal
  protocol
