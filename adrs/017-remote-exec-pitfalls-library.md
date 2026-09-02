# ADR-017. Harden the remote-execution pitfalls into library functions

*[한국어 원문](017-remote-exec-pitfalls-library.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-015](015-preflight-hard-fail.md), [ADR-019](019-ssh-alias-not-ip.md) |

---

## In one line

> There are three pitfalls in running remote commands over `ssh` where
> **failure looks like success**. All three give exit code 0 with empty stderr.
> Rather than being careful every time, they are hardened into functions in
> `scripts/lib/remote.sh`.

## Context

Found while building `preflight-check.sh`. **A check was silently not working.**
It passed with "no residual load" while load was running.

Digging in, there were three pitfalls, and all of them share one property —
**there is no signal at all that something is wrong.**

## Pitfall 1. `pgrep -f` counts itself

`pgrep -f` matches the whole command line. And the command line of the wrapper
ssh sends **contains the pattern string itself.**

```text
bash -c "... pgrep -f \"[s]ustained_load_test|...\" | wc -l"
                       ^^^^^^^^^^^^^^^^^^^^^^^^ this matches
```

The bracket trick (`[s]ustained`) is also neutralised once a form without the
brackets appears on the same command line.

**It is wrong in both directions.**

| Situation | Actual | pgrep reports |
|---|---|---|
| Load running | 1 | **0 (missed)** |
| No load | 0 | **2 (counting its own shell)** |

**The fix**: read the `/proc/PID/exe` symlink. It points at the actual
executable, leaving no room for a shell to get involved.

```bash
n=0
for p in /proc/[0-9]*; do
  case "$(readlink "$p/exe" 2>/dev/null)" in
    *sustained_load_test) n=$((n+1)) ;;
  esac
done
```

## Pitfall 2. `cd DIR && setsid nohup ... &` does not come up

| Form | Result |
|---|---|
| `ssh -n H "cd $DIR && setsid nohup ./prog ... &"` | **does not run** |
| `ssh -n H "setsid nohup $DIR/prog ... &"` | runs |

The `&` applies to the **whole `cd && prog` list**. ssh sends the command and
disconnects immediately, and if the session disappears before the background
subshell gets through `cd` and reaches `setsid`, it dies right there.

Using an absolute path removes the intermediate step, so no race arises.

**The cost is large.** Even on failure the exit code is 0 and stderr is empty.
Without checking, you end up **measuring "the temperature with no load" for
fifteen minutes.**

## Pitfall 3. A heredoc inside ssh nested with sudo does not create the file

Encountered while deploying a systemd unit. This too gave **exit code 0.**

## Decision

**1. Make the avoidance form of all three pitfalls into functions in
`scripts/lib/remote.sh`.** Scripts use those functions rather than calling ssh
directly.

**2. Read `/proc/PID/exe` when counting remote processes.** Do not use
`pgrep -f`.

**3. Background startup uses only the absolute path + `setsid nohup` form.**

**4. Add a step that confirms it is actually running after starting it.** Do not
trust the startup command's exit code.

**5. When adding a new check, break it deliberately and confirm it actually
catches.**

## Rationale

### Point 5 is the heart of this ADR

Pitfall 1 was found precisely because of that procedure. **Had a pass been
trusted at face value, preflight would have remained in place filtering
nothing.**

Check code is especially dangerous. It normally prints only "pass", so nobody
notices when it breaks. It just **gets quieter.**

### Why code rather than documentation

All three pitfalls are the kind you can avoid if you know about them. And yet
this project already has several cases of being caught while knowing better. If
three things have to be recalled every time a remote command is written, one
will eventually be missed.

Making them functions makes **the default path the safe form.**

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Leave it in comments and documentation | Already confirmed not to work |
| Introduce a tool like Ansible | Adds a dependency, and is excessive for a three-machine experimental setup. Problems like pitfall 2 remain regardless |
| Keep a resident agent instead of ssh | That is what `npuforge-node` is. But the measurement scripts have to run independently of the node process |
| Just check the exit code | **All three pitfalls give exit code 0.** Fundamentally does not work |

## Consequences

**Gained**

- New scripts use the safe form by default
- The record of having hit these pitfalls lives next to the code

**Lost / the cost**

- Scripts depend on `lib/remote.sh`. Running them standalone gets harder
- Walking `/proc` is slower than `pgrep` (negligible given how often the checks
  run)

**New constraint introduced**

- **New remote execution has to go through this library.** Calling `ssh`
  directly reopens the pitfalls

## What would overturn this

If a fourth pitfall appears, it gets added here. **There is no reason for the
list to shrink.**
