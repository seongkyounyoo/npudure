# ADR-019. Reach the boards by SSH alias, not by IP

*[한국어 원문](019-ssh-alias-not-ip.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-015](015-preflight-hard-fail.md), [ADR-017](017-remote-exec-pitfalls-library.md) |

---

## In one line

> An IP pinned into a document went stale, so **a node was misdiagnosed as dead**
> and the whole subnet got scanned. `~/.ssh/config` had the correct value all
> along. Access goes only through the `npuforge-k` / `-q` / `-j` aliases.

## Context

On 2026-08-11 `king` could not be reached.

```text
IP written in the document   10.20.0.22
actual IP                    10.20.0.12
```

Believing the node was dead, the subnet was swept. But `~/.ssh/config` had had
**the correct IP from the beginning.** Only the document was stale.

Why this is dangerous. Being unable to connect at all is the better case — you
find out immediately. **The genuinely dangerous case is when another board is at
that IP.**

```text
measure via npuforge-k -> actually attaches to queen -> the measurement finishes normally
                                                     -> the result is recorded as king's
```

It fails quietly. The failure mode this project guards against most.

## Decision

**1. Boards are reached only by SSH alias.**

```text
npuforge-k   king
npuforge-q   queen
npuforge-j   jack
```

**2. Do not write IPs directly in documents or scripts.** The IP lives in one
place, `~/.ssh/config`.

**3. Preflight's **first** check is alias ↔ hostname agreement.** It confirms
that what you attached to really is that board.

**4. Keep the SSH host keys distinct per node.**

## Rationale

### A single source

IPs change. A DHCP lease renews, the network gets reconfigured, a switch gets
replaced. If several documents have to be fixed each time, one will inevitably
be left behind.

`~/.ssh/config` is **the value actually used to connect**, so being wrong shows
up immediately. An IP in a document is used by nobody and stays wrong for a long
time.

### An alias can be wrong too — hence the check

The alias points at an IP, so a reassigned IP can leave the alias pointing at
the wrong board. That is why preflight's check 1 is needed.

```text
ssh npuforge-k hostname   ->  must be "king"
```

That check comes **before the connection-failure check**, because a connection
failure fails loudly while a wrong mapping succeeds quietly.

### Identical host keys make them indistinguishable

`queen` and `jack` currently have identical SSH host keys — apparently from
cloning or copying an image.

In that state, **SSH raises no warning even if a changed IP attaches you to a
different board.** A host key is the device for confirming "is this server the
same server as before", and when two are identical that function is dead.

Having already misdiagnosed a node once, this must not be left alone. **It
remains as an open item in `docs/TODO.md`.**

```bash
ssh npuforge-j 'sudo rm -f /etc/ssh/ssh_host_* && sudo ssh-keygen -A && sudo systemctl restart ssh'
ssh-keygen -R npuforge-j   # clean up known_hosts on the PC
```

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Just manage the IPs in the documents well | Already failed. A value nobody uses goes stale |
| Assign static IPs | Copied into documents it is the same problem again. The alias remains valid on top of static IPs anyway |
| Reach by mDNS / hostname | Does not work in some environments, and the alias is a layer above it so they can coexist |
| Use aliases but skip the check | Does not catch an alias pointing at the wrong board |

## Consequences

**Gained**

- A single source for the IPs
- Preflight catches the accident of a measurement being attributed to the wrong
  node

**Lost / the cost**

- Someone new taking the repository has to create `~/.ssh/config` themselves.
  Stated as a prerequisite in the reproduction procedure

**New constraints introduced**

- **Treat any IP seen in a document with suspicion.** If one is still there, it
  is likely stale
- Until `queen` and `jack`'s host keys are regenerated, an IP reassignment can
  attach you to the wrong board without a warning. **A known risk**

## What would overturn this

Nothing. No reason will arise to pin IPs back into documents.
