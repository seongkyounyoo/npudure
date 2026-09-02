# Host inventory

*[한국어 원문](README.ko.md)*

The scheduler hosts' hardware specifications, kept exactly as the machine
collected them.

| File | Host | Period |
|---|---|---|
| `server-xeon-e5-2630l-20260826.md` | **Dell PowerEdge R620** / Xeon E5-2630L ×2 | 2026-08-20 – 08-26 (**the 421 measurements**) |
| `server-i7-4790-20260826.md` | Core i7-4790 / ASUS H81M-K | 2026-08-26 – |

## Why this exists

**The old server's (Xeon E5-2630L ×2) specification had not been kept.** It is
the equipment the 421 measurements came from, and the documents recorded only
the CPU, RAM capacity and NIC name — no motherboard, RAM type, disk model or
PCIe information.

It was belatedly collected on 2026-08-26 by powering that server back up. **We
were lucky** — the equipment was still within reach. By then the OS had moved
from 9.4 to 9.8 and the 10G card had been removed. **A belated collection cannot
fully restore the state at the time.**

The boards had `collect-node-info.sh`; the hosts did not.
`server-profile-collect.sh` is a performance profiler (S3.9a), not an inventory
tool.

## Collection

```bash
ssh <host> 'bash -s' < scripts/collect-host-info.sh > docs/hosts/<name>-<date>.md
```

**When changing hosts, run it before deployment.** Serial numbers, asset tags
and UUIDs are not collected — what reproduction needs is the model name and
specification, not a unit identifier.
