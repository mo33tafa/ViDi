# vidi

<p>
  <a href="README.md">🇬🇧 English</a>
  &nbsp;·&nbsp;
  <a href="README.fa.md">🇮🇷 فارسی</a>
</p>

`vidi` is a command-line tool for inspecting and troubleshooting Linux and Windows systems. It collects information about the operating system, processes, services, network, hardware and logged-in users, then presents the result as a readable report or JSON.

**Developer:** Mostafa Heshmatpour

> **Project status:** vidi is still under active development. New capabilities and improvements are planned for upcoming releases.

## Installation

Prebuilt 64-bit releases are published for Linux and Windows.

### Linux

```sh
curl --fail --location https://raw.githubusercontent.com/mo33tafa/vidi/main/install.sh | sh
```

When run as root, the installer uses `/usr/local/bin`. For a regular user, it uses `~/.local/bin`.

### Windows PowerShell

```powershell
irm https://raw.githubusercontent.com/mo33tafa/vidi/main/install.ps1 | iex
```

The installer places `vidi.exe` in `%LOCALAPPDATA%\vidi\bin` and adds that directory to the user PATH. Open a new PowerShell window after installation.

## Quick start

```text
vidi --version
vidi get os
vidi get hardware
vidi get time
vidi get list
```

Add `--json` when the output will be consumed by another tool:

```text
vidi --json get hardware
vidi --json get time
```

## Common commands

| Command | Description |
| --- | --- |
| `vidi get os` | OS, kernel, architecture, boot time and uptime |
| `vidi get time` | Local time, UTC, timezone and NTP status |
| `vidi get hardware` | CPU, memory, swap and disks |
| `vidi get hardware --full` | Additional hardware details when available |
| `vidi get process <pid>` | Details for one process |
| `vidi get service <name>` | Details for one service |
| `vidi get network` | Network interfaces and traffic counters |
| `vidi get user` | Current user |
| `vidi get list` | All simple targets |

The detailed command groups are also available:

```text
system       OS and environment information
processes    List, inspect, search and stop processes
services     List and manage services
network      Interfaces, connections, routes, DNS and listening ports
hardware     CPU, memory, storage, sensors and adapters
users        Local accounts, active sessions and groups
```

Other examples:

```text
vidi health
vidi diagnose all
vidi analyze processes
vidi processes list --count 20 --sort memory
vidi network listening
vidi snapshot --output snapshot.json
vidi hash /path/to/file
vidi investigate incident-2026
vidi monitor --duration 30 --interval 1000
```

## Permissions

Some information requires an elevated shell. Use root or Administrator privileges when inspecting another user's processes, managing services, reading hardware details or viewing protected network information.

On Linux:

```sh
sudo vidi services list
sudo vidi processes kill --pid 1234
```

On Windows, open PowerShell with **Run as administrator**.

Missing native tools are reported as unavailable rather than being treated as definitive system information.

## Build from source

Rust is required for local builds:

```sh
cargo build --release
cargo test
```

The binary is written to `target/release/vidi` or `target/release/vidi.exe`.

## Development status

Linux is the primary tested platform. Windows builds are covered by the repository CI workflow, while hardware and service results depend on the tools and permissions available on each machine.

Pushing a version tag such as `v0.1.0` starts the GitHub Actions release workflow and creates Linux and Windows release archives.

## License

MIT
