# Security

## Reporting a vulnerability

Report it privately, not as a public issue:

- **[Open a private report](https://github.com/rengwu/chartr/security/advisories/new)**
  through GitHub's private vulnerability reporting (Security → Report a
  vulnerability), or
- email **johngohrw@gmail.com**.

Whatever you can share helps: the version or commit, what an attacker gets, and
the shortest path you know to reproduce it. A rough description is worth
reporting — don't wait until you have a polished writeup.

## What to expect

chartr is a one-maintainer alpha project, so this is a promise about conduct, not
a service level:

- An acknowledgement, normally within a few days.
- An honest answer on whether it is a bug, and if so what the fix looks like and
  roughly when — including "not soon", if that is the truth.
- Fixes land on `main` and in the next release. There is no back-porting to older
  tags.
- You will be **credited** in the release notes and in any advisory, under
  whatever name you give, unless you ask not to be.
- Nothing is published while you are still working with us, and we will not
  disclose before a fix without telling you first.

There is no bug bounty. Nobody is paid for this project.

## What is already known

Two things are by design, so a report about them will be closed as such — though
a way to *escape* either one is very much a vulnerability:

- **There is no authentication.** Reaching the port is the whole of the access
  check. Binding to anything other than loopback (`-addr :9000`) therefore hands
  shell and agent-spawn access to everyone who can reach that port; chartr warns
  at startup when it does, and the README says so at the flag.
- **chartr runs the agents and shells you ask it to, as you.** A process already
  running under your account is not something chartr defends against — it does
  not need chartr to reach your files.

The boundary chartr does defend is the browser: a web page on another origin
must not be able to drive the cockpit, read the model, or reach a terminal.
Anything that crosses that is in scope.
