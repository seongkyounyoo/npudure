# NPUDure status

*[한국어 원문](TODO.ko.md)*

- Last updated: **2026-08-21**
- To the talk: **D-99** (2026-11-28)
- Feature freeze: 2026-11-15

> This document exists to see **what has to be done now** at a glance.
> Why it was done that way is in `board-worklog.md`, the values are in
> `environment-matrix.md`, and the state is in `infrastructure.md`.

---

# ▶ Current state: **the measurement lineage is closed** (2026-08-21)

Everything from S2 through S3.9b and S0-D is closed. **421 measurements, error
rate 0 throughout.** What remains is presentation material (figures).

| Lineage | Status | Conclusion |
|---|---|---|
| **Transport** | closed | operating point = **2 connections per node @ c12**. 3N 387.2 inf/s (+13.3%) |
| **Scaling** | closed | 3N **2.86× (95.3%)**. The loss **shows up in the tail** — p50 flat, p99 +36% (S3.9a). The micro-mechanism was not isolated |
| **Sustained load** | closed | under active cooling, short-run = sustained (−1.9%) |
| **Policy** | closed | RR is vulnerable to heterogeneity; adaptive gives tail −37%. Default **stays `ect`** |
| **io_uring (S4)** | **refuted** | what it targets is 1% of transport cost. CPU is not the constraint (S3.9b) |

```text
local direct 161.5   operating point 135.5   residual gap 26.0 inf/s = 16.1% of direct
  -> looks like path latency rather than CPU cost (out of scope, observation only)
```

## Operating the cluster

```bash
# connect + preflight
for h in npuforge-k npuforge-q npuforge-j npuforge-server; do ssh $h hostname; done
bash scripts/preflight-check.sh --with-inference

# restarting a node - pkill by comm (never -f, ADR-017), log redirect mandatory
ssh npuforge-k 'pkill -9 npuforge-node; sleep 3;   setsid nohup ~/npuforge/npuforge-node.s36 --config ~/npuforge/node.toml   >>~/npuforge/node.log 2>&1 & disown'
# helper: npuforge_restore_cluster (scripts/lib/remote.sh)

# recount the run total
bash scripts/count-runs.sh
```

> **The two harness invariants** (`experiments/README.md` §4.12) — new harnesses
> keep both.
> 1. Verify shared-resource state at the shared resource
>    (`npuforge_assert_cluster_free`).
> 2. Do not treat the results path as an overwritable scratch directory.

## The M3 topology (settled and measured 2026-08-20)

```text
        server 192.168.123.9  (Xeon x2 24T / 16GB / Rocky 9.4)
                    |
                  10GbE          <- aggregation. 10G full measured
                    |
          NEXI NS-S25G10G-N  (2.5G x4 + 10G x2)
              |-- 2.5G -- king  .3
              |-- 2.5G -- queen .5
              \-- 2.5G -- jack  .4
```

Worker links are 2.5G and **only aggregation is 10G.** The old `dealer` (a
laptop) was removed and the scheduler role moved to `server`.
`infrastructure.md` §1.

> **Static IPs done.** The rework changed the board IPs wholesale
> (`.12/.16/.33` → `.3/.4/.5`) and left the SSH aliases stale. All four are now
> pinned host-side static (`manual`) (§1.1, `infrastructure.md` §2.3).

---

# ▶ The policy lineage — what closed and what was deferred (2026-08-21)

**Closed.** RR is vulnerable to heterogeneity, and load-aware scheduling with
state freshness fixed improves RR's tail markedly (p99 −37%). Both LQ and ECT
work, with no regression. **The default stays `ect`.**

**Deferred (future work).** Whether ECT beats LQ under strong heterogeneity is
undetermined. But **that outcome does not change the central conclusion** — the
core is "load-aware scheduling absorbs heterogeneity", and it holds either way.

S0-D's calibration made that question **reproducible.** An answer takes 40
minutes whenever wanted (fan on, no preheat needed).

```text
king CPU cap    1200   1008    816    600
node latency spread 1.33x  1.79x  2.26x  3.93x     <- 816 reproduces S0-A (2.4x)
```

→ [`experiments/S0_D_CAPACITY_HETERO.md`](experiments/S0_D_CAPACITY_HETERO.md) §6

---

# ▶ S3.9b complete (2026-08-21) — **S4 io_uring cancelled/shelved**

```text
Question   Does io_uring recover the remaining 16.1%?
Answer     No. What it targets (syscall entry) is 1% of transport cost, and 8%
           under the most generous assumption. And CPU is not the constraint.
```

| | Value |
|---|---|
| Transport cost | **16.35 CPU-ms/req** (user 9.37 / kernel 6.99) |
| Network syscalls | ~165 per request × 1 µs = **0.165 ms = 1.0%** |
| Board CPU | **48.9% idle**, the hottest core cpu0 at 78.8% busy (softirq) |
| cpu0 softirq | spread with RPS in S3.5 §4.3 → **−0.2% null** |

**CPU-ms/req is a cost, not a constraint.** Reducing usage of an unsaturated
resource does not raise throughput.

The large term was recorded separately — **user time exceeds kernel time**
(serialization and user-space copies are 57% of transport cost). But per the
third branch of the pre-registered rule, **we stop here**: as long as CPU is not
the constraint, there is no guarantee reducing this raises throughput.

An out-of-scope observation: the gap looks like **path latency** rather than CPU
cost (of the +37.3 ms of latency, node CPU is 16.35 ms, and the 1.2 MB payload
round trip alone is 8.2 ms). If there is a lever, it is **payload size**, not
io_uring.

→ [`experiments/S3_9B_NODE_RESIDUAL.md`](experiments/S3_9B_NODE_RESIDUAL.md)

---

# ▶ (the completed plan) S3.9b — node-side residual cost profiling

## The question (narrow)

> **In the residual gap between 161.5 and 135.5, do node-side serialization,
> copy and syscall costs account for a meaningful share?**

```text
local direct 161.5   operating point 135.5   gap 26.0 inf/s = 16.1% of direct
                                             (1 - 135.5/161.5)
```

> **The 13.2% that had been circulating is wrong.** That percentage came from
> 140.1 (S3.6 C), and 140.1 is a **c32 = overload region** measurement unusable
> for operating decisions (README §4.1). Paired with an operating-point number,
> it mixed two lineages.

**The objective is not to explain the whole gap.** S3.9a separately surfaced the
scale-out tail/TCP cost, so there is no reason a node-side profile has to account
for all 26 inf/s. Whatever is not explained stays unexplained.

## The verdict

| Result | Decision |
|---|---|
| syscall and copy are **large enough** | **proceed to S4 io_uring** |
| **small** | **cancel/shelve S4** |
| **some other term is large** | record that term only. **If it is outside the core scope, dig no further** |

The third row matters. Even if the profile points at an unexpected term, chasing
it is not this experiment's job. Record it and stop if it is out of scope.

## The harness invariants (established 2026-08-21)

Kept without exception when writing a new harness. Both came from real
incidents.

1. **Verify shared-resource state at the shared resource.**
   `npuforge_assert_cluster_free` — do not start if `npuforge-bench` is running
   on the server. Local process observation lies depending on the platform.
2. **Do not treat the results path as an appendable/overwritable scratch
   directory.** Stop if the existing directory is not empty. Distinguish with
   `NPUFORGE_SUFFIX`.

---

# 0. At a glance

| Area | Status |
|---|---|
| Software (M0) | ✅ done — workspace, common, mock backend, policy engine, CI |
| Hardware infrastructure | ✅ done — thermal spread 5.6 °C, no **NPU** throttling. But **the CPU is downgraded** (fans are for the S0-B comparison) |
| RKNN verification | ✅ backend implemented, context sharing risk confirmed by measurement |
| Model conversion | ✅ FP16 and INT8 done, accuracy verified |
| gRPC communication (M2) | 🟡 nearly done — wiring, retries and Mock cluster verified; metrics remain |
| Benchmarks (M3) | ✅ **done (2026-08-21)** — S2, S3, S3.5–3.9b, S0-A–D, **421 runs / error rate 0** |
| Dashboard (M6) | ⬜ not started |

**M3 blockers — all resolved (2026-08-20)**

| # | Item | Status |
|---|---|---|
| 1 | 2.5G/10G switch | ✅ NEXI NS-S25G10G-N |
| 2 | A server with a PCIe slot | ✅ Xeon x2 / 16GB / Rocky 9.4 (.9) |
| 3 | 10G NIC + cable | ✅ `enp4s0` 10GBASE-T, 10G full measured |
| 4 | ~~switch to `want_float=0`~~ | ✅ 2026-08-12 |

**Remaining work (as of 2026-08-21)**

| Item | Scale | Why |
|---|---|---|
| ~~more figures for the talk~~ | ✅ **done (2026-08-21)** | 7 added — `scripts/make-experiment-figures.py`. Paths in handoff §5 |
| Prometheus metrics (the M2 remainder) | — | the last piece of the gRPC communication item |
| Dashboard (M6) | — | not started |
| Move to systemd | — | draft at `scripts/npuforge-node.service.in`. Together with `pkill`→`systemctl stop` |
| Regenerate queen and jack's SSH host keys | — | the two boards share a key and cannot be told apart |

> **No more measurement is needed.** Before starting an additional experiment,
> read `experiments/README.md` §2 (the exclusion table) and §7 (the open list)
> first — to check whether the candidate is already excluded or conditionally
> open.

---

# 1. Immediate tasks

## 1.1 User tasks (physical, purchasing)

- [x] **Make the three boards' placement uniform** — concluded as needing no
  action (2026-08-11)
  - The 19 °C gap **did not reproduce** under controlled re-measurement
  - 15 minutes of concurrent 8-thread load: king 75.8 / queen 70.2 / jack
    74.8 °C (spread 5.6 °C)
  - Never exceeded 90 °C, no NPU clock drop (all 928 samples at 950 MHz)
  - Throughput spread 3.5% (80.5 / 77.7 / 77.8 inf/s)
  - The earlier 19 °C is judged to have been inflated by a load profile
    difference (sweep vs fixed, a 6-minute head start)
  - Details: `board-worklog.md` §2.19
- [ ] **A fanless (S0-A) cluster measurement** — today's baseline is active
  cooling (condition B). Settling the 27% requires measuring condition A over
  the same gRPC path too (§9)
- [x] **Install three identical fans** (2026-08-20) — 120 mm 5V USB, one per
  node (larger than the board). All measurements on 2026-08-20 were under this
  active cooling (condition B)
- [x] **2.5G/10G switch** (2026-08-20) — NEXI NS-S25G10G-N (2.5G×4 + 10G×2)
- [x] **Obtain the scheduler server** (2026-08-20) — Xeon E5-2630L ×2 / 16GB /
  Rocky 9.4 (.9)
- [x] **10G NIC + cable** (2026-08-20) — the server's onboard 10GBASE-T, 10G
  full measured
- [x] **Static IPs** (2026-08-20) — all four pinned via host NetworkManager
  static (handled as development work). An ipTIME router reservation is
  optional; if done, use the table below.
  ```text
  king 22-94-FF-34-46-B1 ->.3   jack 62-CE-3B-B6-E4-41 ->.4
  queen 7E-D8-D7-40-45-82 ->.5  server 6C-B3-11-13-2F-38 ->.9
  ```
- [x] **Decide the calibration image approach** — 200 COCO val2017 images
  adopted (2026-08-11)
  - Deterministic selection with `tools/model-converter/fetch_calibration.py`
    (fixed seed)
  - The images are not put in the repository; only the manifest (licensing)

## 1.2 Development tasks

- [x] `preflight-check.sh` — a hard-failing check before benchmarks (2026-08-11)
  - alias↔hostname, matching kernel/RKNN/driver/model hashes
  - governor, idle temperature, input voltage, residual load, NTP, session count
  - `--with-inference`: whether the three boards give the same answer to the same
    input (the §9 lesson)
  - Negative tests confirm it actually detects (a swapped model, residual load)
- [x] Record `boot_id` — detect and invalidate a reset during a run
  - The node reports it in the heartbeat; the scheduler warns on a change
    (implemented in M2)
- [x] Extend benchmark telemetry — collect temperature, voltage and boot_id via
  the `ListNodes` RPC
  - Querying via heartbeat has the scheduler record those values as observations
    and overwrite its state. A read-only RPC was added separately.
- [ ] **Regenerate queen and jack's SSH host keys** — they are identical and
  cannot be told apart cryptographically. A changed IP attaches you to the wrong
  board without a warning. With DHCP the IPs do change (as happened in the
  2026-08-20 rework), so this must not be left alone.
  ```bash
  ssh npuforge-j 'sudo rm -f /etc/ssh/ssh_host_* &&     sudo ssh-keygen -A && sudo systemctl restart ssh'
  ssh-keygen -R npuforge-j   # clean up known_hosts on the PC
  ```
- [x] **Switch to `want_float=0`** (2026-08-12) — the `[worker] want_float`
  setting
  - blob v2 carries `scale` and `zero_point`. Real-board dequantization verified
    to a maximum error of 9.5e-7
  - Throughput **INT8 +17.3% / FP16 +15.7%**, output size a quarter
- [ ] **A per-request latency raw dump option in the bench** (need confirmed
  2026-08-20)
  The bench currently writes only summary percentiles per run to JSON. So p95/p99
  in tables pooling several runs are **the average of run-level percentiles**,
  not pooled percentiles over all requests (S2 §7.4.1).
  - Run-level averaging dilutes each run's worst window and **makes the tail read
    low.** Fine for comparing conditions, but the absolute values must not be
    quoted as "this system's p99".
  - S3.7 chooses the operating point by the tail, so this distinction actually
    started to matter.
  - To do: dump per-request latency with `--dump-samples <path>` and have the
    analyser compute pooled percentiles. **But the bench is a measurement tool
    and is not changed while S3.7/S3.8 are running** — after the frozen period.
- [x] **Recover the jack node** (2026-08-20) — the hardware was fine
  - eth0 **2.5G up**, IP 192.168.123.4, binary, configuration, model hash
    (`dba155d2…`) and governor all normal. No trace of OOM or segfault.
  - `dmesg` showed the history: at boot the cable was on **eth1**
    (`t=13.6s eth1 link up`), at `t=620819s` eth1 link down, and at `t=689135s`
    **eth0 link up** — the cable had been physically moved. But a link drop does
    not kill the process (the node retries registration).
  - **The cause was never established.** Because there were no logs — the
    startup procedure is `setsid nohup ... &` and the log redirect was missing,
    so stdout was thrown away.
  - Verified after recovery: 3 nodes at 335.4 inf/s, jack 33.3% (3362
    requests), 0 errors, every preflight `--with-inference` item passed (**the
    three nodes' inference output hash identical**, `e84c5b53…`).
  - Preventing recurrence: `lib/remote.sh`'s `npuforge_restore_cluster` restores
    all three nodes and **forces the log redirect**. The problem persisted
    because the measurement scripts killed queen and jack to make a single-node
    configuration but only restored queen.
- [ ] **Move node startup to systemd** — draft at
  `scripts/npuforge-node.service.in`
  It brings log retention (journald), the last exit status and a restart policy.
  In the jack case above, "not knowing why it died" was worse than the process
  dying.
  - ⚠️ **Do not install it now.** The measurement scripts make a single-node
    configuration with `pkill -9 npuforge-node`. With `Restart` set, systemd
    revives it immediately and **a 1-node measurement silently becomes 3-node.**
    The kind of accident you do not know is wrong.
  - To be done alongside: replace `pkill` in `run-*.sh` with `systemctl stop`.
    Handled after the measurement campaign (S3.8) ends.
- [ ] **Compare `ondemand` vs `performance` over 300 s** ← checking the scope of
  §11's conclusion
  - The +7% is a 120-second measurement. Under sustained load `performance` may
    heat up faster and be worse. `discuss.md` §12
- [ ] **Extend S0 to 30 minutes** — settle steady-state throughput and the timing
  of CPU downgrade
- [ ] **Deploy the INT8 model to queen and jack** — currently only on `king`
- [x] **The scheduler build path = native on server** (verified 2026-08-20) —
  rust/cargo (dnf 1.92, satisfying MSRV 1.85), gcc, protoc and git installed on
  server, with a `git archive` tarball scp'd across. The node (aarch64) is still
  built on king. Cross-building is avoided because of linker problems
- [x] **Static IP pinning** (2026-08-20) — server, king, queen and jack all
  manual. Host NetworkManager static rather than router reservations. Same IPs,
  so SSH was uninterrupted
- [ ] **Open the gRPC port in server's firewall** — firewalld public zone, before
  measuring
- [ ] Configure `server` as an NTP server + wait with `chronyc waitsync`
- [x] **Ease the scheduler RSS concern** (2026-08-20) — server RAM 3GB →
  **16GB**. The dealer laptop constraint is resolved. RSS is still observed in S2
  (`environment-matrix.md` §10.1)
- [x] CPU governor → `performance` (2026-08-12) — made permanent with a systemd
  unit
  - `scripts/set-cpu-governor.sh`, survival across reboot confirmed
  - **+7% throughput.** All existing figures had been on ondemand
    (discuss.md §11)
- [x] Settle `worker_count` — **8**, `core_mask` unset (discuss.md §4)

---

# 2. Progress by milestone

## M0. Repository and environment — ✅ done

- [x] Rust workspace (7 crates, edition 2024)
- [x] `npuforge-common` — types, error codes, configuration, backend interface
- [x] `npuforge-mock-backend` — deterministic seed, injection of latency, error
  rate and speed variance
- [x] `npuforge-rknn` stub — the Windows build passes via the feature gate
- [x] Three scheduling policies (round-robin / least-queue / ect)
- [x] Node registry + state machine + drain/disable
- [x] CI (fmt, clippy, test, aarch64 cross, cargo-deny)
- [x] LICENSE (Apache-2.0), NOTICE, DEPENDENCIES.md, MODEL_LICENSES.md
- [x] Configuration examples (scheduler, node, mock 3-node)
- [x] Tests passing — 81 at M0, **currently 209 across the workspace**
  (2026-08-14)

## M1. Single-node inference — 🟡 in progress

- [x] The RKNN C wrapper written and **compile-verified on real hardware**
- [x] Thread-safety verified — **RKNN 2.3.0 confirmed thread-safe**
- [x] FFI signatures checked against the real headers
- [x] The model conversion environment (Docker, rknn-toolkit2 2.3.0)
- [x] YOLOv8n FP16 converted and deployed to 3 nodes
- [x] INT8 conversion — 6.46MB (−33% against FP16's 9.65MB)
- [x] Inference accuracy verified — detection-level comparison on a real board,
  `results/accuracy/README.md`
- [x] The real `npuforge-rknn` implementation — context pool + multiple outputs
  (2026-08-11)
  - Six real-hardware integration tests pass (`tests/real_device.rs`)
  - **A shared context produces 100% wrong results with 0 API errors** —
    measured (the correction in `environment-matrix.md` §3.1)
- [ ] 1,000-iteration inference stability (24,000 confirmed via soak; a formal
  test is separate)

## M2. Remote inference — 🟡 only metrics remain

- [x] `npuforge-proto` — the .proto definitions and tonic wiring
- [x] The `NodeService` gRPC server (node side)
- [x] Node registration / heartbeat (registration backoff retries,
  `must_reregister` re-registration)
- [x] The `SchedulerService` gRPC server
- [x] The scheduler → node gRPC client (per-node channel reuse)
- [x] Local queue + worker pool
- [x] Error handling and retries (selecting a different node on retry)
- [x] Model directory loading + SHA-256 verification
- [x] **Local 3-node Mock cluster confirmed working** (without hardware)
- [ ] Basic metrics (Prometheus)
- [x] The `npuforge-bench` CLI — load generation, aggregation, run-validity
  judgement (2026-08-11)

### What was verified (2026-08-11)

The integration test `crates/npuforge-scheduler/tests/mock_cluster.rs` — it runs
over real gRPC with the scheduler attached to 3 nodes. It is one process, but the
transport path is the same as on real hardware.

| Item | Result |
|---|---|
| Requests spread across 3 nodes | ✅ round-robin uses all three |
| Bypass when 1 node dies | ✅ 6/6 succeeded; the dead node produces no results |
| All nodes dead | ✅ `NPF-1302` + the list of nodes attempted |
| Timing breakdown | ✅ both node-measured and scheduler-measured sections populated |
| Avoiding a slow node | ✅ least-queue uses the fast nodes more |

Also confirmed with four real processes (scheduler + 3 nodes).
Killing the scheduler and bringing it back has all three nodes **re-register by
themselves within about 1.3 seconds.** A node switches a failed heartbeat
straight to re-registration — a transient network error and a scheduler restart
are indistinguishable, so the more expensive option was taken, and registration
is idempotent.

## M3. Multiple nodes — 🟡 cluster operation confirmed (2026-08-20)

- [x] **Three real nodes registered** — king/queen/jack confirmed registered with
  the scheduler (.9)
- [x] **Round Robin routing** — an exact 33.3% three-way split in the pilot bench
- [x] The `npuforge-bench` CLI
- [x] **Pilot 3-node inference** — c6 146 / c24 336 inf/s, 0% errors
- [x] **The first S2 scalability measurement** (2026-08-20) — 1/2/3 nodes at
  111.6/228.7/337.7 inf/s, **scaling efficiency ~98%**. Cluster node ceiling 115
  < local 157 (27% scheduler overhead). Preflight passed. RESULTS §2.5,
  board-worklog §2.25
- [ ] **The formal S2** — repeated runs, fan conditions, `--with-inference`,
  decomposing the overhead with TimingBreakdown
- [x] Fixed the model.toml `model_file` relative path bug (2026-08-20, §6 issue
  8)

## M4. Dynamic scheduling — ⬜

- [ ] Least Queue / ECT verified on real hardware
- [ ] Policy comparison (S3)

## M5. Failure recovery — ⬜

- [ ] Health checks verified on real hardware
- [ ] Automatic exclusion / re-admission
- [ ] The retry path verified
- [ ] **Distinguish a board hard reset from an intentional failure** (boot_id)

## M6. Dashboard — ⬜

- [ ] Cluster overview / node view / benchmark view / event timeline
- [ ] SSE live transport
- [ ] Voltage, temperature and frequency display

## M7. Optimization experiments — ⬜

- [ ] **Re-examine the S2 scalability experiment design** ← the premise changed
  with the INT8 results
  - 1.545 Gbps per node (INT8) / 0.829 Gbps (FP16)
  - 4.636 / 2.486 Gbps at three nodes — **both exceed a single 2.5GbE link**
  - Raise the aggregation link to 10G (the §4 topology)
  - See `discuss.md` §8, `RESULTS.md` §8.1

- [ ] Buffer pool
- [ ] CPU profile (checking the preprocessing share)
- [ ] Decide whether to apply io_uring

## M8. Talk release — ⬜

- [ ] The v0.1 tag, README, installation scripts
- [ ] Publish the raw benchmark data
- [ ] Presentation material, demo video, backup video

---

# 3. Benchmark scenarios

**Premise: S0 determines every other scenario's thresholds and cooldown. It
comes first, without exception.**

- [ ] **S0-A** thermal characterisation (fanless) — 3 nodes × 1,800 s
- [ ] **S0-B** thermal characterisation (cooled) — 3 nodes × 1,800 s
- [ ] S1 single-node baseline
- [ ] S2 scalability (1/2/3 nodes)
- [ ] S3 scheduler policy comparison
- [ ] S4 failure handling
- [ ] S5 network implementation comparison
- [ ] S6 input size comparison

146 runs in total, about 23.4 hours. Unattended overnight execution is required.

---

# 4. Infrastructure status

| Item | Status |
|---|---|
| The 3 boards (king/queen/jack) | 🟡 OS, kernel, RKNN, gcc and governor match; eth0 2.5G measured. **queen and jack share an SSH host key (unresolved)** |
| SSH aliases and key auth | ✅ IPs updated (.3/.5/.4/.9), `npuforge-server` added |
| `server` (Rocky 9.4, scheduler and bench) | ✅ **Xeon x2 24T / 16GB / 10G**. Rust and Docker not installed |
| Power 5V 4A × 3 | ✅ verified under sustained load |
| **2.5G/10G switch** | ✅ NEXI NS-S25G10G-N |
| **Inference network bandwidth** | ✅ worker 2.5G / aggregation 10G, 5.11 Gbps measured across three nodes |
| Management/inference network separation | ⬜ sharing a single range; to be decided before M3 |
| Static IPs | ✅ all four static (manual). Router reservation optional |
| Board physical placement | ✅ spread 5.6 °C, no NPU throttling (confirmed 2026-08-11) |
| Cooling (3 fans) | ⬜ not purchased |
| CPU governor | ✅ fixed to `performance`, survives reboot |
| NTP synchronisation | ⚠️ chrony installed, making `server` the server not done |
| Temperature thresholds | ⚠️ draft values (80/90 °C) — reset after S0 |

---

# 5. Purchase list

The equipment blocking M3 (switch, server, 10G NIC) has **all been obtained**
(2026-08-20). What remains is for measurement quality.

| Item | Qty | Priority | Note |
|---|---:|---|---|
| **Identical-model fans** | 3 | medium | 5V USB, same speed. For S0-B |
| Cat6/6a cables (spare) | 2–3 | low | 10G spares. The current link is fine |
| USB power meters | 3 | low | a 5V input, so USB meters work |
| Spare cables and adapters | 1 each | medium | for the talk |

**The power adapters are resolved** (replaced with 5V 4A × 3).

---

# 6. Known issues

| # | Issue | Severity | Status |
|---|---|---|---|
| 1 | ~~`king` runs 19 °C hotter~~ | resolved | did not reproduce under controlled re-measurement (spread 5.6 °C) |
| 2 | The temperature thresholds conflict with the normal operating range | **high** | reset after S0 |
| 3 | No RTC — the clock is wrong immediately after boot | medium | chrony wait logic needed |
| 4 | No current sensor → FPS/Watt cannot be computed | medium | an external USB power meter is needed |
| 5 | Throughput has not bent at 8 threads | low | MAX_THREADS needs widening |
| 7 | Only the board (king) has the Rust toolchain | low | build-only. The binary is built once and deployed |
| 6 | The collected `npu_cores` value is the devfreq count (1) | low | fix the metric definition |
| ~~8~~ | ~~model.toml `model_file` relative path unresolved~~ | resolved | `main.rs` replaces `spec.model_file` with an absolute path before `load_model`. Re-verified with 3-node loading and a bench on a relative-path model.toml (2026-08-20) |
| 9 | The NPU context is not released on node restart | medium | a dead node holds the context and restart gives status=-2. `pkill -9` plus a wait is needed. Graceful shutdown to be reviewed |

---

# 7. Settled headline figures

Measured values that can be quoted in the talk and the documents.

| Item | Value | Source |
|---|---|---|
| SoC | RK3576, 2-core NPU, 6 TOPS | measured |
| RKNN concurrency | **dedicated contexts can run concurrently / sharing a context is forbidden** | when shared, 0 API errors but 200/200 result mismatches |
| FP16 8-thread burst throughput | 70–78 inf/s | measured on 3 nodes |
| **FP16 8-thread sustained throughput** | **84.3 inf/s** | governor=performance, 120 s |
| **INT8 8-thread sustained throughput** | **157.2 inf/s** | governor=performance, **1.86× against FP16** |
| INT8 mean latency | 50.8 ms | −46% against FP16's 94.5 ms |
| CPU governor effect | +7% | ondemand→performance. **A 120-second measurement.** Unverified under sustained load |
| `want_float=0` effect | **INT8 +17.3% / FP16 +15.7%** | output a quarter too (discuss.md §12) |
| **Steady-state throughput (300 s)** | **FP16 59.7 inf/s** | **−27%** against the starting 81.6. CPU throttling |
| (reference) on ondemand | FP16 79.0 / INT8 146.2 | every measurement before 08-11 is on this basis |
| Kernel ioctls per inference | **76 (identical for FP16 and INT8)** | strace; the ceiling is set by time, not count |
| **Peak vs sustained degradation** | **about 10%** | 77.3 → 69.7 |
| Recommended `worker_count` | **8** (`core_mask` unset) | the core_mask sweep |
| Actual contribution of the NPU's 2 cores | **1.51×** (not 2×) | against a control group |
| NPU temperature under sustained load | **67.5–75.8 °C** (3 boards, 8 threads, 15 min, FP16) | controlled measurement 2026-08-11 |
| INT8 accuracy (vs FP16) | detection cells 10/10, classes 100%, box cos 0.997 | detection level on a real board |
| **Shared-context result mismatch** | **100%** (0 API errors) | why a context pool is mandatory |
| Node-to-node thermal spread | **5.6 °C** (no NPU throttling) | concurrent load |
| **CPU thermal downgrade** | A72 2208→**816MHz**, A53 2016→**600MHz** | after 60 s of load. The NPU holds 950 MHz |
| Input voltage under load | minimum 5.05V | measured on all three simultaneously |

> The peak vs sustained gap is a figure absent from vendor spec sheets and one of
> this project's central outputs.
> But the current value is contaminated by desk fan intervention and **has to be
> cleanly re-measured in S0.**
>
> The controlled measurement on 2026-08-11 (15 min × 3 boards) gives sustained
> throughput of **77.7–80.5 inf/s**, higher than the soak's 69.7 inf/s. The soak
> conditions differ (24,000 iterations, longer duration) so they are not compared
> directly. S0 unifies the conditions and settles it.
