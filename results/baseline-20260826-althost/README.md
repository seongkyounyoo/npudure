# Baseline reproduction — after the scheduler host swap (2026-08-26)

*[한국어 원문](README.ko.md)*

**This directory is not counted towards the 421 measurements.** The scheduler
host is different.

`scripts/count-runs.sh` sees the `-althost` suffix and counts it separately.

## What was measured

The old server (Xeon E5-2630L ×2, 24T) was physically replaced and things moved
to a spare desktop (Core i7-4790, 8T). **The three nodes, the switch, the model
and the binaries are unchanged; only the scheduler host differs.** These are the
runs checking whether the baseline reproduces.

```text
npuforge-bench --scheduler http://127.0.0.1:50051 --model yolov8n \
               --concurrency 36 --duration 60 --policy sanity
```

## Results

| | run1 | run2 | run3 | old server baseline |
|---|---:|---:|---:|---:|
| throughput (inf/s) | 360.5 | 362.5 | 357.2 | **~391** |
| p50 (ms) | 93.2 | — | — | ~86 |
| error_rate | 0 | 0 | 0 | 0 |

**It did not reproduce (−7.5%).** The three runs are consistent, so it is not
chance.

## The cause — host CPU

Server CPU during measurement was 82.2% (across 8 threads). Under the old
server's conditions it was 42%.

```text
scheduler          45.3%  ~ 3.6 cores
other (bench+kernel) 36.9%  ~ 2.9 cores
```

The nodes are fine — NPU inference p50 28.3 ms, distribution an even 33.3%, 0
errors. The application queue is empty too, with `scheduler_queue` 0.00 ms and
`scheduler_route` 0.01 ms. All the added time is in the transport sections.

That is, what S3.9a excluded (the application queue) still stands, and what
narrowed is **the host CPU outside it.** The measurement structure — **the bench
client runs on the same host as the scheduler** — amplifies this.

→ `../../docs/infrastructure.md` §3.2.1 ·
`../../docs/environment-matrix.md` §10.2
→ On conditioning an exclusion verdict, `../../docs/experiments/README.md` §2

## How to use this data

- **Do not compare directly with the old server's values.** The host differs.
- If measurement continues on the new server, **re-lay the baseline here.**
- The 421 measurements were taken on the old server and stand as recorded. They
  are not retroactively edited.

## A caveat when reading

In `stage_breakdown`, `network_to_node` and `network_to_client` **hold the same
value.** They are not two separately measured directions but a derived figure:
the round trip minus what the node reported, split in half. Quoting one alone
halves the apparent transport cost.
