# NPUDure board work log

*[한국어 원문](board-worklog.ko.md)*

- Document: `board-worklog.md`
- Subject: NanoPi R76S × 3 (`king` / `queen` / `jack`)
- Purpose: record every change made to the boards, chronologically

---

# 0. This document's rules

Commands run on the boards and their results are **appended
chronologically**. Existing entries are not edited.

There are three reasons for keeping it.

1. **Reproducibility.** Setting the boards up again, or adding a fourth node,
   should be possible by following this document alone.
2. **Cause tracing.** When benchmark results differ per node, this is where you
   check what was applied differently to the three boards.
3. **Open-source publication.** An external user has to be able to build the
   same environment.

Each entry leaves the following.

```text
date / target node / command run / result / basis for the judgement
```

**Irreversible changes** (package upgrades, kernel replacement, partition
operations) are flagged separately before execution and their approval recorded.

---

# 1. Node names

The labels physically attached to the boards are used as-is.

| Label | hostname | Node ID | Management IP | SSH alias |
|---|---|---|---|---|
| K | `king` | `king` | `192.168.123.12` | `npuforge-k` |
| Q | `queen` | `queen` | `192.168.123.16` | `npuforge-q` |
| J | `jack` | `jack` | `192.168.123.33` | `npuforge-j` |

Scheduler host (the development PC): `192.168.123.26`

---

# 2. 2026-08-07

## 2.1 Securing SSH access

**Situation.** All three boards moved to `192.168.123.0/24`, the same range as
the PC (`192.168.123.26`). All three confirmed responding to ping and tcp/22.

**Problem.** `ssh-copy-id` failed immediately on all three.

```text
Permission denied, please try again.   (twice per host, all three, immediately)
```

**Cause.** The password was not wrong — **there was no TTY.** SSH reads the
password from the controlling terminal (`/dev/tty`), not stdin. An automated
environment has no TTY, so the prompt could not appear and it failed
immediately on EOF. The pattern of exactly two attempts per host, with all three
finishing at once, is the evidence.

**Action.** OpenSSH 9.7's `SSH_ASKPASS_REQUIRE=force` was used to pass the
password without a TTY.

```bash
ASKPASS=$(mktemp)
printf '#!/bin/sh\nprintf "%%s\\n" "$NPUFORGE_SUDO_PASS"\n' > "$ASKPASS"; chmod 700 "$ASKPASS"
SSH_ASKPASS="$ASKPASS" SSH_ASKPASS_REQUIRE=force DISPLAY=dummy \
  ssh-copy-id -i ~/.ssh/id_ed25519_npuforge.pub npuforge-k
```

The helper file was deleted with `shred -u` afterwards.

**Result.** Key authentication succeeded on all three. The account is `pi`.

**PC-side setup.**

- A dedicated key generated: `~/.ssh/id_ed25519_npuforge` (no passphrase, for
  automation)
- `npuforge-k` / `npuforge-q` / `npuforge-j` aliases added to `~/.ssh/config`

> This key is for automation only and has no passphrase. Keep it from being
> exposed in a public repository or on an untrusted network.

## 2.2 Collecting the hardware specification

**Command.** `scripts/collect-node-info.sh` run remotely on all three.

```bash
for pair in "k:npuforge-k" "q:npuforge-q" "j:npuforge-j"; do
  name="${pair%%:*}"; host="${pair##*:}"
  ssh "$host" 'bash -s' < scripts/collect-node-info.sh > "benchmarks/node-info/${name}.txt"
done
```

**Raw output.** `benchmarks/node-info/{k,q,j}.txt` (66 lines each)

**The settled specification.** Details in `environment-matrix.md` §2.1.

```text
board   FriendlyElec NanoPi R76S / friendlyelec,nanopi-r76s rockchip,rk3576
CPU     8 cores - little 2.016GHz(policy0) + big 2.208GHz(policy4)
RAM     4GB LPDDR4X (3,997,848 kB)
eMMC    64GB (rootfs 50G free)
NPU     2 cores (Core0, Core1), 300-950MHz, IOMMU enabled
        RKNPU driver v0.9.8
RKNN    Runtime 2.3.0 (c949ad889d@2024-11-07T11:35:33)
        librknnrt.so SHA-256 identical on all three
OS      Ubuntu 24.04, kernel 6.1.141, glibc 2.39
sensors 6 thermal zones (soc / bigcore / little-core / ddr / npu / gpu)
```

**Important.** The NPU has **2 cores.** The RK3588 has 3, so RK3588-based
`core_mask` examples cannot be used as-is.

`rknn_api.h`'s `rknn_core_mask` enum defines up to three cores
(`RKNN_NPU_CORE_2`), but on RK3576 the usable ones are `CORE_0`, `CORE_1`,
`CORE_0_1`, `CORE_AUTO` and `CORE_ALL`.

## 2.3 Confirming the NIC specification

**Context.** The initial collection showed `eth1` at `speed=1000`, which could
be mistaken for a 1G port.

**Commands.**

```bash
sudo apt-get install -y ethtool
sudo ethtool -i eth0 ; sudo ethtool eth0
sudo ethtool -i eth1 ; sudo ethtool eth1
```

**Result. Both ports are 2.5G.**

| Item | eth0 | eth1 |
|---|---|---|
| Driver | `r8125` 9.010.01-NAPI | `r8125` 9.010.01-NAPI |
| PCIe bus | `0001:21:00.0` | `0000:01:00.0` |
| Supported link modes | 10/100/1000/**2500** baseT | 10/100/1000/**2500** baseT |
| Current link | none (down) | 1000Mb/s Full |

`eth1` being at 1000Mb/s is **the result of negotiating with a 1G hub**, not a
limit of the port.

The two ports are on different PCIe buses and do not share bandwidth. That
favours separating management from inference networks.

**Decision.**

```text
eth1 -> management network (currently the 1G hub, 192.168.123.0/24)
eth0 -> inference network (when the 2.5G switch arrives, 10.20.0.0/24)
```

`eth0` is free on all three, so it is used exclusively for the inference
network.

## 2.4 Changing the hostnames

**Before.**

| Node | hostname |
|---|---|
| K | `NanoPi-R76S` |
| Q | `NanoPi-R76S` |
| J | `localhost.localdomain` |

K and Q were identical, making them indistinguishable in logs and the
dashboard.

**Commands.**

```bash
sudo hostnamectl set-hostname <king|queen|jack>
sudo sed -i "s/^127\.0\.1\.1.*/127.0.1.1\t<new>/" /etc/hosts
```

**Result.** Changed to `king` / `queen` / `jack`.

### Incidental finding: jack's `/etc/hosts` was empty

`jack`'s `/etc/hosts` was **0 bytes.** That is why its hostname was
`localhost.localdomain`.

It was restored to identical content using king's file as the reference.

```text
127.0.0.1	localhost
::1		localhost ip6-localhost ip6-loopback
ff02::1		ip6-allnodes
ff02::2		ip6-allrouters

127.0.1.1	jack
```

**Judgement.** The three boards are **not perfect clones.** The missing
`/etc/hosts` appearing together with the Ubuntu patch level difference (§2.5)
suggests jack may have been set up at a different time or by a different route.

### A script pitfall found during the work

Piping file contents into a helper function that itself pipes the sudo password
causes a collision.

```bash
S() { printf '%s\n' "$NPUFORGE_SUDO_PASS" | sudo -S -p "" "$@"; }

printf "text\n" | S tee -a /etc/hosts    # does not work
```

`sudo -S` consumes stdin's first line as the password, so the following command
gets EOF. Writing a file uses this instead.

```bash
cat > /tmp/file.new <<'EOF'
...
EOF
printf '%s\n' "$NPUFORGE_SUDO_PASS" | sudo -S -p "" cp /tmp/file.new /etc/target
```

## 2.5 Discovered node mismatches (unresolved)

The three nodes are supposed to be on the "same OS image"
(`02-HARDWARE-SETUP.md` §5.1). The following are out of line.

| # | Item | king | queen | jack | Risk |
|---|---|---|---|---|---|
| 1 | Ubuntu patch level | 24.04.**3** | 24.04.4 | 24.04.4 | library differences appearing as per-node performance variance |
| 2 | gcc | `~24.04` | `~24.04.1` | `~24.04.1` | as above |
| 3 | Unapplied updates | 374 | 280 | 279 | as above |
| 4 | SSH host key | identical on all three (`<redacted-fingerprint>`) | | | nodes indistinguishable, MITM undetectable |
| 5 | CPU Governor | `ondemand` | `ondemand` | `ondemand` | frequency variation reduces measurement reproducibility |

**Matching items** (no problem): kernel 6.1.141, glibc 2.39, Python 3.12.3, RKNN
Runtime 2.3.0 and the `librknnrt.so` SHA-256, RKNPU driver v0.9.8, 2-core NPU,
4GB RAM, 64GB eMMC.

### ⚠️ No kernel upgrades

Kernel `6.1.141` is the FriendlyElec BSP kernel and **RKNPU driver v0.9.8 is
tied to it.**

If `apt upgrade` replaces the kernel, the NPU may stop working. Always hold the
kernel packages when synchronising packages.

```bash
sudo apt-mark hold linux-image-* linux-headers-* linux-modules-*
```

This is cumbersome to undo, so it is **executed after approval.** Currently not
done.

## 2.6 Board software status

| Item | Status |
|---|---|
| `librknnrt.so` | `/usr/lib/librknnrt.so` (2.3.0) |
| `rknn_api.h` | `/usr/include/rknn_api.h` |
| `rknn_matmul_api.h` | installed |
| `rknn_custom_op.h` | installed |
| `rknn_server` | `/usr/bin/rknn_server` (for Toolkit2 connected debugging) |
| `.rknn` model files | **none** — conversion needed |
| gcc | 13.3.0 |
| rustc | not installed (normal, since cross-compilation is used) |
| ethtool | installed 2026-08-07 (king only; queen/jack not) |

With `rknn_server` present, RKNN-Toolkit2's connected mode can call the board's
NPU directly from the PC to verify a model.

---

## 2.7 Verifying the C wrapper on real hardware

**Context.** `crates/npuforge-rknn/native/rknn_wrapper.c` was written from the
RKNN API documentation alone and was unverified on real hardware. That fact was
stated at the top of the file.

**Verification method.** The actual signatures were extracted from
`rknn_api.h`, compared, and then compiled directly on the board.

```bash
scp crates/npuforge-rknn/native/rknn_wrapper.{c,h} npuforge-k:~/npuforge-rknn-test/
ssh npuforge-k 'cd ~/npuforge-rknn-test && gcc -c -Wall -Wextra -O2 rknn_wrapper.c -o rknn_wrapper.o'
```

**Result. Compiled without warnings.** The signatures written matched the real
headers.

| Item | Result |
|---|---|
| `rknn_init(rknn_context*, void*, uint32_t, uint32_t, rknn_init_extend*)` | matches |
| `rknn_query(rknn_context, rknn_query_cmd, void*, uint32_t)` | matches |
| `rknn_inputs_set(rknn_context, uint32_t, rknn_input[])` | matches |
| `rknn_run(rknn_context, rknn_run_extend*)` | matches |
| `rknn_outputs_get(rknn_context, uint32_t, rknn_output[], rknn_output_extend*)` | matches |
| `rknn_outputs_release(rknn_context, uint32_t, rknn_output[])` | matches |
| `rknn_input` fields (`index/buf/size/pass_through/type/fmt`) | matches |
| `rknn_output` fields (`want_float/is_prealloc/index/buf/size`) | matches |
| `rknn_sdk_version` (`api_version[256]`, `drv_version[256]`) | matches |
| `rknn_context` | `uint64_t` (aarch64) |
| `RKNN_SUCC` | 0 |

**Additional finding.** `rknn_set_core_mask(rknn_context, rknn_core_mask)`
exists. The `rknn_core_mask` enum defines up to three cores, but RK3576 has two,
so only `CORE_0`, `CORE_1`, `CORE_0_1`, `CORE_AUTO` and `CORE_ALL` are valid.

**Unresolved.** `npf_rknn_get_runtime_version()` was written to call
`rknn_query` without a context, and whether that call actually succeeds can only
be confirmed with a model present. If it fails, it changes to creating a
temporary context at node startup, querying, and caching the result.

## 2.8 Writing the thread-safety test program

**File.** `crates/npuforge-rknn/native/thread_safety_test.c`

**Build confirmed.**

```bash
gcc -O2 -Wall -Wextra -o thread_safety_test thread_safety_test.c -lrknnrt -lpthread
# succeeded without warnings, 71,888 bytes
```

**Verification scenarios.**

| # | Configuration | What it checks |
|---|---|---|
| baseline | 1 thread, dedicated context | single-thread throughput |
| 1 | 2 threads, **shared context** | whether concurrent calls on one context are possible |
| 2 | 2 threads, dedicated contexts, `CORE_AUTO` | whether dedicated contexts parallelise |
| 3 | 2 threads, dedicated contexts, `CORE_0` / `CORE_1` separated | the effect of explicit core separation |
| 4 | 4 threads (more than the 2 cores) | the counterproductive effect of excess workers |

**Decision criteria.**

```text
scenario 1 with err > 0        -> concurrent calls on one context impossible
                                  serialization with a dedicated worker thread per model needed
scenario 2 ~2x the baseline    -> 2-way parallelism with dedicated contexts, worker_count = 2
scenario 2 ~1x                 -> serialization inside the runtime, keep worker_count = 1
scenario 3 > scenario 2        -> explicit core separation is effective
scenario 4 < scenario 2        -> more workers than cores is counterproductive
```

**⛔ Execution deferred. There is no model file.**

There is not a single `.rknn` file on the boards. The program is ready and can be
run the moment a model exists.

```bash
ssh npuforge-k 'cd ~/npuforge-rknn-test && ./thread_safety_test model.rknn 50'
```

### Routes to obtaining a model

| Route | Possible | Note |
|---|---|---|
| Download on the board | ✗ | `curl` and `wget` not installed |
| Convert on the board | ✗ | RKNN-Toolkit2 is x86_64 Linux only |
| **Convert in the PC's WSL2** | **✓** | WSL2 Ubuntu confirmed (currently Stopped) |
| `rknn_server` connected mode | ✓ | Toolkit2 calls the board's NPU directly from the PC |

With `rknn_server` installed on the boards, once Toolkit2 is set up the board's
NPU can be called remotely from the PC to verify a model on the spot.

## 2.9 Preparing the node consistency script (not run)

**File.** `scripts/fix-node-consistency.sh`

It defaults to a DRY RUN and requires `--apply` to actually execute. `--only`
splits the stages.

| Stage | `--only` value | Content | Risk |
|---|---|---|---|
| 1 | `kernelhold` | `apt-mark hold` the kernel packages | low |
| 2 | `hostkeys` | regenerate SSH host keys + clean up the PC's `known_hosts` | low |
| 3 | `packages` | install base packages (curl, ethtool, iperf3, chrony and so on) | low |
| 4 | `chrony` | enable time synchronisation | low |
| 5 | `upgrade` | package upgrade (24.04.3 → 24.04.4) | **high** |
| 6 | `governor` | CPU Governor → `performance` | medium |

**Safety measures.**

- Stage 5 first checks whether the kernel is held and aborts if it is not
- Stage 6 raises heat output, so applying it after the S0 thermal measurement is
  recommended
- A DRY RUN confirmed connectivity to all three and the stage output

**Recommended execution order.**

```bash
./scripts/fix-node-consistency.sh --apply --only kernelhold
./scripts/fix-node-consistency.sh --apply --only hostkeys
./scripts/fix-node-consistency.sh --apply --only packages,chrony
./scripts/fix-node-consistency.sh --apply --only upgrade     # run on its own
# (after the S0 measurement)
./scripts/fix-node-consistency.sh --apply --only governor
```

Always check the following after upgrading:

```bash
ssh npuforge-k 'uname -r'                                              # still 6.1.141?
ssh npuforge-k 'printf "$NPUFORGE_SUDO_PASS\n" | sudo -S cat /sys/kernel/debug/rknpu/version'  # NPU alive?
ssh npuforge-k 'sha256sum /usr/lib/librknnrt.so'                       # still 73993ed4...?
```

---

# 3. Outstanding work

| # | Task | Status | Note |
|---|---|---|---|
| 1 | RKNN thread-safety verification | planned | decides `worker_count`. Needs a model file |
| 2 | Verify `rknn_wrapper.c` against the real headers | planned | written unverified |
| 3 | Resolve the node mismatches (§2.5) | script prepared, awaiting approval | kernel hold mandatory |
| 4 | Regenerate the SSH host keys | script prepared, awaiting approval | |
| 5 | CPU Governor → `performance` | applied just before benchmarking | |
| 6 | Install the base packages | not done | `02-HARDWARE-SETUP.md` §5.2 |
| 7 | Configure the inference network (`eth0`, 10.20.0.0/24) | after the 2.5G switch arrives | |
| 8 | Build the model conversion environment | not done | match Toolkit2 to Runtime 2.3.0 |

## 3.1 Next step: obtaining a model

Thread-safety verification (1) is blocked on the model file, and the model is
also a prerequisite for every other real-hardware task. So this is the top
priority.

```text
PC WSL2 (Ubuntu, currently Stopped)
  -> install rknn-toolkit2==2.3.0        <- matched to Runtime 2.3.0
  -> obtain the YOLOv8n ONNX
  -> rknn.config(target_platform='rk3576')
  -> produce yolov8n.rknn
  -> scp to the 3 nodes + verify SHA-256
  -> run thread_safety_test
  -> record in environment-matrix.md §3.1, §6
```

**Caution.** If the Toolkit version is higher than the Runtime, converted models
may fail to load. Try `rknn-toolkit2==2.3.0` first.

---

## 2.10 Measuring the scheduler host (the laptop)

**Subject.** A Samsung 370E5J-series old laptop, `192.168.123.14`

**Results.** Details in `environment-matrix.md` §4.2.

```text
CPU     Intel i7-4712MQ (Haswell, 4C/8T @2.30GHz)
RAM     3.5GB (1.8GB available)      <- less than a node (4GB)
NIC     RTL8111/8168 (r8169), 1GbE ceiling. No 2.5G
USB     Bus 004 = USB 3.0 (5000M, 4 ports). The rest are USB 2.0
TB      none
Docker  installed
arch    x86_64
```

### The 100Mb/s link problem (resolved)

The initial measurement had negotiated `Speed: 100Mb/s`. The port supports
`1000baseT/Full`, so it was a physical layer problem.

Replacing the cable **normalised it to 1000Mb/s.**

**Impact analysis.** Left alone, at 100KB JPEGs the link would have saturated at
about 125 FPS, and we would have measured the cable rather than NPU scaling
efficiency. The three boards were at 1000Mb/s from the start, so the cause was
the laptop's cable rather than the hub.

**Follow-up.** A procedure for checking link speed before every experiment goes
into the benchmark script.

```bash
ethtool enp3s0 | grep Speed
```

### Verdict

| Role | Verdict | Basis |
|---|---|---|
| Model conversion | **suitable** | x86_64 Linux + Docker |
| Development scheduler (M2–M5) | **sufficient** | link speed is unrelated to functional correctness |
| Official benchmarks (JPEG) | **conditionally suitable** | judge after confirming measured FPS |
| Official benchmarks (raw RGB, S6) | **unsuitable** | exceeds 1GbE |

**Buying a 2.5G adapter is deferred.** Without knowing the actual per-node FPS,
the need cannot be judged. Decided after the S0/S1 measurements.

Assuming 40 FPS per node, 3 nodes at 120 FPS × 100KB ≈ 96 Mbps leaves headroom
on 1GbE. Judging by measurement also matches this project's approach.

**3.5GB of RAM is a more real constraint than the NIC.** The response is not a
hardware purchase but an operational policy — during official measurements,
Prometheus and the dashboard are stopped and `npuforge-bench` records only raw
JSONL.

### Unconfirmed items

```bash
cat /etc/os-release      # the prompt is [root@localhost ~]# - the distribution needs confirming
uname -r
df -h /                  # the Docker image needs 5-8GB
```

The hostname is `localhost`. The measuring host has to be identifiable in the
result files, so it gets a name (proposal: `dealer`).

## 2.11 Building the model conversion environment

**Decision.** Built on **the laptop (x86_64 Linux)** rather than WSL2. Docker is
already installed and it satisfies RKNN-Toolkit2's x86_64 Linux requirement.
There is no reason to set up WSL2 separately.

**Why wrap it in Docker.** If the conversion result varies with the host
environment, reproducibility breaks. The image pins the Python, Toolkit and
dependency versions, so the same `.rknn` comes out on anyone's machine. For an
open-source release, "reproduce it with this image" becomes possible.

**Files written.**

```text
tools/model-converter/
├── Dockerfile            Ubuntu 22.04 + rknn-toolkit2==2.3.0
├── requirements.txt
├── convert_yolov8n.py    ONNX -> RKNN, metadata recorded automatically
└── README.md             usage and deployment procedure
```

**Version pinning.** Toolkit 2.3.0 is matched to the boards' Runtime 2.3.0. A
Toolkit higher than the Runtime may produce models that fail to load.

**Target platform.** Fixed to `target_platform='rk3576'`. A `.rknn` converted
for `rk3588` does not work on RK3576.

**Reproducibility records.** `convert_yolov8n.py` leaves the following as JSON
at conversion time.

```text
ONNX SHA-256 / RKNN SHA-256 / calibration manifest SHA-256
calibration image count / quantization scheme / all conversion options
toolkit version / python version / platform
```

The calibration image list is sorted and fixed, because the order affects the
quantization result.

## 2.12 Accessing and configuring the scheduler host (`dealer`)

**Subject.** `192.168.123.14`, account `yoo2`

### Distribution confirmed: Rocky Linux 9.7

The SSH banner showing `OpenSSH_8.7` + `gssapi-keyex` gave away that it was a
RHEL family. Confirmed as **Rocky Linux 9.7.**

```text
PRETTY_NAME  Rocky Linux 9.7 (Blue Onyx)
kernel       5.14.0-611.13.1.el9_7.x86_64
glibc        2.34
package mgr  dnf
Docker       29.2.1 (overlayfs)
disk         60GB free
Swap         3.9GB
```

**The `sudo apt install ...` run earlier had failed silently.** The errors were
hidden by `2>/dev/null`, and since `ethtool`, `lspci` and `dmidecode` were
already installed the output looked normal. This host uses `dnf`.

### What was encountered while securing access

**First failure.** The `printf`-based askpass helper did not emit the password
correctly. The cause was narrowed by checking the helper's output directly.

```bash
printf "[%s]\n" "$("$ASKPASS")"    # see what actually comes out
```

Switching to a heredoc made it work.

```sh
#!/bin/sh
cat <<'PW'
<password>
PW
```

**Second problem — no sudo.** `yoo2` was not in the `wheel` group
(`id -nG yoo2` → `yoo2`). Rocky does not put users in `wheel` by default.

**Third problem — root SSH blocked.** `PermitRootLogin` was disabled, so root
could not connect directly.

**Solution — escalate with `su`.** `su` reads the password from the controlling
terminal rather than stdin, so `ssh -tt` has to allocate a PTY. And **the prompt
needs time to appear.**

```bash
# fails: the password flows past before su reads it, and gets echoed
printf 'PW\n' | ssh -tt host 'su -c "..."'

# works: insert delays
( sleep 3; printf 'PW\n'; sleep 2 ) | ssh -tt host 'su -c "..."'
```

This pattern has the same cause as §2.1's SSH password problem (no TTY) but a
different solution. SSH is worked around with `SSH_ASKPASS_REQUIRE=force`, and
`su` needs PTY allocation.

### Changes applied

| Item | Change |
|---|---|
| hostname | `localhost.localdomain` → **`dealer`** |
| `yoo2` groups | `wheel` added (sudo possible) |
| `yoo2` groups | `docker` added (docker without sudo) |
| SSH key | `id_ed25519_npuforge` installed |
| SSH alias | `npuforge-dealer` |

`dealer` comes from a card dealer. With the nodes as `king`/`queen`/`jack`, the
naming scheme is consistent.

### ⚠️ The host and the nodes run different distributions

| | `dealer` | `king`/`queen`/`jack` |
|---|---|---|
| Distribution | Rocky Linux 9.7 | Ubuntu 24.04 |
| glibc | 2.34 | 2.39 |
| Package manager | `dnf` | `apt` |

**The binary deployment direction is safe.** A binary built against the lower
glibc (2.34) runs on the higher one (2.39). The reverse does not hold. So
cross-compiling on `dealer` and deploying to the boards is fine.

`scripts/fix-node-consistency.sh` is `apt`-only and targets the nodes, so it can
stay as it is. A script that also handles the host has to branch on package
manager.

## 2.13 Building the model conversion image

**First attempt failed.** The Dockerfile COPYs `validate_rknn.py` and the file
did not exist.

```text
ERROR: "/validate_rknn.py": not found
```

**Incidental lesson.** Piping a background run as `docker build ... | tail -40`
makes the exit code `tail`'s, so **a failure gets reported as success.** It was
changed to log to a file and check the exit code separately.

```bash
docker build -t img . > /tmp/build.log 2>&1; echo "EXIT=$?"; tail -25 /tmp/build.log
```

**Action.** `validate_rknn.py` was written. It checks the converted model's
input/output shapes and, if `onnxruntime` is present, compares cosine similarity
against the ONNX original. The default threshold is 0.98.

It covers the "ONNX result ↔ RKNN simulator result" comparison among DEV-REQ
§2.2's verification targets. The three-board real-hardware comparison is
performed separately.

### Room for improvement: image size

The build log showed `rknn-toolkit2` pulling in `torch` as a dependency, and in
the process **downloading hundreds of MB of NVIDIA CUDA libraries.**

```text
nvidia_cusolver_cu12   124.2 MB
nvidia_cusparse_cu12   196.0 MB
nvidia_nccl_cu12       176.2 MB
...
```

`dealer` has no GPU, so none of it is used. Installing a CPU-only torch first
would save several GB.

```dockerfile
RUN python3 -m pip install torch --index-url https://download.pytorch.org/whl/cpu \
    && python3 -m pip install "rknn-toolkit2==${RKNN_TOOLKIT_VERSION}"
```

With 51GB of disk free it is not an immediate problem. It gets optimized after
the build completes.

## 2.14 Obtaining the YOLOv8n ONNX

### ⚠️ The standard Ultralytics export is unsuitable for RKNN

YOLOv8 for RKNN has to be produced with **Rockchip's modified exporter.** The
standard Ultralytics export includes DFL and NMS postprocessing in the ONNX
graph, and those operators do not map to the NPU, causing extensive CPU
fallback.

The modified version **outputs the raw tensors before decoding** and performs
postprocessing separately on the CPU.

```text
official original : 1 output (decode and NMS included)
optimized version : 3 output groups
                    [1,64,80,80]  box coordinates
                    [1,80,80,80]  per-class confidence for 80 classes
                    [1,1,80,80]   confidence sum
```

This connects directly to `environment-matrix.md` §6's "CPU fallback operator
list" item. **Export it wrongly and you measure the CPU rather than the NPU.**

### The file obtained

`rknn_model_zoo` distributes a pre-optimized ONNX. No manual export is needed,
which reduces the risk.

```text
source    airockchip/rknn_model_zoo  examples/yolov8
upstream  airockchip/ultralytics_yolov8
path      ~/npuforge/models/yolov8n.onnx  (dealer)
size      12,650,184 bytes
SHA-256   0c8716701f471067932b797eeb67c8e5db47c693c2557c881d7679ec12e21bc5
format    PyTorch 2.0 export
```

**RK3576 is on the official supported list.**

```text
RK3562, RK3566, RK3568, RK3576, RK3588, RV1126B, RV1109, RV1126, RK1808, RK3399PRO
```

### License

The `rknn_model_zoo` repository is Apache-2.0 but **the model itself is
AGPL-3.0** (inherited from the Ultralytics original). A repository's license and
a datum's license are separate things.

Details and the response policy are in `MODEL_LICENSES.md`. In summary, the
model file is not included in the repository and users download it themselves.

---

## 2.15 Model conversion succeeded

### An onnx version conflict

The first conversion failed.

```text
AttributeError: module 'onnx' has no attribute 'mapping'
```

**Cause.** `rknn-toolkit2`'s dependency specification does not constrain the
onnx version, so the latest (1.22.0) got installed. `onnx.mapping` was removed
in onnx 1.16 and rknn-toolkit2 2.3.0 uses it.

**Solution.** Pinning to `onnx==1.14.1` made the conversion succeed
immediately. The pin and a verification step went into the Dockerfile.

```dockerfile
RUN python3 -m pip install "onnx==1.14.1" \
    && python3 -c "import onnx; assert hasattr(onnx, 'mapping'), 'onnx.mapping missing'"
```

**An improvement applied alongside.** torch is now installed from the CPU-only
index. That removes the waste of downloading several GB of NVIDIA CUDA libraries
on a host with no GPU.

### Producing the FP16 model

Since the calibration data was not settled, FP16 was converted first instead of
INT8. **Without quantization, thread-safety verification is unaffected.**

```text
file      yolov8n-fp16.rknn
size      9,645,065 bytes
SHA-256   459602ea70479c1ce4fdd7419aa81e10e2f795fe6fe87444f3607f25b7054c0f
```

Deployed to the three nodes with matching SHA-256 confirmed. The test program
also compiled successfully on all three.

## 2.16 Thread-safety verification — in progress

### Preliminary observations (2 iterations)

```text
RKNN api        2.3.0 (c949ad889d@2024-11-07T11:35:33)
RKNN driver     0.9.8
inputs/outputs  1 / 9              the optimized version's 3 groups x 3
input size      1,228,800 bytes    = 640x640x3, matching the documented calculation
FP16 inference  78.8 - 116.1 ms
```

**Scenario 1 (shared context, 2 threads) gave 0 errors.** Two iterations is far
too small a sample to conclude from, so it is being re-measured at 20.

At about 100 ms for FP16, that is around 10 FPS per node. INT8 is usually 3–5×
faster, so 30–50 FPS is expected. **That figure is the basis for the 2.5G switch
purchase decision.**

### Two pitfalls hit during execution

**1. `head` in a pipe swallows the output.**

```bash
ssh host './test model 30' | grep -v ... | head -70    # 0 bytes of output
```

`head` closed the pipe early, raising SIGPIPE and terminating the remote command.
The background job reported exit 0 and looked successful.

**2. Block buffering plus SIGHUP on file redirection.**

```bash
ssh host './test model 50 > run50.log 2>&1'
```

When stdout is a file, libc uses **block buffering** rather than line buffering.
The SSH session dropped, the process was terminated by SIGHUP, and everything in
the buffer was lost. Only the one line that went to stderr survived (stderr is
always unbuffered).

**Solution.** Detach from the session and force line buffering.

```bash
nohup bash -c 'stdbuf -oL -eL ./test model 20 > run20.log 2>&1; echo DONE=$? > done.marker' &
```

The completion marker file is polled to collect the result. **The same pattern is
needed for long-running benchmarks** — reflected in `run-benchmark.sh`'s
unattended execution requirements (`01-TECHSPEC.md` §20.4).

---

# 2.17 ⚠️ The boards reboot under heavy load (unresolved)

## Symptom

Running the thread count sweep (3–8 threads) **reboots `king` and `jack`.**
`queen` completes the same test.

| Node | Boot count | uptime (2026-08-10 02:00) | Sweep result |
|---|---:|---|---|
| `king` | **13** | 15 min | **3 reboots** (01:26, 01:38, 01:45) |
| `queen` | 5 | **3 days 17 hours** | **completed** |
| `jack` | 5 | 26 min | rebooted |

`king` has 8 more boots than the other two. All coincide with when today's sweep
was run.

## It is a hard reset

There is **no shutdown sequence at all** in the log before the reboot. The log
simply stops right after an SSH session opens.

```text
Aug 10 01:45:45 king sshd[1586]: Accepted publickey for pi ...
Aug 10 01:45:45 king systemd-logind[488]: New session 4 of user pi.
(end of log - no kernel panic, no shutdown message)
```

There is no kernel panic, OOM killer or thermal shutdown message. It looks like
**a hard reset from power loss or a watchdog.**

## Candidate causes

| Candidate | Basis | Verdict |
|---|---|---|
| **Insufficient power supply** | hard reset, no logs, per-node variance | **leading** |
| Heat | 45–50 °C at the time of reboot | **excluded** (far from the threshold) |
| Out of memory | 3.2GB available, no OOM logs | excluded |
| A defective unit | only `queen` is fine | possible |

Power is the leading candidate because it fits **the load characteristics.**
Eight threads use 8 CPU cores and 2 NPU cores at maximum simultaneously. If the
instantaneous current exceeds the adapter's capacity, voltage drops and the board
resets. Leaving no log is consistent with that.

That `queen` completed the same test with 3 days 17 hours of uptime suggests
**a per-unit hardware condition difference rather than a software problem.**

## The document's power assumption needs correcting

`02-HARDWARE-SETUP.md` §8 assumes a **USB-C PD adapter**, but the regulator
names in the kernel log are:

```text
vcc12v_dcin      12V DC input
vcc_sys
rk806-regulator
```

**The actual power input method has to be confirmed.** If it is 12V DC, all of
§8 written on a USB-C PD premise is wrong.

## The impact on the project — serious

The official benchmarks are **300 s of sustained load × 5 repetitions × 143
runs, 22 hours in total** (`01-TECHSPEC.md` §20.4).

In the current state:

- A node reboots mid-measurement and the run becomes invalid
- Recording a reboot as a "node failure" would **mismeasure the software's
  failure detection performance**
- In the S4 failure recovery experiment, intentional failures and power problems
  cannot be told apart
- Unattended overnight execution is impossible

**This has to be resolved before the S0 thermal measurement.**

## Action plan

1. **Check the three nodes' power adapters** — manufacturer, model, rated
   output. Needs physical inspection
2. Confirm the input method — USB-C PD or a 12V DC barrel jack
3. Attempt reproduction by putting `queen`'s adapter on `king` — to tell an
   adapter problem from a board problem
4. Unify on three identical adapters (`infrastructure.md` §5 purchase list)
5. After resolution, re-run the sweep to confirm consistency across all three

**No repeating heavy-load tests until it is resolved.** There is no reason to
raise the risk of eMMC damage by repeatedly forcing reboots.

## The valid data survives

`queen` completed the whole sweep, so **the thread-safety conclusion (§3.1) is
valid.** But confirming reproducibility across three is deferred until the power
problem is resolved.

## Correction: these were two different phenomena

Re-checking the boot history by absolute time, the analysis above is corrected.
**Grouping them under one cause by comparing uptime alone was hasty.**

### Event A — individual reboots under load (to investigate)

```text
01:26:16  king  reboot
01:34:40  jack  reboot
01:38:12  king  reboot
01:45:58  king  reboot
```

All coincide with the sweep test's execution times. Throughout this window
**`queen` had 3 days 17 hours of uptime.**

Being correlated with load and appearing differently per node, the power supply
or per-unit variance hypothesis **is valid for this event.**

### Event B — all three rebooting together (unrelated to load)

```text
king   previous boot ended  02:01:00
queen  previous boot ended  02:05:20
jack   previous boot ended  02:05:10
       | about 27 minutes with no power
all three booted around 02:32   (identical uptime of 1h47m as of 04:19)
```

All three went down within 4 minutes of each other and **came back together
after 27 minutes off.** No load test was running at that time.

This is a **shared power cut** (a blackout, a power strip switched off, physical
relocation) and has a different cause from event A. A load-induced reset reboots
immediately; execution does not stop for 27 minutes.

**So the earlier statement that "heavy load reboots all three" overstated it.**
Only event A is connected to load.

### What event B actually was: power rearrangement work

Confirmed with the user: **work was done separating the three boards' power onto
independent sources.** The simultaneous stop at 02:05 and the 27-minute gap match
that work's duration.

**So event B is not a failure but planned physical work.** Recording it as an
unexplained reboot would have been a false trail.

`02-HARDWARE-SETUP.md` §8.1's requirement of "not putting all three on one
multi-port charger" is thereby satisfied.

### What remains to diagnose

| Event | Time | Cause | Status |
|---|---|---|---|
| A: king ×3, jack ×1 | 01:26–01:45 | heavy load on the configuration **before** the power rearrangement | **needs re-verification** |
| B: all three, 27 min | 02:01–02:32 | power rearrangement work | resolved (not a failure) |

**Event A occurred before the rearrangement.** With power now on independent
sources it may not reproduce. To be re-verified under identical conditions.

### Lesson: do not judge from uptime comparison alone

The initial conclusion from `uptime` alone was "heavy load reboots all three",
but by absolute time they were two different events. And one was not a failure
but planned work.

**When recording a node restart during a benchmark, leave the absolute time and
the work history together.** Otherwise physical work gets misread as a software
failure. That is this document's reason for existing.

## 2.17.1 Cause established: `king`'s bootloader firmware is an old version

After verifying the power hypothesis twice, the actual cause was found.

### Evidence that power is not the cause

| Observation | Implication |
|---|---|
| `queen` completed 8 threads even on the shared 3-port supply | the shared supply itself is not the problem |
| `king` still resets at 5 threads after switching to individual supplies | it is not an adapter capacity problem |
| The three adapters are under identical conditions | it is not a per-adapter difference |

### Firmware comparison

```bash
grep -oE 'androidboot\.fwver=[^ ]*' /proc/cmdline
```

| Component | `king` | `queen` | `jack` |
|---|---|---|---|
| DDR init | **v1.09** | v1.13 | v1.13 |
| SPL | **v1.07** | v1.09 | v1.09 |
| **BL31 (ATF)** | **v1.17** | **v1.24** | **v1.24** |
| BL32 | **v1.05** | v1.10 | v1.10 |
| U-Boot | **2025-07-17** | 2026-07-10 | 2026-07-10 |
| PMIC initialisation | **`ON:0x20 OFF:0x2`** | `ON:0x40 OFF:0x0` | `ON:0x40 OFF:0x0` |

`queen` and `jack` match exactly and **only `king` is about a year old.**

**BL31 is the ARM Trusted Firmware and handles DVFS and voltage regulation on
Rockchip platforms.** If the voltage tables or DVFS logic changed between v1.17
and v1.24, an old version failing to cope with heavy-load voltage is exactly the
symptom observed.

The DDR firmware difference (v1.09 vs v1.13) can also cause instability under
memory-heavy multi-threaded conditions.

The differing PMIC initialisation register is a consequence of the firmware
difference.

### The cost of a wrong diagnosis

Suspecting power, the user replaced all three adapters, and that was not the
cause. The individual power configuration itself satisfies
`02-HARDWARE-SETUP.md` §8.2's requirements so it is not wasted, but **time was
spent going in the wrong diagnostic direction.**

The fact that `queen` completed 8 threads on the shared supply was already
weakening the power hypothesis, and that signal was not taken seriously enough.

### The gap in the documents

`environment-matrix.md` had the kernel, glibc and RKNN versions but **no
bootloader firmware entry.** `collect-node-info.sh` did not collect it either.

Claiming to verify "three identical machines" while **omitting the layer
responsible for power management.** Both were fixed (2026-08-10).

### The image version is identified

```text
/etc/rom-version
  king   20251222     the 2025-12-22 image
  queen  20260721     the 2026-07-21 image
  jack   20260721
```

Only `king` is on an image 7 months old. That is where the firmware difference
comes from.

`/etc/friendlyelec-release` is identical on all three (`BOARD=NanoPi-R76S`,
`LINUXFAMILY=nanopi-m5`, `BRANCH=dev`). What distinguishes them is
`rom-version`, so **that value is added to the node consistency checks.**

### Action: reinstall `king`'s OS (decided 2026-08-10)

Rather than updating only the bootloader, **the OS is reinstalled.** The grounds:

- `king` is also behind on OS patch level (24.04.3 vs 24.04.4). A reinstall
  resolves both
- Six hard resets during diagnosis make the filesystem state hard to trust
- A bootloader-only update procedure needs `rkdeveloptool`/`eflasher` and is
  actually more complex

**Target image: `rom-version = 20260721`** (Ubuntu 24.04 for the NanoPi-R76S,
the FriendlyElec distribution)

After reinstalling, `scripts/setup-node.sh` handles the setup automatically.

```bash
./scripts/setup-node.sh 192.168.123.12 king npuforge-k
```

What that script does:

| Stage | Content |
|---|---|
| 1 | Install the SSH key (using `SSH_ASKPASS_REQUIRE=force`) |
| 2 | Register the `~/.ssh/config` alias |
| 3 | Set the hostname, clean up `/etc/hosts` |
| 4 | **Regenerate the SSH host key** (preventing duplication from image cloning) |
| 5 | **Hold the kernel packages** (protecting the RKNPU driver) |
| 6 | Install base packages, enable chrony |
| 7 | **Compare the environment against the reference node (`queen`)** — `rom-version`, `fwver`, kernel, glibc, RKNN version and hashes, NPU core count, RAM |

Stage 7 is the important one. The script itself judges whether the reinstall
achieved its purpose.

### The verification sequence after reinstalling

```bash
# 1. collect the measurements
ssh npuforge-k 'bash -s' < scripts/collect-node-info.sh > benchmarks/node-info/king.txt

# 2. confirm firmware match (setup-node.sh compares automatically, but re-check)
for h in npuforge-k npuforge-q npuforge-j; do
  ssh $h 'printf "%s %s\n" "$(hostname)" "$(grep -oE "androidboot.fwver=[^ ]*" /proc/cmdline)"'
done

# 3. re-verify stability - the 5-8 thread range that used to reset
ssh npuforge-k 'cd ~/npuforge-rknn-test && ./thread_safety_test yolov8n-fp16.rknn 20 5 8'
```

If 3 passes, `worker_count` can be set identically on all three nodes and the
"three identical machines" premise is restored.

## 2.17.2 Cause established: insufficient power adapter current (resolved, 2026-08-10)

### The decisive evidence: measuring the input voltage

It was belatedly discovered that the board has an input voltage sensor.

```bash
cat /sys/class/power_supply/simple-vin/voltage_now
```

| State | Idle voltage |
|---|---|
| **The previous adapter** | **4.983 V** ← already below 5V at no load |
| **A 5V 4A adapter** | **5.26 – 5.31 V** |

The previous adapter **could not hold 5V even at no load.** Dropping further
under heavy load past the board's brownout threshold was the cause of the
reboots.

The new adapter's voltage under load (`king`, 984 samples up to 8 threads):

```text
minimum 5.061 V   mean 5.260 V   maximum 5.341 V   range 0.280 V
```

It does not fall below 5V even under load.

### Verification: all three complete 8 threads

| Node | 8-thread throughput | Errors | Reboots |
|---|---:|---:|---|
| `king` | 77.3 inf/s | 0 | **none** |
| `queen` | 70.2 inf/s | 0 | **none** |
| `jack` | 78.0 inf/s | 0 | **none** |

`king` passed 4 threads too (54.1 inf/s). Previously it rebooted even at 3.

### ⚠️ The record of misjudging the voltage as 12V

The kernel log's `vcc12v_dcin: 12000 mV` was taken as the actual input voltage
and recorded in the documents as "12V DC input". **That was wrong.**

The name is a device-tree fixed-regulator declaration, left over from Rockchip
device trees being copied between boards. The actual input is 5V.

**What should have been checked was the measurement, not the declaration.**

```text
declaration (device tree)  vcc12v_dcin: 12000 mV     <- not trustworthy
measurement (sensor)       simple-vin: 4983000 uV    <- this is the fact
```

When the user said they would replace with a 5V 4A adapter, I nearly warned that
"5V is dangerous". Checking the measurement first prevented the error.

### The hypotheses that were wrong during diagnosis

| # | Hypothesis | Result | Refuting evidence |
|---|---|---|---|
| 1 | The shared 3-port supply is the cause | **wrong** | `queen` completed 8 threads on the shared supply |
| 2 | An old bootloader firmware | **wrong** | `king` still rebooted after a reinstall matched the firmware. `jack` had the same firmware from the start and failed |
| 3 | The input voltage is 12V | **wrong** | measured 4.983V |
| 4 | **Insufficient adapter current** | **right** | idle 4.983V → 5.3V after replacement, and all three completed 8 threads |

**Hypothesis 1 got the cause half right and was still dismissed as refuted.** The
question was not "shared or individual" but "is the capacity sufficient", and
focusing on the configuration meant missing the capacity. That it got worse after
switching to individual supplies was the evidence (the new adapters were weaker),
and even then the direction turned to firmware rather than back to current
capacity.

That `queen` completed 8 threads on the shared supply meant "that adapter was
sufficient", not "power is not the cause".

### Lesson: find the measurement sensor first

`/sys/class/power_supply/` was not in the first `collect-node-info.sh`. Had that
sensor been found at the point power came under suspicion, **hypotheses 2 and 3
could have been skipped entirely.**

An input voltage item was added to `collect-node-info.sh`.

### Sustained load verification (all three simultaneously, 8 threads)

Passing under burst load does not guarantee passing under sustained load, so it
was checked separately.

**Voltage — no problem.**

| Node | Minimum voltage |
|---|---|
| `king` | 5.061 V |
| `queen` | 5.157 V |
| `jack` | 5.124 V |

Even at maximum load on all three simultaneously it does not fall below 5V. No
reboots. **The power problem is resolved.**

**Temperature — a new problem appeared.**

| Node | Peak SoC | Peak NPU |
|---|---:|---:|
| **`king`** | **88.7 °C** | **91.3 °C** ⚠️ |
| `queen` | 70.2 °C | 70.2 °C |
| `jack` | 71.2 °C | 72.1 °C |

`king` is about **19 °C hotter** than the other two. And it **exceeded
`disable_temperature_c` (90 °C).**

The three boards are the same model with the same firmware under the same load,
so a software cause is excluded. The candidates are:

- A difference in physical placement (airflow, proximity to a wall, spacing
  between boards)
- Heatsink contact
- Per-unit variance

`king`'s load started about 6 minutes before the others, but `queen` and `jack`
had already reached their plateau (70–72 °C), so the time difference alone cannot
explain 19 °C.

**Handled separately in §2.19.**

### Remaining checks

- Throughput had not bent even at 8 threads, so raise `MAX_THREADS` and find the
  optimum again
- There is no means of measuring current. Only `voltage_now` exists and not
  `current_now`, so power consumption cannot be computed. An external power meter
  is needed for the FPS/Watt metric

## 2.19 `king` runs 19 °C hotter (did not reproduce, 2026-08-11)

Found during the sustained load trial. Under identical conditions, only `king`
reached NPU 91.3 °C and crossed the scheduling exclusion threshold.

### Why it matters

**Per-node temperature spread directly contaminates scaling efficiency
measurement.**

- If `king` enters throttling first, its throughput falls
- The scheduler recognises it as a "slow node" and reduces its load
- The result is a low measured three-node scaling efficiency, with **the cause
  being physical placement rather than scheduling**
- Above 90 °C it is excluded from scheduling entirely, making it effectively a
  2-node experiment

That is why `02-HARDWARE-SETUP.md` §9.1 requires "the same ambient temperature,
the same orientation, at least 10 cm between boards".

### What to check

| Item | Method |
|---|---|
| Physical placement | check the three boards' spacing, orientation and surrounding obstructions |
| Stacking | separate them if stacked |
| Airflow | whether blocked by a wall, a corner or a bundle of cables |
| Ambient temperature | the actual temperature at each board's position (sunlight, heat from other equipment) |
| Heatsink contact | the state of the case mounting |

After making the placement uniform, repeat the same trial to see whether the
spread disappears. If it remains, it is per-unit variance and gets stated in the
results.

### There is no spread in idle temperature (confirmed 2026-08-11)

The three boards were measured simultaneously 19.9 hours after the load ended.

| Board | NPU (idle) | SoC | load1 | NPU under load (2026-08-10) |
|---|---|---|---|---|
| `king` | 39.8 °C | 40.7 °C | 1.34 | 91.3 °C |
| `queen` | 36.1 °C | 36.1 °C | 0.07 | 70.2 °C |
| `jack` | 37.0 °C | 38.8 °C | 0.23 | 72.1 °C |

**The idle spread is only 2.8–3.7 °C.** And even that is with a
`gnome-control-center` session running on `king` at the time of measurement (load
1.34) while the other two were effectively idle. At idle, the three boards are
essentially the same.

What that means:

- The 19 °C is **a gap that opens only under sustained load.** That fits
  explaining it by a difference in heat dissipation (airflow) — the difference
  does not show at idle heat output, and grows into a temperature gap as heat
  output rises
- A defective unit (poor heatsink contact, say) would likely have shown to some
  degree at idle. It cannot be fully excluded, but the placement hypothesis is
  stronger
- **So re-measurement has to be done under load.** Judging "it is resolved" from
  idle temperature alone would be wrong

All three were also confirmed identically configured with `graphical.target` +
`gdm` active. A desktop session running on only one board is itself a source of
measurement contamination, so session state is matched immediately before a
benchmark (a `preflight-check.sh` item).

### Controlled re-measurement: the 19 °C gap does not reproduce (2026-08-11)

A dedicated load tool (`sustained_load_test`) applied 8-thread load
**simultaneously** to all three boards for 15 minutes. A summary of the plateau
(from 300 s after load to the end, about 557 samples per board).

| Board | NPU mean | NPU peak | SoC mean | Min input voltage | Throughput |
|---|---|---|---|---|---|
| `king` | 73.0 °C | **75.8 °C** | 71.2 °C | 5.070 V | **80.5 inf/s** |
| `queen` | 67.5 °C | 70.2 °C | 65.8 °C | 5.090 V | 77.7 inf/s |
| `jack` | 72.6 °C | 74.8 °C | 71.6 °C | 5.046 V | 77.8 inf/s |

**Maximum spread 5.6 °C. Never exceeded 90 °C. No NPU clock drop** (all 928
samples at 950 MHz, not one dropped).

The rise curves run parallel across the three boards too.

```text
 t(s)   king  queen   jack
    0   37.0   35.2   37.0
   60   66.5   61.9   66.5
  120   72.1   65.6   69.3
  300   73.0   67.5   73.0
  600   73.9   67.5   73.0
  880   74.8   68.4   72.1
```

### What differed from the earlier measurement

The 08-10 measurement (`king` 91.3 / `queen` 70.2 / `jack` 72.1) cannot be
compared directly. **The load profile differed.**

| | 2026-08-10 | 2026-08-11 |
|---|---|---|
| Tool | `thread_safety_test` | `sustained_load_test` |
| Load shape | a sequential 1→8 thread sweep | fixed at 8 threads |
| Start | `king` about 6 minutes ahead | simultaneous |
| Duration | until the sweep completed | fixed at 900 s |

`thread_safety_test` runs single- and two-thread baselines before reaching the
target thread count. So `king` had been heating for far longer by the time the
other two entered 8 threads. Add the 6-minute head start and the conditions for
an inflated gap are in place.

`queen`'s peak temperature is **an identical 70.2 °C** in both measurements, and
`jack` rose slightly, 72.1 → 74.8 °C. Only `king` moved (91.3 → 75.8 °C). Given
that the placement was not changed, much of the gap was likely **a measurement
method problem rather than physical placement.**

Placement cannot be fully excluded, of course. But under the current conditions:

- No board reaches `degraded_temperature_c` (80 °C)
- No board throttles
- Throughput spread is within 3.5% (80.5 / 77.7 / 77.8 inf/s)

so **it is not a blocker for benchmarking.** The S0 experiment can proceed.

That `king` is both the hottest and the fastest is consistent too — over 15
minutes it did 72,481 inferences, 3.6% more work than `queen` (69,928). But a
3.6% difference in work does not fully explain 5.5 °C, so a small difference in
heat dissipation conditions is presumed to remain.

### The measurement principle obtained here

**Do not compare temperatures across different load profiles.** Even the same
"heavy load" accumulates different amounts of heat depending on how it is
reached. After S0, all thermal comparisons are performed with
`scripts/run-thermal-comparison.sh`. That script:

- verifies alias↔hostname agreement first (§2.20)
- confirms the three boards' binary and model hashes match
- takes an idle baseline first
- applies load to all three **simultaneously**
- compares `boot_id` before and after the run and invalidates any board that
  reset mid-run

### The thresholds need re-examining

The current settings are the draft values.

```text
degraded_temperature_c = 80.0
disable_temperature_c  = 90.0
```

If a fanless board reaches 70–91 °C during normal operation, these values are
**an obstruction to measurement rather than protection.** They get reset from S0's
results (`02-HARDWARE-SETUP.md` §9.2).

RK3576's actual critical temperature (Tj max) has to be confirmed and the values
set comfortably below it, but above the normal operating range.

## 2.20 The `king` IP written in the documents was wrong (2026-08-11)

`king` had been recorded as `192.168.123.22` but **its actual address is
`192.168.123.12`.** `.22` was an empty address that did not even answer ARP in a
full subnet sweep, and the result was the wrong conclusion that "`king` is dead".

### Why it was missed

`~/.ssh/config`'s `npuforge-k` alias had **`.12` correctly from the beginning.**
What was wrong was only the IP hardcoded into documents and scripts. Using the
alias would have meant the problem never surfaced.

| Location | Value | Status |
|---|---|---|
| `~/.ssh/config` `npuforge-k` | `.12` | correct |
| `board-worklog.md` §1 table | `.22` | **wrong** |
| `environment-matrix.md` §7 | `.22` | **wrong** |
| `infrastructure.md` | `.22` | **wrong** |
| `setup-node.sh` usage example | `.22` | **wrong** |
| `fix-node-consistency.sh` IP list | `.22` | **wrong** |

All corrected to `.12`.

### Preventing recurrence

**Boards are reached by alias (`npuforge-k/q/j`), not by IP.** IPs change under
DHCP, and pinning them into documents guarantees one goes stale. An alias needs
fixing in one place only (`~/.ssh/config`).

The following go into `preflight-check.sh`.

- Do all three aliases connect
- Does the `hostname` of the host each alias reaches match `king/queen/jack`

Running a benchmark with the names misaligned attributes results to the wrong
node. Ending in "the node is dead", as happened here, is the better case;
silently attaching to a different board is far more dangerous.

### Note: the boards' MACs have no OUI

All three boards use locally administered MACs (`82:`, `66:`, `26:` — second
nibble 2/6/A/E). That means a board cannot be identified by manufacturer OUI, so
finding boards by network scan does not work.

But `addr_assign_type = 0` (permanent), so **the MAC survives a reboot.** There
is no reason for the DHCP lease to move. Still, pinning the IP (static assignment
or a DHCP reservation) is safer.

## 2.21 Two pitfalls in remote background execution (2026-08-11)

While building `preflight-check.sh`, the check was found to be **silently not
working.** It passed with "no residual load" while load was running.

Two things overlapped.

### Pitfall 1: `pgrep -f` counts itself

`pgrep -f` matches the whole command line. The wrapper ssh sends is

```text
bash -c "... pgrep -f \"[s]ustained_load_test|...\" | wc -l"
```

and that command line contains the pattern string. The bracket trick
(`[s]ustained`) is neutralised once a form without brackets appears on the same
command line.

**It is wrong in both directions.**

| Situation | Actual | pgrep reports |
|---|---|---|
| Load running | 1 | 0 (missed) |
| No load | 0 | 2 (counting its own shell) |

It was changed to read the `/proc/PID/exe` symlink. That points at the actual
executable, leaving no room for a shell to get involved.

```bash
n=0
for p in /proc/[0-9]*; do
  case "$(readlink "$p/exe" 2>/dev/null)" in
    *sustained_load_test) n=$((n+1)) ;;
  esac
done
```

### Pitfall 2: `cd DIR && setsid nohup ... &` does not come up

The two forms were compared under identical conditions.

| Form | Result |
|---|---|
| `ssh -n H "cd $DIR && setsid nohup ./prog ... &"` | **does not run** |
| `ssh -n H "setsid nohup $DIR/prog ... &"` | runs |

The `&` applies to the whole `cd && prog` list. ssh sends the command and
disconnects immediately, and if the session disappears before the background
subshell gets through `cd` and reaches `setsid`, it dies. Using an absolute path
removes the intermediate step so no race arises.

**There is no signal at all on failure.** The exit code is 0 and stderr is empty.
Without checking, you end up measuring "the temperature with no load" for fifteen
minutes.

`run-thermal-comparison.sh` was already using the absolute path form, so the
2026-08-11 thermal measurement was unaffected. But **a step confirming it is
actually running after starting it** was added.

### The shared lesson

Both pitfalls make **failure look like success.** The same family as discuss.md
§10's type A (not checking what a metric counts).

**When adding a check, break it deliberately and confirm it actually catches.**
That procedure is how this was found again. Trusting a pass at face value would
have left preflight filtering nothing.

## 2.29 S3 saturation sweep — near-linear by the ceiling measure too (2026-08-20)

Each node count's true throughput ceiling was found by a concurrency sweep (S2 is
linearity under identical load, S3 is maximum throughput — separate experiments).
45 runs, frozen `1da69d4`.

| Config | Ceiling @ conc | Speedup | Eff |
|---|---|---:|---:|
| 1N | 115.2 @ c32 | 1.00× | 100% |
| 2N | 232.0 @ c24 | 2.01× | 101% |
| 3N | **341.8 @ c32** | **2.97×** | **99%** |

- The curve: unsaturated (round-trip latency) → plateau (~10–16 concurrent per
  node) → a slight decline under overload. 0 errors (the queues absorb it).
  SD ≤ 2.2.
- **Near-linear re-confirmed from two angles, S2 (identical load) and S3
  (ceiling).**
- Report: `docs/experiments/S3_SATURATION.md`, raw data:
  `results/saturation-20260820/`.

Next: S4 io_uring — comparing the cost reduction on payload transfer (94% of
non-inference latency).

## 2.28 gRPC baseline over 30 repetitions — reproduction confirmed, baseline frozen (2026-08-20)

The first result was promoted to a "reproduced result". With code and
configuration frozen (bench `254d560`), 10 runs of 60 s at each of 1N/2N/3N,
with **the condition order rotating** (spreading time and temperature drift).
`scripts/run-grpc-baseline30.sh`. Raw data and aggregation:
`results/baseline-20260820/`.

### Results

| N | Throughput Mean±SD | Speedup | Eff | p50/p99 ms | Err | Bal |
|---:|---:|---:|---:|---|---:|---:|
| 1 | 112.9 ± 0.5 | 1.00× | 100% | 68.0 / 116.3 | 0% | 0.00 |
| 2 | 229.0 ± 0.9 | 2.03× | 101% | 67.0 / 118.6 | 0% | 0.00 |
| 3 | **338.4 ± 1.1** | **3.00×** | 100% | 67.6 / 123.9 | 0% | 0.00 |

- **The first measurement's 337.7 reproduced as 338.4 ± 1.1.** SD of 0.5–1.1 is
  extremely small.
- 30/30 active node determinations correct, 0 invalid, 0% errors, balance 0 pp.
- Against saturation (115), 3N efficiency is 98%; against the 1N c8 reference,
  speedup is 3.00×.

### The TimingBreakdown reproduced too (30-run average of p50)

3N: network_to_node 17.11 + network_to_client 17.11 = 34.21 ms
  = **94%** of the non-inference overhead (36.34), 58% of E2E (58.83).
scheduler_queue/route are ~0 at both 1N and 3N — no scheduler bottleneck,
re-confirmed.
1N's and 3N's network figures are nearly the same (17.7 vs 17.1), so transfer
time is independent of node count.

### The promoted statement

"337.7 once" → **"3-node near-linear scaling confirmed across 30 repeated
experiments (338.4 ± 1.1 inf/s, speedup 3.00×, error 0%)."** The gRPC baseline is
frozen.

Next: the saturation sweep → (freeze maintained) → comparing io_uring under
identical conditions.

## 2.27 Re-measuring the local fan baseline — the overhead settles at 27% → 28.8% (2026-08-20)

The 27%'s reference value of 157 was fanless (08-11/12), so its cooling condition
differed from the cluster's (fan). Local sustained was re-measured under the same
fan condition. The king node was stopped and a purely local
`sustained_load_test` run (no gRPC), INT8, governor=performance, fan on.

```text
8 threads (worker 8, matching the cluster) 60 s x 3:  159.2 / 162.0 / 163.2 -> 161.5
16 threads (checking saturation):                     165.7
```

**Settled: overhead = (161.5 − 115) / 161.5 = 28.8%** (with cooling, workers and
measurement duration unified).

### The finding — the 27% did not collapse under cooling

The concern was "with a fan, local would be far above 157 and the overhead would
widen substantially". In fact it was 161.5 with a fan vs 157.2 fanless — **a
small difference.** The reason:

**A 60/30-second measurement is before throttling appears.** CPU throttling shows
as −27% at 300 seconds (§2.24, discuss §12). In a short window, initial
throughput is similar with or without a fan, so the cooling condition has little
effect.

→ **The 27% was not invalidated by cooling but adjusted slightly to 28.8%.** The
bottleneck's location (payload transfer, §8 and §2.26) was unrelated to cooling
in the first place and is unchanged. **The two hardest facts did not move**:
(1) scaling efficiency ~98% linear, (2) 94% of non-inference latency is payload
transfer.

**Do not multiply the two quantities.** Throughput loss 28.8% (a throughput
figure) and the latency breakdown 94% (a share of latency) are different axes.
"94% of 28.8%" is a wrong multiplication. The accurate wording: the cluster's
single-node throughput was 28.8% below local, and separately, a latency breakdown
found 94% of non-inference latency in payload transfer.

### What remains (a separate condition)

- **Sustained load (300 s) overhead**: if the fan's benefit grows, the overhead
  could widen. How throttling applies differently to local (sustained) and to the
  cluster (nodes) is the next question. But that is a separate axis from the
  "short measurement 28.8%".
- Saturation: 16 threads (165.7) > 8 threads (161.5), so worker 8 is not local's
  maximum. Since the cluster nodes run worker 8, 8 threads is the right
  like-for-like comparison.

## 2.26 The first TimingBreakdown measurement — the overhead is payload transfer (2026-08-20)

The bench was extended to collect all 11 stages of the response's `Timing`
(proto) (previously only `inference_us`). This is the first measurement breaking
the 27% per-node overhead into stages.

Measured: 3 nodes / c24 / 10 s / active cooling / gRPC.

```text
stage (p50 ms)
  scheduler_queue      0.00
  scheduler_route      0.00
  network_to_node     17.16   +- payload transfer
  node_queue           0.02   |
  decode/preprocess    0.00   |
  npu_input            0.00   |
  inference (NPU)     22.49   | <- the actual inference
  postprocess          0.00   |
  network_to_client   17.16   -+
  end_to_end          58.99
```

**The finding: the per-node overhead is payload network transfer.**
payload transfer = `network_to_node + network_to_client` = 34.32 ms.
Not protobuf serialization, not the scheduler queue (~0), not the node queue
(~0). Most of it is the time to carry 1.17 MiB in and out over 2.5G.

**Distinguish the denominators clearly (to prevent confusion):**

```text
payload transfer / E2E latency            = 34.32 / 58.99 = 58%
payload transfer / non-inference overhead = 34.32 / 36.50 = 94%
  (non-inference overhead = E2E - inference = 58.99 - 22.49 = 36.50 ms)
```

The accurate wording: **"94% of the per-node overhead (= E2E − inference) is
payload transfer"**, and "58% of E2E latency is payload transfer, 38% pure
inference".

→ It is confirmed by measurement that what io_uring, zero-copy, JPEG input and
postprocessing (NMS) for response reduction would aim at is **the network
transfer path.**

### The instrumentation's limits (stated honestly)

- gRPC **serialization time cannot be isolated** — the proto `Timing` has no
  separate field. Measuring it needs an additional instrumentation point. It is
  currently mixed into the residual (~2 ms).
- bench↔scheduler is **the same host (loopback)**, so client→scheduler is ~0. The
  real network is only the scheduler↔node 2.5G section.
- **The cooling condition is unsettled:** this breakdown is internal to the
  cluster and valid regardless of cooling, but the "27%" itself is not settled,
  being fanless 157 vs cluster-with-fan 115 (§2.24).
- These are c24 (24 concurrent) values, so `network_*` depends on concurrency.
  Single-request transfer time has to be looked at separately at low concurrency.

The working aggregation table is
`results/NPUForge_Benchmark_Result_Workbook.md` §8 (local only).

## 2.25 The first S2 scalability measurement — scaling efficiency 98%, per-node overhead found (2026-08-20)

After fixing the model_file bug and with preflight passing, 1/2/3-node
scalability was measured for the first time. **Close to formal (preflight passed,
30 s, conditions controlled) but a single run without --with-inference, so not a
settled figure.**

Measured: INT8, want_float=0, governor=performance, **active cooling (a dedicated
fan per node, from the start of measurement)**, gRPC via the scheduler (.9),
round-robin. Node count was reduced by stopping processes (jack, then queen),
with cooldown in between.

> ⚠️ **Cooling condition corrected (after the fact, 2026-08-20).** Every
> measurement in this session was with fans fitted. It was initially recorded as
> "cold/fanless", but in reality large fans were attached from the start. That
> affects the 27% calculation — see the conclusions below.

### Equal per-node load (concurrency = 8 × node count)

| Configuration | Throughput | Distribution |
|---|---:|---|
| 1 node c8  | 111.6 inf/s | king 100% |
| 2 nodes c16 | 228.7 inf/s | 50/50 |
| 3 nodes c24 | 337.7 inf/s | 33/33/33 |

Error rate 0%, round-robin splitting exactly evenly. 3 nodes / 1 node =
**3.03×.**

### 1-node concurrency sweep — a ceiling of ~115

| c8 | c16 | c32 |
|---:|---:|---:|
| 111.6 | 114.0 | 115.1 |

Raising concurrency **saturates at ~115 inf/s.** That is the single-node ceiling
through the scheduler.

### Two conclusions

**1. Scaling efficiency ~98% (nearly linear).** Against the 1-node saturation of
115, three nodes at 337.7 is 2.93×. Data parallelism (`adrs/001`) holds and the
scheduler is not a bottleneck even with three nodes. `adrs/003`'s single
scheduler is confirmed sufficient at this scale by measurement.

**2. The cluster node ceiling of 115 < the local sustained 157 (−27%).** The
round-trip p50 is 69 ms while node-reported inference is 24–28 ms — **40 ms+ is
overhead from going through the scheduler's gRPC** (serialization + transferring
1.17 MB in and out + queueing/routing). Scaling is linear while the per-node
absolute ceiling is cut by the network and scheduling.

> The first measured answer to the project's central question, "do three 6 TOPS
> units really make 18 TOPS": **2.93× (98%) on a cluster basis.** The bottleneck
> is not scaling but per-node overhead. Where that 27% comes from gets broken
> down next with the `TimingBreakdown` stages.

### Minor issues

- Every bench run filename is `-n3` — the run_id's node count comes from **the
  initial ListNodes (registrations)** rather than what is active at measurement
  time. Stopping jack/queen leaves their registrations, so it was stamped as 3.
  The actual node count is established only from the result's distribution.
  Taking the run_id from the nodes active at the end of measurement is correct.
- Node count was reduced by killing processes. A drain RPC would let in-flight
  requests through and remove them cleanly (`adrs/027`). To be considered for the
  formal S2.

### What remains for the formal S2

Repeated runs (variance), fan conditions (S0-B), --with-inference, the full
concurrency sweep, 2-node combinations (king+queen vs king+jack), decomposing the
overhead with TimingBreakdown.

## 2.24 The first M3 3-node cluster running (2026-08-20)

With the infrastructure, builds and static IPs done, a real 3-node inference
cluster was brought up for the first time. The scheduler (server .9) +
king/queen/jack, over real gRPC.

### Deployment

- The node was built on king (`cargo build --release -p npuforge-node --features
  rknn`, 1m37s, 24MB) → deployed to queen/jack via the development PC
- Model: INT8 `model.rknn` (dba155d2) + `model.toml` on all three boards, hash
  verification passed
- Scheduler: `scheduler.example.toml` (policy round-robin) on server, 50051

### Pilot bench (not formal)

Preflight not run, active cooling (fan on), 12 s. **Conditions were not yet
controlled, so these are not used as settled figures.**

| Concurrency | Throughput | Node inference p50 | Round-trip p50 | Distribution |
|---:|---:|---:|---:|---|
| 6  | 146.3 inf/s | 14.4 ms | 39.8 ms | an even 33.3% |
| 24 | 336.4 inf/s | 22.2 ms | 67.7 ms | an even 33.3% |

Error rate 0%, with round-robin splitting the three nodes exactly in thirds.
About 2.1× the single-node INT8 ceiling of 157 at c24 — **multi-node scaling
actually happens.** The formal S2 with preflight, a concurrency sweep and
duration is separate.

### The three bugs caught this time (all failures that did not look like success, so caught quickly)

**1. A relative `model_file` path in model.toml leads to a load failure (a code bug, unfixed)**

`main.rs` verifies the sha256 against the absolute `PathBuf` that `load_spec`
produced (`:77`), but passes `spec.model_file` (the original relative path
`"model.rknn"`) to `backend.load_model(&spec)` (`:81`). The backend looks for the
file relative to CWD and fails to read it before `rknn_init` →
`status=-2` (a read_file failure and an rknn_init failure are both
NPF_RKNN_ERR_MODEL_LOAD and indistinguishable). RKNN leaves nothing on stderr.
→ **Fixed (2026-08-20).** `main.rs` replaces `spec.model_file` with the absolute
path `load_spec` resolved, immediately before `load_model`. Three nodes load and
register normally from a relative-path `model.toml`, and a bench re-verification
(c24 336 inf/s, 0% errors) passed. The real_device test put an absolute path
directly into spec.model_file and so did not catch this bug — there is no
regression test for the relative path case.

**2. A dead node does not release the NPU context, so a restart fails with status=-2**

Killing a node and immediately restarting it makes rknn_init fail. `pkill -9`
plus a wait of several seconds is needed to clear it properly. Whether the node's
graceful shutdown (ContextPool drop → rknn_destroy) reliably runs on SIGTERM
needs checking.

**3. `pkill -f npuforge-node` killed its own shell — ADR-017 pitfall 1, reproduced**

The cleanup command's shell command line contained the pattern string, so pkill
killed itself and the subsequent commands silently did not run. **Deployment and
cleanup use `pkill` by comm, without `-f`.** I walked straight into the pitfall I
had written into the documents myself.

The 3 nodes and the scheduler are still running. The server firewall rules for
50051/8080/9090 are runtime rules (they disappear on reboot).

## 2.23 The network rework — building and measuring 10G aggregation (2026-08-20)

The equipment §2.22 was waiting on arrived and the M3 network was built. **Every
blocker is resolved.**

### What was introduced

| Equipment | Specification |
|---|---|
| Switch | **NEXI NS-S25G10G-N** — 2.5G×4 + 10G×2, all RJ45 |
| Server | Xeon E5-2630L ×2 (24T) / 16GB / Rocky 9.4 / x86_64 |
| Server NIC | `enp4s0` 10GBASE-T (not DAC/SFP+) |

Port wiring: 1=internet (ipTIME), 2=king, 3=queen, 4=jack, 5=dev PC (a 10G port
but a 1G NIC), 6=server (10G).

### What was encountered

1. **The board IPs changed wholesale.** Being DHCP, they were reassigned
   `.12/.16/.33` → `.3/.4/.5`, and the stale aliases in `~/.ssh/config` meant all
   three nodes failed to connect. Exactly the situation
   `adrs/019-ssh-alias-not-ip.md` warned about. Recovered by updating the config
   and adding an `npuforge-server` alias.

2. **The server did not get a 10G IP.** The cause was neither the cable nor the
   switch but NetworkManager — `enp4s0` was `UP LOWER_UP` (link established) with
   no connection profile, so it never ran DHCP. `nmcli device connect enp4s0`
   immediately obtained `192.168.123.9`. A textbook situation when fitting a new
   NIC on Rocky 9.

3. **The remote iperf3 startup did not come up** — `setsid nohup iperf3 ... &`
   failed silently (`adrs/017` pitfall 2). Resolved by restarting it in the
   absolute path form.

### The measurements

```text
server enp4s0             10000 Mb/s full     ethtool
single king->server       2.34 Gbps           iperf3   (the effective 2.5G ceiling)
3 nodes concurrently ->server  1.70 each, 5.11 Gbps total  nc  (the three streams stayed even)
```

The three streams staying even means **the server's 10G aggregation is not a
bottleneck.** It comfortably accommodates the INT8 three-node RX target of
4.60 Gbps. The detailed judgement is in
`adrs/014-10g-aggregation-separate-scheduler.md`'s build result section.

### What was cleaned up

The measurement firewall runtime rules (5201-5210), temporary listeners and files
were all removed afterwards. The server's permanent state was not changed.

### What remains

- **Static IP pinning** — server (.9) done. The three boards await the pi sudo
  password. Host static was adopted rather than router reservations
  (`infrastructure.md` §2.3)
- Deploy the INT8 model to queen and jack
- Open the server's gRPC firewall

`dealer` (the old scheduler, the laptop at .14) does not respond — it has been
removed. Its role moved to server.

### The static IP method decided (2026-08-20)

**Host NetworkManager static** was chosen rather than a router (ipTIME) DHCP
reservation. The settings stay on the host even if the router changes, which is
better for measurement reproducibility, and pinning the current IP means SSH is
not dropped. The server was applied immediately, being root
(`nmcli con mod enp4s0 ipv4.method manual ...`); the boards need `pi`'s sudo
password. The residual risk (a DHCP pool collision) is in
`infrastructure.md` §2.3.

### The scheduler build path decided (2026-08-20)

The old dealer had no Rust, leaving this undecided. It is settled as server.

- Toolchain `stable`, MSRV 1.85. Server's dnf rust/cargo at **1.92** suffices
- Windows→Linux cross-building is avoided due to linker problems. **A native
  24-thread build on server** is faster and more certain
- `rust cargo gcc gcc-c++ protobuf-compiler git` installed on server (tonic-build
  0.12 requires protoc). GitHub access is fine but foxden is not directly
  reachable, so sources go as a `git archive` tarball over scp
- The node (aarch64) is still built natively on king. Only the scheduler (x86_64)
  is on server

**Pitfall: protoc is not in Rocky 9's default repositories.**
`dnf install protobuf-compiler` fails with "No match", and
`dnf install -y a b c ...` fails entirely if it cannot find even one, so rust was
not installed either. **The CRB repository** has to be enabled
(`dnf config-manager --set-enabled crb`) for protobuf-compiler to appear.

**Build verified (2026-08-20).** `cargo build --release -p npuforge-scheduler
-p npuforge-bench` succeeded.

```text
cargo 1.92.0 / rustc 1.92.0 / libprotoc 3.14.0 / gcc 11.5.0
npuforge-scheduler  25 MB
npuforge-bench      19 MB
config parsing and startup normal (--config configs/scheduler.example.toml)
```

The uncertainty in the scheduler build path is gone. Actual deployment and
startup happen when M3 begins.

## 2.22 State at the point of suspension (2026-08-12, awaiting the 10G scheduler setup)

> **Follow-up: §2.23 (2026-08-20) resolved this wait.** Below is the record at
> the point of suspension.

The M3 real-hardware measurement cannot start without the 10G aggregation setup.
Work stops until then, so the state needed to resume is recorded.

### Board state

| Item | king | queen | jack |
|---|---|---|---|
| SSH alias | `npuforge-k` | `npuforge-q` | `npuforge-j` |
| IP | 192.168.123.12 | .16 | .33 |
| CPU governor | `performance` (made permanent) | same | same |
| Idle NPU temperature | 37.9 °C | 37.9 °C | 38.8 °C |
| Residual load processes | none | none | none |

The three nodes' kernel, `librknnrt.so`, RKNPU driver and model hashes all match.
It stopped with `preflight-check.sh --with-inference` passing every item.

### What was installed on the boards (that was not there originally)

| Node | Addition | Reason |
|---|---|---|
| `king` | The Rust toolchain (rustup) | native `npuforge-node --features rknn` builds. Cross-compilation requires matching an aarch64 sysroot with the RKNN SDK and has many failure points |
| `king` | `protobuf-compiler` | building `npuforge-proto` |
| all 3 | `strace` | syscall decomposition measurement |
| all 3 | `/etc/systemd/system/npuforge-cpu-governor.service` | making the governor permanent |
| all 3 | The C tools in `~/npuforge-rknn-test/` | measurement tools |

**Only `king` has Rust.** It breaks environment matching, but it is build-only
and does not affect runtime. Binaries are built once and deployed to all three
nodes (the same principle as the model).

### Settled figures (on governor=performance)

| Item | Value |
|---|---|
| FP16 8-thread sustained throughput | **84.3 inf/s** (94.5 ms latency) |
| INT8 8-thread sustained throughput | **157.2 inf/s** (50.8 ms latency) |
| INT8 / FP16 ratio | **1.86×** |
| Kernel ioctls per inference | 76 (identical for FP16 and INT8) |
| Node-to-node thermal spread | 5.6 °C, no **NPU** throttling |
| CPU thermal downgrade | A72 2208→816MHz / A53 2016→600MHz (after 60 s of load) |
| NPU temperature under sustained load | 67.5–75.8 °C (on ondemand, 15 min) |
| `want_float=0` effect | INT8 +17.3% / FP16 +15.7%, output a quarter |

The 79.0 / 146.2 inf/s in earlier documents are on `ondemand`. discuss.md §11.

### What to do first when resuming

1. `bash scripts/preflight-check.sh --with-inference`
   - A board may have rebooted. The governor holds but `boot_id` changes
   - If it fails, resolve that item first. Do not measure before it passes
2. After connecting the 2.5G/10G switch, decide the inference network IP range
   and update `advertise_address` (the scheduler is on a 10G SFP+ uplink,
   `02-HARDWARE-SETUP.md` §3.3.2)
3. Build `npuforge-node` on `king` and deploy to all three nodes
4. Re-examine the S2 scalability experiment design — INT8 is **1.545 Gbps** per
   node and **4.636 Gbps** across three. The output is 3.96× the input so RX
   reaches at most 18.4 Gbps. **10G aggregation is required.**
   `02-HARDWARE-SETUP.md` §3.3.2
   (the 1.43/4.3 first written here was an error from calculating Gbps with a
   binary prefix)

### Pitfalls to watch on resuming (encountered this time)

- Reach the boards **by alias, not by IP** (§2.20)
- Remote background execution uses **absolute paths**, and confirm it actually
  came up (§2.21)
- Check processes with `/proc/PID/exe`, not `pgrep -f` (§2.21)
- A heredoc nested with sudo inside ssh fails silently. Send files with `scp`
- Compare temperatures **only when the load profile is the same** (§2.19)

## 2.18 The RTC does not hold

A separate problem was found while querying the boot history.

```text
queen  current boot started  Tue 2025-11-25 18:16:31 UTC
jack   current boot started  Tue 2025-11-25 18:16:31 UTC
king   current boot started  Fri 2025-07-11 18:52:59 UTC
```

On all three nodes, **the system time right after boot is a fixed value in the
past.** There is no RTC battery, or it does not work, so the clock resets when
power is cut. Log timestamps are wrong until NTP synchronises.

### Impact

- Timestamps in logs recorded right after boot cannot be trusted
- Event ordering between nodes cannot be aligned (`02-HARDWARE-SETUP.md` §10)
- A wrong time could be recorded in the benchmark results

### Action

`chrony` has to be enabled and **measurement started only after confirming
synchronisation is complete.** It is included in
`scripts/fix-node-consistency.sh`'s `chrony` stage but has not been run.

The benchmark scripts confirm the following before running.

```bash
chronyc tracking | grep -E "Leap status|System time"
# Leap status must be Normal; wait if it says Not synchronised
```

Since the design has each node carry only the durations it measured in its
response and never compare absolute clocks (§10.1), the measurements themselves
are unaffected. The problem is **log correlation analysis.**

---

# 3.5 Document reorganisation (2026-08-07)

The work history grew long and was split into two documents.

| Document | Role |
|---|---|
| `board-worklog.md` (this document) | **The chronological work history.** Append only. Why it was done that way |
| `infrastructure.md` | **A snapshot of the current state.** What state it is in now |
| `environment-matrix.md` | **Version and hash pinning.** The values needed for reproduction |

For "what state is it in now" read `infrastructure.md`; for "how did it get this
way" read this document.

---

# 4. PC-side changes

Applied to the development PC (`192.168.123.26`) rather than the boards.

| Date | Item | Content |
|---|---|---|
| 2026-08-07 | SSH key | `~/.ssh/id_ed25519_npuforge` generated (no passphrase, for automation) |
| 2026-08-07 | SSH config | `npuforge-k` / `npuforge-q` / `npuforge-j` aliases added. The existing config backed up as `~/.ssh/config.bak.*` |

## 4.1 SSH aliases

```text
npuforge-k -> pi@192.168.123.12  (king)
npuforge-q -> pi@192.168.123.16  (queen)
npuforge-j -> pi@192.168.123.33  (jack)
```

The aliases stay as `npuforge-k/q/j` while only the hostnames became
`king/queen/jack`. Changing the aliases would mean editing every script already
written, so it is tidied up in one go when the inference network is configured.

## 4.2 The sudo execution pattern

The `pi` account requires a password for sudo. Automation uses the following
form.

```bash
ssh npuforge-k 'printf "$NPUFORGE_SUDO_PASS\n" | sudo -S -p "" <command>'
```

Beware the pipe collision pitfall recorded in §2.4.

**Room for improvement.** As sudo calls grow in benchmark automation, a NOPASSWD
sudoers rule limited to specific commands would be better. But that is a
privilege expansion and proceeds only after separate approval.
