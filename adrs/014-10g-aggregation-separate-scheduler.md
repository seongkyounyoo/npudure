# ADR-014. Leave the worker links at 2.5G, raise only aggregation to 10G, and put the scheduler on a separate server

*[한국어 원문](014-10g-aggregation-separate-scheduler.ko.md)*

| | |
|---|---|
| **Status** | accepted (equipment obtained and measured 2026-08-20) |
| **Date** | 2026-08-12 (decision), 2026-08-20 (build and measurement) |
| **Supersedes** | the judgement that "2.5GbE is sufficient as the reference network" |
| **Related** | [ADR-003](003-central-simple-scheduler.md), [ADR-011](011-int8-quantization.md), [ADR-012](012-want-float-zero-blob-v2.md), `docs/02-HARDWARE-SETUP.md` §3.3.2 |

---

## In one line

> Traffic from three nodes converges at one point, the scheduler. **It is that
> confluence, not the worker link (2.5G), that fills up first.** So only
> aggregation is raised to 10G, and a **separate server** that can take a 10G
> NIC becomes the scheduler host.

## Context

### The original calculation went like this

```text
3 nodes 150 FPS x 1.23 MB ~ 184 MB/s ~ 1.5 Gbps  -> exceeds 1GbE, 2.5GbE is enough
```

That calculation set "the reference network is 2.5GbE". **Two things were
wrong.**

**(a) The throughput assumption was stale.** 150 FPS was assumed as the
**total** across three nodes. Measurement says **one** node does 157.2 inf/s at
INT8. Three nodes is 471 inf/s.

**(b) The output direction was ignored.** Only the input was counted. The node
does not postprocess and sends raw tensors back, so the response uses the link
too. With `want_float=1` the response was **3.96×** the request.

On top of that there was a unit error — converting MiB/s to Gbps used the
binary prefix (÷1024). **Network speeds are decimal.**

### The recomputed values

```text
one raw RGB input = 640 x 640 x 3 = 1,228,800 byte

                     per node        3-node total
INT8  157.2 inf/s    1.545 Gbps      4.636 Gbps
FP16   84.3 inf/s    0.829 Gbps      2.486 Gbps
```

**Even FP16's three-node total of 2.486 Gbps exceeds a single 2.5GbE link
(effectively about 2.35 Gbps).** INT8 exceeds it by nearly double.

## Decision

**1. Leave the worker links at 2.5G.** At most 1.545 Gbps per node, which fits.

**2. Raise only the aggregation link to 10G.**

```text
        Benchmark / Scheduler Server
                    |
                  10GbE          <- this is the point
                    |
            2.5G / 10G Switch
              |-- 2.5G -- king
              |-- 2.5G -- queen
              \-- 2.5G -- jack
```

**3. Make the scheduler host a separate server with a PCIe slot.**

**4. Reduce the output alongside.** Even with 10G laid, `want_float=1` puts RX
at 18.38 Gbps and it is still not enough. →
Solved in [ADR-012](012-want-float-zero-blob-v2.md).

**5. Keep 1GbE rather than removing it, as a comparison condition.** Presenting
"the network is the bottleneck" and "it is not" side by side has value as a
bottleneck-analysis result (scenarios S5 and S6).

## Rationale

### Why aggregation rather than the workers

Each node uses only its own link. At most 1.545 Gbps, which fits inside 2.5G.
But **all three nodes' traffic converges in front of the scheduler.** The load
at the confluence is threefold.

```text
king  --1.5G--\
queen --1.5G---+--> 4.6 Gbps --> scheduler   <- impossible on 2.5G
jack  --1.5G--/
```

**Only this point degrades linearly as nodes are added.** In a project measuring
three-node scaling efficiency, if something fills up first as you scale, **you
end up measuring link saturation rather than NPU scaling efficiency.**

### Why a separate server — two reasons overlap

**(1) Symmetry of the measurement conditions.** Running the scheduler on one of
the nodes raises CPU and network load on that node alone. The three nodes'
conditions diverge and the 1/2/3-node comparison is distorted. You could no
longer call it a "like-for-like comparison" in a talk.

**(2) The PCIe slot.** A 10G SFP+ NIC is a PCIe card. The current scheduler
host, `dealer`, is **a laptop with nowhere to put it.**

(1) alone required a separate host, and (2) narrowed "any host" to "a server
with a PCIe slot".

## Build result (2026-08-20)

The equipment was assembled as designed and the bandwidth measured. It came
together as **10GBASE-T (RJ45) rather than DAC/SFP+** — the switch is a NEXI
NS-S25G10G-N (2.5G×4 + 10G×2, all RJ45), so the SFP+ plan became RJ45. No
effect on the conclusion.

```text
server (Rocky 9.4, Xeon x2 24T / 16GB)
  \ enp4s0 10GBASE-T -- measured 10G full (ethtool)
                        |
              NS-S25G10G-N -+ 2.5G - king  .3
                            + 2.5G - queen .5
                            \ 2.5G - jack  .4
```

| Measurement | Value | Tool |
|---|---:|---|
| Server link negotiation | 10000 Mb/s full | ethtool |
| Single king→server | 2.34 Gbps | iperf3 (the effective 2.5G ceiling) |
| **3 nodes concurrently →server** | 1.70 each, **5.11 Gbps total** | nc |

With three nodes concurrent, the three streams **stayed even** — had the server
been the bottleneck the total would have been cut, and it was not. It
comfortably accommodates the INT8 3-node RX target of **4.60 Gbps**. (The
individual 1.70 being below the 2.34 link ceiling is an nc/board-CPU limit, not
a switch or server limit. Actual M3 traffic is gRPC, so this figure is for
infrastructure verification.)

As a side effect the **scheduler host's RAM went from 3 GB (dealer) to 16 GB
(server)**, easing ADR-003's concern about scheduler RSS.

> ⚠️ Because the boards use DHCP, this rework changed their IPs wholesale
> (`.12/.16/.33` → `.3/.4/.5`). The [ADR-019](019-ssh-alias-not-ip.md) situation
> recurred, with stale SSH aliases failing to find the nodes. MAC-based static
> IPs are follow-up work.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| 10G everywhere (workers included) | Wasteful at 1.545 Gbps per node. The boards' NICs are 2.5G anyway |
| 25G or above | Needed to keep `want_float=1`, but reducing the output is cheaper and more correct (ADR-012) |
| Run the scheduler on `king` | Breaks the three nodes' experimental conditions. Allowed for development and demos only, never for official figures |
| Keep 2.5GbE and just measure | **The most dangerous choice.** Link saturation would get reported as scaling efficiency |
| Switch input to JPEG to reduce TX | The decode cost lands on node CPU. CPU is already the bottleneck and this stacks on it. Valid as an S6 comparison item |

## Consequences

**Gained**

- The premise for measuring scaling efficiency holds — the link does not fill
  up first
- The required equipment became clear: a 2.5G/10G switch, a PCIe server, a 10G
  NIC and an SFP+ DAC

**Lost / the cost**

- **M3 was blocked for a while on procurement.** From the decision on
  2026-08-12 to the build on 08-20. A hardware problem, not a code one
- Cost went up — switch, server, NIC, cables

**The biggest consequence of this decision: choosing not to start measuring**

Measuring without the equipment would still produce numbers. **And those numbers
would be wrong.** Scaling efficiency would come out low at three nodes, with the
cause being the link rather than the NPU. Publishing results in that state would
invalidate the project's central claim.

So **the choice was to stop measuring and wait.** That is a decision too.

**New constraints introduced**

- The scheduler host formally joined the experimental equipment list. Changing
  its specification is a change of measurement conditions
- Before starting M3, **measured TX/RX must be recorded rather than calculated**
  (`02-HARDWARE-SETUP.md` §3.3.3). This section's original error was trusting
  calculation alone

## What would overturn this

- **Switching the input format to JPEG** cuts TX roughly tenfold and the whole
  link budget has to be recomputed. 2.5G might suffice in that case — but where
  the decode CPU cost lands has to be considered alongside
- **Implementing node-side postprocessing (NMS)** effectively removes RX. What
  remains is TX at 4.64 Gbps, lowering the requirement → ADR-021
- **Adding more nodes** raises the aggregation requirement proportionally. Five
  nodes at INT8 comes to 7.7 Gbps, leaving no headroom even at 10G
