/*
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License,
or (at your option) any later version.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>
 */

mod audio;
mod command_line;
mod configuration;
mod hardware;
mod signaling;
mod webrtc;

use crate::configuration::{Configuration, TracingLogLevel};
use crate::hardware::audio_io::AudioSessionManager;
use crate::signaling::signaling_server_manager::SignalingServerManager;
use crate::webrtc::webrtc_session_manager::WebrtcSessionManager;
use clap::Parser;
use nix::fcntl::{flock, open, FlockArg, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::{close, dup2, fork, setsid, ForkResult};
use std::fs::{self, File, OpenOptions};
use std::io;
use std::io::{Seek, SeekFrom, Write};
use std::os::fd::AsRawFd;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::{debug, error};
use tracing_subscriber::prelude::*;

const APPLICATION_VERSION: &str = "0.1.0";
const AGENT_TYPE_NAME: &str = "QSP Agent";

fn main() {
    let cli = command_line::Cli::parse();
    let config_path = cli.config.clone().unwrap_or("config.toml".parse().unwrap());
    let config = match configuration::load_config(config_path) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Failed to load configuration: {error}");
            return;
        }
    };

    if let Err(error) = daemonize(&cli) {
        eprintln!("Failed to daemonize: {error}");
        return;
    }

    if let Err(error) = init_tracing(cli.daemon, config.agent_log_level) {
        eprintln!("Failed to initialize tracing: {error}");
        return;
    }

    let _lock_file = match lock_file(&config.lock_file) {
        Ok(lock_file) => lock_file,
        Err(error) => {
            error!(
                "Failed to lock file '{}': {}",
                config.lock_file.display(),
                error
            );
            return;
        }
    };

    if let Err(error) = write_pid_file(&config.pid_file) {
        error!(
            "Failed to write PID file '{}': {}",
            config.pid_file.display(),
            error
        );
        return;
    }

    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            error!("Failed to create Tokio runtime: {}", error);
            return;
        }
    };

    runtime.block_on(start_server(config));
    debug!("End !");
}

fn init_tracing(
    daemon: bool,
    log_level: TracingLogLevel,
) -> Result<(), Box<dyn std::error::Error>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::builder()
            .with_default_directive(default_log_directive(log_level))
            .from_env_lossy()
    });

    if daemon {
        init_daemon_tracing(filter)
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .try_init()?;
        Ok(())
    }
}

fn default_log_directive(log_level: TracingLogLevel) -> tracing_subscriber::filter::Directive {
    match log_level {
        TracingLogLevel::Error => tracing_subscriber::filter::LevelFilter::ERROR.into(),
        TracingLogLevel::Warn => tracing_subscriber::filter::LevelFilter::WARN.into(),
        TracingLogLevel::Info => tracing_subscriber::filter::LevelFilter::INFO.into(),
        TracingLogLevel::Debug => tracing_subscriber::filter::LevelFilter::DEBUG.into(),
        TracingLogLevel::Trace => tracing_subscriber::filter::LevelFilter::TRACE.into(),
    }
}

#[cfg(target_os = "linux")]
fn init_daemon_tracing(
    filter: tracing_subscriber::EnvFilter,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_journald::layer()?)
        .try_init()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn init_daemon_tracing(
    filter: tracing_subscriber::EnvFilter,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_oslog::OsLogger::new(
            "org.ham-qsp.qsp-agent",
            "daemon",
        ))
        .try_init()?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn init_daemon_tracing(
    filter: tracing_subscriber::EnvFilter,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_etw::LayerBuilder::new("HamQsp.QspAgent").build()?)
        .try_init()?;
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn init_daemon_tracing(
    _filter: tracing_subscriber::EnvFilter,
) -> Result<(), Box<dyn std::error::Error>> {
    Err(Box::new(io::Error::new(
        io::ErrorKind::Unsupported,
        "daemon tracing is not supported on this platform",
    )))
}

#[cfg(unix)]
fn daemonize(cli: &command_line::Cli) -> Result<(), io::Error> {
    if !cli.daemon {
        return Ok(());
    }
    debug!("Daemonized");
    match unsafe { fork() }.map_err(io::Error::other)? {
        ForkResult::Parent { .. } => std::process::exit(0),
        ForkResult::Child => {}
    }

    setsid().map_err(io::Error::other)?;

    let devnull = open("/dev/null", OFlag::O_RDWR, Mode::empty()).map_err(io::Error::other)?;
    dup2(devnull, 0).map_err(io::Error::other)?;
    dup2(devnull, 1).map_err(io::Error::other)?;
    dup2(devnull, 2).map_err(io::Error::other)?;
    if devnull > 2 {
        close(devnull).map_err(io::Error::other)?;
    }

    Ok(())
}

#[cfg(not(unix))]
fn daemonize(cli: &command_line::Cli) -> Result<(), io::Error> {
    if cli.daemon {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "daemon mode is only supported on unix platforms",
        ))
    } else {
        Ok(())
    }
}

fn write_pid_file(path: &Path) -> Result<(), io::Error> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, format!("{}\n", std::process::id()))
}

fn lock_file(path: &Path) -> Result<File, io::Error> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)?;

    flock(file.as_raw_fd(), FlockArg::LockExclusiveNonblock).map_err(io::Error::other)?;
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(format!("{}\n", std::process::id()).as_bytes())?;

    Ok(file)
}

async fn start_server(config: Configuration) {
    let transceiver_manager =
        hardware::transceiver::transceiver_manager::TransceiverManager::new(config.clone())
            .unwrap();

    let audio_session_manager = Arc::new(Mutex::new(AudioSessionManager::new()));
    let webrtc_session_manager = Arc::new(WebrtcSessionManager::new(
        audio_session_manager,
        transceiver_manager,
    ));

    let signal_server_session =
        SignalingServerManager::new(config.clone(), webrtc_session_manager.clone());

    signal_server_session
        .start(config.signaling_server.url)
        .await;
}
