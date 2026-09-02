# Thermal comparison — 2026-08-11 10:48

*[한국어 원문](README.ko.md)*

A measurement to re-check, under controlled conditions, the 08-10 observation
(§2.19) that `king` ran 19 °C hotter on the NPU than the other two.

## Conditions

| Item | Value |
|---|---|
| Load | `sustained_load_test`, fixed at 8 threads, 900 s |
| Model | `yolov8n-fp16.rknn` (sha 459602ea70479c1c) |
| Start | all three boards simultaneously |
| Cooling | fanless, no desk fan |
| Logger | 1-second intervals (temperature, CPU/NPU clocks, input voltage) |

The binary and model hashes were verified identical across the three boards
beforehand (`preflight.txt`). Comparing `boot_id` before and after the run
showed no reboot (no `INVALID.txt`).

## Results

| Board | NPU mean | NPU peak | Throughput |
|---|---|---|---|
| king | 73.0 °C | 75.8 °C | 80.5 inf/s |
| queen | 67.5 °C | 70.2 °C | 77.7 inf/s |
| jack | 72.6 °C | 74.8 °C | 77.8 inf/s |

**Maximum spread 5.6 °C. Never exceeded 90 °C. No NPU clock drop (all 928
samples at 950 MHz).**

The 19 °C gap did not reproduce. The interpretation is in
`docs/board-worklog.md` §2.19.

## Files

- `preflight.txt` — the pre-run record of hostnames, hashes and boot_id
- `baseline.txt` — the idle baseline
- `<board>-temp.log` — the 1-second sensor log
- `<board>-load.log` — throughput at 10-second intervals plus the final summary

Re-analyse: `python scripts/analyze-thermal.py results/thermal-20260811-104811`
