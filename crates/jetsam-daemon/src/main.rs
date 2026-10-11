//! `jetsamd` — the Jetsam usage daemon and its small CLI.

#![forbid(unsafe_code)]

use std::sync::Arc;
use std::time::Duration;

use jetsam_core::alpm_backend::AlpmBackend;
use jetsam_core::backend::Backend;
use jetsam_core::protocol::{DaemonStatus, Method, OkData};

use jetsam_daemon::client::Client;
use jetsam_daemon::collector::Collector;
use jetsam_daemon::config::Config;
use jetsam_daemon::store::Store;
use jetsam_daemon::{ipc, logging, now_epoch, paths, SharedState};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HELP: &str = "\
jetsamd - Jetsam usage daemon

USAGE:
    jetsamd [COMMAND]

COMMANDS:
    run         Run the daemon (default)
    once        Take a single sample and print a summary
    status      Print daemon status
    usage [N]   Print the N most-launched packages (default 20)
    ignore <P>  Add a package to the ignore list
    unignore <P>  Remove a package from the ignore list
    ignore-list Print the ignore list
    opt-in      Enable usage collection
    opt-out     Disable usage collection
    wipe        Delete all collected usage data
    help        Print this help
    version     Print the version

ENVIRONMENT:
    JETSAM_LOG  error | warn | info | debug (default info)
";

fn main() {
    logging::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("run");
    let rest = args.get(1..).unwrap_or(&[]);

    let result = match command {
        "run" => cmd_run(),
        "once" => cmd_once(),
        "status" => cmd_status(),
        "usage" => cmd_usage(rest.first()),
        "ignore" => cmd_set_ignored(rest.first(), true),
        "unignore" => cmd_set_ignored(rest.first(), false),
        "ignore-list" => cmd_ignore_list(),
        "opt-in" => cmd_set_opt_in(true),
        "opt-out" => cmd_set_opt_in(false),
        "wipe" => cmd_wipe(),
        "help" | "-h" | "--help" => {
            print!("{HELP}");
            Ok(())
        }
        "version" | "-V" | "--version" => {
            println!("jetsamd {VERSION}");
            Ok(())
        }
        other => Err(format!("unknown command: {other}\n\n{HELP}").into()),
    };

    if let Err(err) = result {
        logging::error(format!("{err}"));
        std::process::exit(1);
    }
}

type AppResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// Build the usage store, seeding the opt-in state from the config on first run.
fn open_store(config: &Config) -> AppResult<Store> {
    let store = Store::open(&paths::db_path())?;
    if !store.opt_in_recorded()? {
        store.set_opt_in(config.opted_in)?;
    }
    Ok(store)
}

fn build_collector() -> AppResult<Collector> {
    let backend = AlpmBackend::open_default()?;
    let index = backend.ownership()?;
    logging::info(format!("ownership index: {} entries", index.len()));
    Ok(Collector::new(index))
}

fn cmd_run() -> AppResult {
    let config_path = paths::config_path();
    let mut config = Config::load(&config_path)?;
    config.sanitize();
    let store = open_store(&config)?;
    let started_at = now_epoch();

    let collector = match build_collector() {
        Ok(collector) => Some(collector),
        Err(err) => {
            logging::error(format!(
                "could not build the ownership index ({err}); IPC will run but no usage will be recorded"
            ));
            None
        }
    };

    let state = Arc::new(SharedState::new(
        store,
        config.clone(),
        config_path,
        VERSION,
        started_at,
    ));

    // Bind before spawning the server so a second daemon fails fast.
    let listener = ipc::bind()?;
    let ipc_state = Arc::clone(&state);
    std::thread::spawn(move || {
        if let Err(err) = ipc::serve_with(listener, ipc_state) {
            logging::error(format!("ipc server stopped: {err}"));
        }
    });

    if config.opted_in {
        logging::info("usage collection enabled");
    } else {
        logging::info("usage collection disabled (opt in from the app or `jetsamd opt-in`)");
    }

    let mut collector = collector;
    let mut last_pss: i64 = 0;
    let mut last_prune: i64 = 0;
    loop {
        let cfg = state.lock_config()?.clone();
        let now = now_epoch();
        let opted_in = state.lock_store()?.is_opted_in()?;

        if opted_in {
            if let Some(collector) = collector.as_mut() {
                let include_pss =
                    cfg.track_pss && now.saturating_sub(last_pss) >= cfg.pss_interval_secs as i64;
                let outcome = collector.scan(include_pss);
                if include_pss {
                    last_pss = now;
                }
                if !outcome.launches.is_empty() || !outcome.uptime.is_empty() {
                    state.lock_store()?.record_sample(&outcome, now)?;
                }
            }
            if now.saturating_sub(last_prune) >= 3_600 {
                let store = state.lock_store()?;
                match store.prune(cfg.retention_days, cfg.launch_event_retention_days, now) {
                    Ok(0) => {}
                    Ok(removed) => logging::debug(format!("pruned {removed} rows")),
                    Err(err) => logging::warn(format!("prune failed: {err}")),
                }
                last_prune = now;
            }
        }

        std::thread::sleep(Duration::from_secs(cfg.sampling_interval_secs));
    }
}

fn cmd_once() -> AppResult {
    let config_path = paths::config_path();
    let mut config = Config::load(&config_path)?;
    config.sanitize();
    let store = open_store(&config)?;
    let opted_in = store.is_opted_in()?;
    let mut collector = build_collector()?;
    let outcome = collector.scan(config.track_pss);
    let now = now_epoch();

    if opted_in {
        let mut store = store;
        store.record_sample(&outcome, now)?;
    }

    println!(
        "{} sample: {} processes examined, {} attributed, {} launches, {} uptime rows, {} pss rows",
        if opted_in { "recorded" } else { "dry-run" },
        outcome.examined_processes,
        outcome.attributed_processes,
        outcome.launches.len(),
        outcome.uptime.len(),
        outcome.pss.len()
    );
    if !opted_in {
        println!("not opted in; nothing was written (use `jetsamd opt-in`)");
    }
    Ok(())
}

fn connect() -> AppResult<Client> {
    Client::connect_default().map_err(|err| {
        format!(
            "could not reach the daemon at {} ({err}); is jetsamd running?",
            Client::default_socket_path().display()
        )
        .into()
    })
}

fn cmd_status() -> AppResult {
    let mut client = connect()?;
    let OkData::Status(status) = client.request(Method::Status)? else {
        return Err("unexpected response".into());
    };
    print_status(&status);
    Ok(())
}

fn print_status(status: &DaemonStatus) {
    println!("jetsamd {}", status.daemon_version);
    println!("protocol:          v{}", status.protocol_version);
    println!("opted in:          {}", status.opted_in);
    println!("sampling interval: {}s", status.sampling_interval_secs);
    println!("pss interval:      {}s", status.pss_interval_secs);
    println!("packages observed: {}", status.packages_observed);
    println!("ignored:           {}", status.ignored_count);
    println!("days of history:   {}", status.days_of_history);
    println!("daemon uptime:     {}s", status.uptime_secs);
}

fn cmd_usage(limit_arg: Option<&String>) -> AppResult {
    let limit: u32 = match limit_arg {
        Some(raw) => raw.parse().map_err(|_| format!("invalid limit: {raw}"))?,
        None => 20,
    };
    let mut client = connect()?;
    let OkData::UsageList(list) = client.request(Method::TopUsage { limit })? else {
        return Err("unexpected response".into());
    };
    if list.is_empty() {
        println!("no usage recorded yet");
        return Ok(());
    }
    println!("{:<30} {:>10} {:>12}", "PACKAGE", "LAUNCHES", "UPTIME(s)");
    for info in list {
        println!(
            "{:<30} {:>10} {:>12}",
            info.name, info.launches, info.uptime_secs
        );
    }
    Ok(())
}

fn cmd_set_ignored(name: Option<&String>, ignored: bool) -> AppResult {
    let name = name.ok_or("expected a package name")?;
    let mut client = connect()?;
    client.request(Method::SetIgnored {
        name: name.clone(),
        ignored,
    })?;
    println!("{} {}", if ignored { "ignored" } else { "unignored" }, name);
    Ok(())
}

fn cmd_ignore_list() -> AppResult {
    let mut client = connect()?;
    let OkData::Names(names) = client.request(Method::IgnoreList)? else {
        return Err("unexpected response".into());
    };
    if names.is_empty() {
        println!("ignore list is empty");
    } else {
        for name in names {
            println!("{name}");
        }
    }
    Ok(())
}

fn cmd_set_opt_in(enabled: bool) -> AppResult {
    let mut client = connect()?;
    client.request(Method::SetOptIn { enabled })?;
    println!(
        "usage collection {}",
        if enabled { "enabled" } else { "disabled" }
    );
    Ok(())
}

fn cmd_wipe() -> AppResult {
    let mut client = connect()?;
    client.request(Method::Wipe)?;
    println!("all collected usage data deleted");
    Ok(())
}
