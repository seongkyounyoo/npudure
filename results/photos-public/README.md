# Photographs of the rig

*[한국어 원문](README.ko.md)*

Photographs of the actual NPUDure three-node cluster. This is the hardware the
measurements came from.

| File | Content |
|---|---|
| `boards-labeled-01.jpg` `-02.jpg` | Three NanoPi R76S. The J / Q / K labels are `jack` / `queen` / `king` |
| `fans-01.jpg` `-02.jpg` `-03.jpg` | The three 120 mm fans used for the active cooling condition |
| **`cluster-overview-01.jpg` `-02.jpg`** | **The whole setup** — the scheduler host (left), the three nodes with their cooling fans (centre), and the dashboard running (right) |
| `cluster-front.jpg` | The assembled rig from the front. Switch and cabling top right |
| `cluster-side-01.jpg` `-02.jpg` | A low side angle. The boards' LEDs lit under the fans |
| `cluster-top-01.jpg` `-02.jpg` | From above. The switch and power distribution are visible together |
| `server-chassis-01.jpg` `-02.jpg` | Inside the scheduler host desktop (case open) |
| `server-i7-internal-01.jpg` `-02.jpg` | The same host, close up |
| `server-nic-10g-01.jpg` `-02.jpg` | The **Intel X550T 10GBASE-T** seated in its PCIe slot |
| `switch-nexi-01.jpg` `-02.jpg` | The **NEXI NS-S25G10G-N** — the `2.5G Link` and `10G` port markings and their link LEDs |

All have had their EXIF stripped and were resized to 2048px on the long edge.

## The hardware

Specifications and topology are in
[`../../docs/infrastructure.md`](../../docs/infrastructure.md).

- Three nodes — NanoPi R76S (RK3576, 2-core NPU), 2.5GbE each
- Switch — NEXI NS-S25G10G-N (2.5G ×4 + 10G ×2)
- Scheduler host — 10GbE. The point where the three nodes' traffic converges

**10G was a requirement, not a choice.** On INT8 a single node demands
1.545 Gbps, and three nodes' input alone is 4.6 Gbps. The scheduler host is that
confluence, and 2.5G cannot take it. The cable in the `10G` port in
`switch-nexi-*.jpg` is that uplink, and `server-nic-10g-*.jpg` is the other end.

**The fans are not decoration.** Whether cooling is active decides the operating
point under sustained load. With the fans on, the CPU clock is not downgraded
even under load.
→ [`../../docs/experiments/S0_SUSTAINED_LOAD.md`](../../docs/experiments/S0_SUSTAINED_LOAD.md)

> **The scheduler host changed twice.** The 421 measurements were taken on a
> Xeon E5-2630L ×2 server, replaced on 2026-08-26 with a Core i7-4790 desktop.
> The `server-*.jpg` images are the host **after** that replacement. The swap
> changed the baseline throughput; the story is in `infrastructure.md` §3.2.1.
> There are no photographs of the old server in the published set.

## Shots that do not exist yet

- **The whole setup including the switch** — `cluster-overview-*` gets the
  scheduler host, the nodes and the screen into one frame, but the switch is
  hidden behind the cabling. There is a standalone switch shot in
  `switch-nexi-*`.
