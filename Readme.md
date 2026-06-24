# QSP Remote agent
The agent program expose the radio for external connection.
The link between the agent the clients is done using a signaling
server.

By default, `qsp-agent` runs in the foreground. Use `--daemon` or `-d` to start it as a daemon on Unix platforms.

The PID file path can be configured with `pidFile`. If omitted, it defaults to `qsp-agent.pid` in the current working directory.

The lock file path can be configured with `lockFile`. If omitted, it defaults to `qsp-agent.lock` in the current working directory.

When running in the foreground, logs are emitted to the console. In daemon mode, logs are sent to the platform logging system: `systemd-journald` on Linux, `os_log` on macOS, and ETW on Windows.

The default tracing level can be configured with `agentLogLevel` using `error`, `warn`, `info`, `debug`, or `trace`. If omitted, it defaults to `error`. `RUST_LOG` still overrides this default when set.
