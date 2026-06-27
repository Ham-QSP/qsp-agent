# QSP Agent

QSP Agent is a Rust daemon that connects a local radio transceiver to remote QSP
clients through a signaling server and WebRTC.

It is intended to run close to the hardware:
- it talks to the transceiver through Hamlib
- it captures local audio input
- it exposes the radio to remote clients through the QSP signaling flow

## What It Does

QSP Agent combines three responsibilities:

1. It connects to the QSP signaling server and authenticates itself as an agent.
2. It controls the local transceiver through Hamlib CAT commands.
3. It creates WebRTC sessions for remote clients and streams audio/control data.

## Repository Layout

- `qsp-agent/`: main daemon crate
- `lib-hamlib/`: Rust binding crate around Hamlib
- `package/`: Debian and systemd packaging assets

## Requirements

### Runtime Requirements

- A supported radio reachable through Hamlib
- A working audio input device
- Access to the QSP signaling server

### Linux Build Dependencies

For local builds on Debian or Ubuntu, install:

```bash
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  libasound2-dev \
  libhamlib-dev \
  libopus-dev \
  pkg-config
```

## Build And Test

Build the full workspace:

```bash
cargo build --workspace --locked
```

Run tests:

```bash
cargo test --workspace --locked
```

Check formatting:

```bash
cargo fmt --all --check
```

Run Clippy:

```bash
cargo clippy --workspace --all-targets
```

## Configuration

The agent reads a TOML configuration file. By default it looks for `config.toml`
in the current working directory. A different file can be passed with
`--config`.

A packaged example configuration is provided in [`package/config.toml`](/Users/florian/dev/qsp/qsp-remote-agent/package/config.toml:1).

### Minimal Example

```toml
name = "My Station"
description = "Remote HF station"

[signaling_server]
url = "ws://signaling.example.net/server/session"
agentId = "YOUR_AGENT_ID"
agentSecret = "YOUR_AGENT_SECRET"

[transceiver]
model = 1

[transceiver.port]
rig_pathname = "/dev/ttyUSB0"
serial_speed = "115200"
```

### Configuration Reference

#### Top-Level Fields

- `name`: displayed agent name
- `description`: displayed agent description
- `agentLogLevel`: optional default log level. Allowed values: `Error`, `Warn`, `Info`, `Debug`, `Trace`
- `pidFile`: optional PID file path. Default: `qsp-agent.pid`
- `lockFile`: optional lock file path. Default: `qsp-agent.lock`

#### `[signaling_server]`

- `url`: WebSocket URL of the signaling server
- `agentId`: registered agent identifier
- `agentSecret`: secret used to authenticate the agent
- `connectionRetryDelaySeconds`: optional retry delay sequence in seconds

If `connectionRetryDelaySeconds` is omitted, the default sequence is:

```toml
[1, 1, 3, 5, 15, 30, 60]
```

The last value is reused indefinitely for later retries.

#### `[transceiver]`

- `model`: Hamlib rig model number
- `hamlibDebugLevel`: optional Hamlib log level. Allowed values: `None`, `Bug`, `Err`, `Warn`, `Verbose`, `Trace`, `Cache`
- `statePollingInterval`: transceiver polling interval in milliseconds. Default: `1000`

#### `[transceiver.port]`

This section is passed directly to Hamlib configuration tokens. Typical values
depend on your rig and connection type, for example:

- `rig_pathname`
- `serial_speed`
- network transport settings for TCP-connected radios

Use the appropriate Hamlib parameters for your hardware.

## Running The Agent

Run in the foreground:

```bash
cargo run -p qsp-agent -- --config ./config.toml
```

Or run the compiled binary directly:

```bash
./target/debug/qsp-agent --config ./config.toml
```

### CLI Options

- `-c, --config <CONFIG_PATH>`: path to the TOML configuration file
- `-d, --daemon`: detach into the background on Unix platforms

## Logging

- In foreground mode, logs are written to the console.
- In daemon mode, logs are written to the platform logging backend:
  - Linux: `systemd-journald`
  - macOS: `os_log`
  - Windows: ETW

If `RUST_LOG` is set, it overrides the configured default log level.

Example:

```bash
RUST_LOG=debug ./target/debug/qsp-agent --config ./config.toml
```

## Daemon Mode

On Unix, the agent can detach itself with:

```bash
./target/debug/qsp-agent --config ./config.toml --daemon
```

This mode is useful for manual deployments. When using systemd, do not use
`--daemon`; the packaged service runs the agent in the foreground.

## Debian And Ubuntu Packaging

The repository includes Debian packaging assets under `package/`.

The generated package installs:

- binary: `/usr/bin/qsp-agent`
- configuration: `/etc/qsp-agent/config.toml`
- systemd unit: `/lib/systemd/system/qsp-agent.service`
- man page: `/usr/share/man/man1/qsp-agent.1.gz`

The package also creates a dedicated system user and group named `qsp-agent`.

### Packaged Service Behavior

The packaged systemd service:

- runs as `qsp-agent:qsp-agent`
- uses `/var/lib/qsp-agent` as working directory
- uses `/run/qsp-agent` for runtime files
- restarts on failure
- expects `/etc/qsp-agent/config.toml` to exist

The unit file is available in [`package/systemd/qsp-agent.service`](/Users/florian/dev/qsp/qsp-remote-agent/package/systemd/qsp-agent.service:1).

### Build Debian Packages

The repository includes a packaging script:

```bash
package/build-deb.sh
```

It expects a release binary to already exist at `target/release/qsp-agent`.

Build manually:

```bash
cargo build --release --locked -p qsp-agent
package/build-deb.sh
```

Generated packages are written to `dist/`.

## Systemd Usage

After installing the Debian package:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now qsp-agent.service
```

Inspect logs:

```bash
journalctl -u qsp-agent.service
```

If your radio is exposed through `/dev/tty*` or `/dev/serial/*`, the
`qsp-agent` user may need access to a device group such as `dialout`.

## CI And Release Artifacts

GitHub Actions performs:

- workspace build and tests
- Debian package builds for:
  - `amd64`
  - `arm64`
  - `armhf`

On tags matching `v*`, the workflow publishes the generated `.deb` packages as
GitHub release assets.

## Operational Notes

- The signaling connection automatically retries with backoff.
- The agent keeps a lock file to avoid running multiple instances on the same
  configuration.
- Audio capture depends on a valid local input device and a supported audio
  format.
- Transceiver behavior depends heavily on Hamlib support and rig-specific
  configuration tokens.

## License

This project is distributed under the GNU General Public License, version 3 or
later. See [`COPYING`](/Users/florian/dev/qsp/qsp-remote-agent/COPYING:1).
