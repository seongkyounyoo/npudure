# Node hardware inventory

*[한국어 원문](README.ko.md)*

The raw output `scripts/collect-node-info.sh` gathered from the three boards.
It is the basis for environment matching verification
(`preflight-check.sh`).

## Redacted values

**Unit-identifying information was replaced with `<redacted>`** (2026-08-27).

| Field | Why |
|---|---|
| `serial` | a board unit identifier. Not needed for reproduction |
| `*_mac` | interface MACs. Not needed for reproduction |
| `ssh_host_*_fp` | SSH host key fingerprints. No reason to publish them |

**They are not secrets.** They are simply values unused in reproduction, with no
reason to remain in a public snapshot. **The values reproduction does need** —
model names, kernel, driver hashes, clocks, temperatures — **are all still
there.**

The collection script still gathers them. They are needed when diagnosing your
own equipment. There is just no reason for someone else's equipment values to
sit in a public repository.
