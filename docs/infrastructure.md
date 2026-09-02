# NPUDure infrastructure status

*[한국어 원문](infrastructure.ko.md)*

- Document: `infrastructure.md`
- Last updated: 2026-08-20
- Related: `board-worklog.md` (chronological work history),
  `environment-matrix.md` (version pinning)

This document is **a snapshot of the current state.** How that state was reached
is in `board-worklog.md`.

> **The 2026-08-20 rework.** Introducing the 2.5G/10G switch and the 10G server
> cleared every blocker for M3, and the IPs and roles changed substantially. The
> previous arrangement (the dealer laptop + a 1G management network) is retired.
> The story is in `board-worklog.md` §2.23.

---

# 1. Equipment

```text
                    +----------------------------------+
                    | server   192.168.123.9           |
                    | Rocky Linux 9.4 / x86_64         |
                    | Core i7-4790 (4C/8T) / 16GB      |
                    |                                  |
                    | . Scheduler (planned)            |
                    | . Benchmark Client (planned)     |
                    +----------------+-----------------+
                                     | 10GbE (enp1s0)   <- aggregation
                                     |
                    +----------------v-----------------+
                    | NEXI NS-S25G10G-N                |
                    | 2.5G x4 + 10G x2 (all RJ45)      |
                    +-+----+------+------+------+------+
              10G ----+    |2.5G  |2.5G  |2.5G  +--2.5G-- internet (ipTIME)
          dev PC (laptop)  |      |      |
          (1G NIC, unused) |      |      |
                    +------v+ +---v---+ +v------+
                    | king  | | queen | | jack  |
                    |  .3   | |  .5   | |  .4   |
                    | 6 TOPS| |6 TOPS | |6 TOPS |
                    +-------+ +-------+ +-------+
                       Ubuntu 24.04 / RK3576 / aarch64
                       eth0 2.5G each, static
```

| Host | IP | Role | OS | Arch | Switch port |
|---|---|---|---|---|---|
| `server` | 192.168.123.9 | Scheduler / Bench | Rocky Linux 9.4 | x86_64 | **10G (6)** |
| `king` | 192.168.123.3 | NPU Worker | Ubuntu 24.04 | aarch64 | 2.5G (2) |
| `jack` | 192.168.123.4 | NPU Worker | Ubuntu 24.04 | aarch64 | 2.5G (4) |
| `queen` | 192.168.123.5 | NPU Worker | Ubuntu 24.04 | aarch64 | 2.5G (3) |
| dev PC | 192.168.123.26 | writing code / remote operation | Windows | x86_64 | 10G (5) — **1G NIC** |
| internet | — | ipTIME upstream | — | — | 2.5G (1) |

> **Static IPs completed (2026-08-20).** The rework changed the board IPs
> wholesale (`.12/.16/.33` → `.3/.4/.5`), leaving stale SSH aliases unable to
> find the nodes. Rather than router DHCP reservations, **the current IP is
> pinned as NetworkManager static on each host** (host configuration is better
> for measurement reproducibility). **All four are `ipv4.method=manual`**
> (§2.3). `adrs/019-ssh-alias-not-ip.md`.

**`dealer` (the old scheduler, laptop .14) has been removed.** No response. Its
roles (scheduler and bench) moved to `server`. The model conversion Docker also
lived on dealer, so the conversion environment is due for rebuilding (§6).
Though the model is already converted, so it is not needed immediately.

The dev PC is plugged into the switch's 10G port (5) but **its NIC is 1G
(currently negotiating 100 Mb/s) and cannot do 10G.** The bench client runs on
`server`, not on the dev PC.

---

# 2. Access

## 2.1 SSH aliases

Registered in the dev PC's `~/.ssh/config`. **The IP lives only here**
(`adrs/019-ssh-alias-not-ip.md`).

```text
npuforge-k        -> pi@192.168.123.3     (king)
npuforge-q        -> pi@192.168.123.5     (queen)
npuforge-j        -> pi@192.168.123.4     (jack)
npuforge-server   -> root@192.168.123.9   (server)
```

All connect without a password using the `~/.ssh/id_ed25519_npuforge` key.

> This key is for automation only and has no passphrase. Do not expose it in a
> public repository or on an untrusted network.

## 2.2 Privilege escalation

| Host | Account | sudo | Note |
|---|---|---|---|
| king / queen / jack | `pi` | passed via `NPUFORGE_SUDO_PASS` | `printf '%s\n' "$NPUFORGE_SUDO_PASS" \| sudo -S -p "" <cmd>` |
| server | `root` | not needed (root directly) | the automation key is in root's `authorized_keys` |

`sudo -S` consumes stdin's first line as the password. File contents cannot be
piped in, so writing a file goes through a temporary file.

### 2.2.1 Board credentials are the vendor defaults — a deliberate choice

The board accounts and sudo password are **the OS image's vendor defaults,
unchanged.** Writing that down openly was judged better than hiding it.

| | |
|---|---|
| Premise | The boards are on the private `192.168.123.0/24` range, behind NAT. No inbound forwarding |
| Keeping the defaults | Vendor defaults are **already public information.** At the very least this tells nobody anything new |
| Why not change them | Using a custom value **creates a secret that did not exist.** If that value leaks anywhere — a document, a history, a photograph — it exposes a password **pattern**, and that is information that spreads beyond this lab |

> **This judgement changes when the conditions do.** The moment port 22 is
> forwarded externally or the boards sit on a non-isolated network, the defaults
> become a problem immediately. Like an exclusion, this decision carries **under
> what conditions.**

```bash
S() { printf '%s\n' "$NPUFORGE_SUDO_PASS" | sudo -S -p "" "$@"; }
cat > /tmp/f.new <<'H'
...
H
S cp /tmp/f.new /etc/target       # printf "text" | S tee ... does not work
```

Remote execution pitfalls (background startup, process counting) are in
`adrs/017-remote-exec-pitfalls-library.md`.

## 2.3 Static IPs (host-side static)

To stop DHCP reassignment changing the IPs, **the current IP is pinned as
NetworkManager static on each host.** Host configuration is used rather than
router (ipTIME) DHCP reservations because the settings survive a router change,
which is better for measurement reproducibility. **Since the current IP is
pinned as-is, SSH sessions are not dropped.**

Common parameters: gateway `192.168.123.254`, DNS `210.94.0.73 210.220.163.82`,
prefix `/24`. All managed by NetworkManager (not netplan or networkd).

```bash
# server (root, connection enp1s0) - done
nmcli con mod enp1s0 ipv4.method manual \
  ipv4.addresses 192.168.123.9/24 ipv4.gateway 192.168.123.254 \
  ipv4.dns "210.94.0.73 210.220.163.82"
nmcli con up enp1s0

# boards (pi/sudo, connection 'Wired connection 1', eth0) - done (2026-08-20)
#   king .3 / queen .5 / jack .4. Same IPs, so SSH held; external reachability confirmed
```

> ⚠️ **Beware DHCP pool collisions.** If `.3/.4/.5/.9` fall inside the ipTIME
> DHCP pool, the router can lease those addresses to another device (it does not
> know about host-side statics). Full avoidance means excluding those addresses
> from the pool in the ipTIME UI. The risk is low on a small home LAN but it is
> residual.

## 2.4 The sudo password file

The sudo password for board automation is passed via the dev PC's local
`~/.npuforge/sudo-pass` (chmod 600) or the `NPUFORGE_SUDO_PASS` environment
variable. It is not put in the repository. `preflight-check.sh` and the
deployment scripts read that path.

---

# 3. Software status

## 3.1 Nodes (king / queen / jack)

| Item | Value | Matching across 3 |
|---|---|---|
| SoC | Rockchip RK3576 | ✓ |
| NPU | 2 cores, 300–950 MHz, IOMMU enabled | ✓ |
| RKNN Runtime | 2.3.0 (`librknnrt.so` SHA-256 identical) | ✓ |
| RKNPU Driver | v0.9.8 | ✓ |
| Kernel | 6.1.141 | ✓ |
| glibc | 2.39 | ✓ |
| RAM / eMMC | 4GB / 64GB | ✓ |
| Ubuntu patch level | 24.04.4 | ✓ |
| gcc | 13.3.0-6ubuntu2~24.04.1 | ✓ |
| CPU Governor | **`performance`** | ✓ survives reboot |
| eth0 link | **2.5G (2500 Mb/s)** | ✓ measured 2026-08-20 |
| **SSH host key** | **identical on queen and jack** | ✗ **unresolved** |
| `.rknn` model (FP16) | `459602ea…` deployed to all 3 | ✓ |
| `.rknn` model (INT8) | `dba155d2…` **on `king` only** | ✗ needs deploying |
| Rust toolchain | 1.97.1 | **on `king` only.** For builds |
| C measurement tools | `~/npuforge-rknn-test/` | ✓ hashes identical |

**The SSH host keys are identical on queen and jack.** The two boards cannot be
told apart cryptographically, so a changed IP attaches you to the wrong board
without a warning. Since DHCP means the IPs do change (§1), the risk is
significant. The remediation commands are in `TODO.md` §1.2.

`preflight-check.sh` confirms the above items match before every measurement.

## 3.2 server (192.168.123.9)

| Item | Value |
|---|---|
| OS | Rocky Linux 9.4 (Blue Onyx), kernel 5.14.0-427.13.1.el9_4 |
| Motherboard | ASUS H81M-K (H81 chipset) |
| CPU / RAM | **Core i7-4790 (4C/8T, 3.6–4.0 GHz)** / **16GB DDR3-1600 non-ECC** |
| Disk | ST2000VN004 2TB, root LVM 70GB (65GB free) |
| NIC | `enp1s0` **Intel X550T 10GBASE-T, 10G full measured** (2026-08-26), driver `ixgbe` |
| NIC slot | `PCIEX16_1` (direct to CPU). **Operating at PCIe 2.0 x4** |
| Basis for the slot limit | root port `00:01.0`'s `LnkCap: Speed 5GT/s, Width x16` — **the motherboard's x16 slot is itself capped at PCIe 2.0.** The slot decides, not the card (`LnkCap 8GT/s x4`). Nothing to be done |
| Is it a bottleneck | **No.** PCIe 2.0 x4 = about 16 Gbps per direction. Real use is ~4.6 Gbps per direction across three nodes — 3× headroom |
| Full inventory | new server [`hosts/server-i7-4790-20260826.md`](hosts/server-i7-4790-20260826.md) · old server [`hosts/server-xeon-e5-2630l-20260826.md`](hosts/server-xeon-e5-2630l-20260826.md) |
| Firewall | firewalld active, zone `public`. gRPC ports need opening (before measuring) |
| Build toolchain | **rust/cargo 1.92, gcc 11.5, protoc 3.14, git** (installed 2026-08-20) |
| Docker | not installed — to be set up if model conversion is needed |

> protoc is not in Rocky 9's default repositories; **the CRB repository**
> (`dnf config-manager --set-enabled crb`) has to be enabled for
> `protobuf-compiler` to appear. tonic-build 0.12 requires the system protoc.

**Two of dealer's (the laptop's) constraints are resolved by this server.**

1. **RAM 3GB → 16GB.** The concern about scheduler RSS (relaying payloads) is
   greatly eased. `environment-matrix.md` §10.1,
   `adrs/003-central-simple-scheduler.md`
2. **1GbE → 10GbE.** Aggregation bandwidth secured. Measured in §4.

**The scheduler (x86_64) is built natively on server.** With MSRV 1.85 < dnf
rust 1.92, it builds on the stable channel. Sources are handed over as a
`git archive` tarball via scp (server cannot reach foxden directly; github is
fine). The node (aarch64) is still built on king. Windows→Linux cross-building
is not used because of linker problems.

## 3.2.1 Server replacement (2026-08-26) — the baseline dropped 7.5%

The old server (Xeon E5-2630L ×2, **24 threads**) was physically replaced and
things moved to a spare desktop (i7-4790, **8 threads**). **Only the scheduler
host changed; the three nodes, switch, model and binaries are unchanged.**

| | Old server (to ~2026-08-24) | New server (2026-08-26–) |
|---|---|---|
| CPU | Xeon E5-2630L ×2 · 24T · **2.0–2.5 GHz** | Core i7-4790 · 8T · 3.6–4.0 GHz |
| RAM | 16GB | 16GB DDR3-1600 |
| NIC | **Intel X550T** `enp4s0` | **the same card** `enp1s0` (PCIe 2.0 x4) |
| **Baseline throughput** | **~391 inf/s** | **~360 inf/s** (3 runs: 360.5 / 362.5 / 357.2) |
| Round-trip p50 | ~86 ms | ~93 ms |
| Node spread | ~1.02× | ~1.07× |
| Error rate | 0 | 0 |

> **The 10G NIC is the same physical card.** There is only one Intel X550T; it
> was pulled from the old server and plugged into the new one. So **the
> hardware of the 10G path is identical across both measurements** — the NIC is
> a controlled variable and what changed is only the host (CPU, motherboard,
> PCIe slot). That narrows the verdict below accordingly.
>
> **Which link that card negotiated on each host was confirmed on 2026-08-26 by
> powering the old server back up** — the slot's capability remains after the
> card is removed.
>
> | | Old server (R620) | New server (H81M-K) |
> |---|---|---|
> | Slot generation | **PCIe 3.0** (`LnkCap 8GT/s`) | PCIe 2.0 (`LnkCap 5GT/s`) |
> | X550T link | 8GT/s × x4 | 5GT/s × x4 |
> | Bandwidth per direction | **about 32 Gbps** | about 16 Gbps |
>
> **The link bandwidth halved. It is still not a bottleneck** — real three-node
> use is ~4.6 Gbps per direction, so even 16 Gbps is 3.5× headroom. This is now
> a measurement rather than an estimate.
> → [`hosts/server-xeon-e5-2630l-20260826.md`](hosts/server-xeon-e5-2630l-20260826.md)

### Cause — the scheduler host narrowed to CPU

Server CPU utilisation during measurement is **82.2%** (across 8 threads).

```text
scheduler          45.3%  ~ 3.6 cores
other (bench+kernel) 36.9%  ~ 2.9 cores
────────────────────────────────
total              82.2%
```

**The bench client runs on the same host as the scheduler.** On the old server
the same work was ~27% of 24 threads; on the new server it is 82% of 8.

Where the loss sits supports this. The node side is unchanged (NPU inference p50
28.35 ms, distribution an even 33.3%, temperature 53–57 °C), and the scheduler's
internal queue is empty too, with `scheduler_queue` 0.00 ms and
`scheduler_route` 0.01 ms. All the added time is in the transport sections
(`network_to_node` / `network_to_client`, p50 24.2 ms each) — not an application
queue but **CPU contention on the host.**

> **The PCIe downgrade is not the cause.** `LnkSta 5GT/s x4` is 16 Gbps per
> direction, 3× the headroom over real use (~4.6 Gbps). It is a hardware limit
> arising from H81M-K's x16 slot being PCIe 2.0, it cannot be remedied, and it
> does not need to be.

> **Raw data.** The bench JSON for those 3 runs is in
> [`../results/baseline-20260826-althost/`](../results/baseline-20260826-althost/).
> The `-althost` suffix keeps `count-runs.sh` from adding them to the 421 and
> counts them separately.

### Effect on existing measurements — none

**All 421 measurements were taken on the old server and those values stand as
recorded.** The numbers are not retroactively edited. The new server's values
are written here separately as "reproduction figures on a different scheduler
host".

That said, S3.9a's verdict — **the scheduler is not a resource bottleneck** —
**has turned out to have been conditional.** That verdict held on a 24-thread
host. It does not hold at 8.

> Exactly the principle in the experiment ledger §4. **Exclusions are
> conditional.** A candidate once excluded reopens when conditions change. A
> verdict has to carry "under what conditions".

If measurement continues on the new server, **its values are not compared
directly with the old server's.** Where a comparison is needed, a baseline is
re-laid on the new server and relative comparison is done on top of that.

## 3.3 Distribution differences

```text
server  Rocky Linux 9.4   glibc 2.34   dnf   x86_64
nodes   Ubuntu 24.04      glibc 2.39   apt   aarch64
```

The node binary is aarch64, so it is built natively on `king` and deployed to
all three nodes (all three boards are on glibc 2.39). The scheduler is x86_64
and therefore a separate build.

---

# 4. Network

## 4.1 Current (rework completed 2026-08-20)

```text
                server (10G) -+
                              +-- NS-S25G10G-N --+-- king  (2.5G)
       dev PC (10G port/1G NIC)                  +-- queen (2.5G)
                                                 +-- jack  (2.5G)
                                                 \-- internet (2.5G, ipTIME)
```

- **Worker links 2.5G, aggregation (server) 10G.** As ADR-014 designed.
- **The management and inference networks are still not separated.** Everything
  is on the single `192.168.123.0/24` range and the boards' eth1 is unused.
  VLAN/subnet separation to prevent measurement contamination is to be decided
  before M3's main measurements.

## 4.2 Bandwidth measurements (2026-08-20)

| Measurement | Value | Tool | Meaning |
|---|---:|---|---|
| server enp1s0 negotiation | 10000 Mb/s full | ethtool | 10G link confirmed |
| Single king→server | **2.34 Gbps** | iperf3 | the effective 2.5G ceiling |
| **3 nodes concurrently →server** | **1.70 each, 5.11 Gbps total** | nc | **aggregation is not a bottleneck** |

Under concurrent three-node transmission the three streams **stayed even (213
MB/s each)**. Had the server been the bottleneck the total would have been cut
somewhere, and it was not. It comfortably accommodates the INT8 three-node RX
target of **4.60 Gbps** (`RESULTS.md` §8.1).

> The individual 1.70 Gbps being below the link ceiling (2.34) is an nc /
> single-core board CPU limit, not a switch or server limit. Actual M3 traffic
> is gRPC inference traffic, so this figure is used only to verify "does the
> infrastructure absorb 4.6 Gbps aggregate" — and the answer is yes.

## 4.3 Link speed gets checked every time

Faulty cables lowering the negotiated speed has happened repeatedly (the old
dealer at 100 Mb/s, the current dev PC at 100 Mb/s). 10GBASE-T requires Cat6/6a,
and Cat5e silently falls back to 2.5G/5G. Left unchecked you measure the cable
rather than the NPU.

```bash
ssh npuforge-server 'ethtool enp1s0 | grep Speed'
for h in npuforge-k npuforge-q npuforge-j; do
  ssh "$h" 'printf "%s eth0=%s\n" "$(hostname)" "$(cat /sys/class/net/eth0/speed)"'
done
```

---

# 5. Purchases needed

The equipment that was blocking M3 has **all been obtained.**

| Item | Status |
|---|---|
| ~~2.5G/10G switch~~ | ✅ NEXI NS-S25G10G-N (2.5G×4 + 10G×2) |
| ~~server with a PCIe slot~~ | ✅ i7-4790 / 16GB / Rocky 9.4 (replaced 2026-08-26) |
| ~~10G NIC~~ | ✅ Intel X550T `enp1s0` 10GBASE-T |
| ~~10G cable~~ | ✅ 10G full negotiation confirmed (RJ45, not DAC) |

The remaining purchases are for measurement quality and do not block starting M3.

| Item | Qty | Priority | Basis |
|---|---|---|---|
| Identical-model fans | 3 | medium | for the S0-B cooling comparison |
| USB power meters | 3 | low | for computing FPS/Watt |
| Cat6/6a cables (spares) | 2–3 | low | spares for the 10G link. The current link is fine |

Permanent cooling equipment is not on the list. Fanless is kept and thermal
throttling is treated as something to measure
(`adrs/013-fanless-thermal-as-measurement.md`).

---

# 6. Open items

| # | Item | Status | Blocker |
|---|---|---|---|
| 1 | ~~Static IP pinning~~ | ✅ all four manual (2026-08-20) | — |
| 2 | **Duplicate SSH host keys (queen, jack)** | not done | none. Commands in `TODO.md` §1.2 |
| 3 | **Deploy the INT8 model to queen and jack** | not done | none |
| 4 | **Settle the scheduler build/deploy path** | undecided | no Rust on server (§3.2) |
| 5 | **Open the gRPC port in server's firewall** | not done | before measuring. firewalld public zone |
| 6 | Rebuild the model conversion environment | on hold | dealer is gone. Not urgent since the model is already converted |
| 7 | Separate management and inference networks | undecided | before M3's main measurements |
| 8 | Record measured TX/RX (inference traffic) | not measured | after the node software is up |
| 9 | S0 thermal characterisation (30 min × 2 conditions) | not run | needs 3 fans (for S0-B) |

**Per-host MAC / static IP** (the actual MACs confirmed in §1). Use this table if
router reservations are done alongside:

```text
king    22-94-FF-34-46-B1  ->  192.168.123.3
jack    62-CE-3B-B6-E4-41  ->  192.168.123.4
queen   7E-D8-D7-40-45-82  ->  192.168.123.5
server  6C-B3-11-13-2F-38  ->  192.168.123.9
```

Resolved on 2026-08-20: the 2.5G/10G switch, the 10G scheduler server, the 10G
NIC and cable, measured aggregation bandwidth, and dealer's 3GB RAM constraint.
Previously resolved: RKNN thread-safety (context sharing forbidden), model
conversion (FP16 and INT8), calibration (200 COCO images), CPU governor
(`performance`), board placement variance, and OS patch level.
