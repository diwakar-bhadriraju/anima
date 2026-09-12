//! `anima-run`: experiment harness CLI (step 9).
//! Subcommands: run | replay | report.

mod config;
mod env;
mod harness;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "anima-run", about = "ANIMA experiment harness")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run an experiment from a config TOML.
    Run {
        #[arg(long)]
        config: PathBuf,
        /// Attach the browser viz server (default: headless at max speed).
        #[arg(long, default_value_t = false)]
        live: bool,
    },
    /// Replay a recorded telemetry file through the live protocol.
    Replay {
        /// Path to telemetry.jsonl.
        path: PathBuf,
        #[arg(long, default_value_t = 8788)]
        port: u16,
        /// Target ticks/s (0 = max).
        #[arg(long, default_value_t = 1000.0)]
        speed: f32,
    },
    /// Re-analyze an existing run dir (writes metrics.json + report.md).
    Report { dir: PathBuf },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Run { config, live } => {
            let cfg = config::ExpConfig::parse(&config).unwrap_or_else(|e| {
                eprintln!("config error: {e}");
                std::process::exit(2);
            });
            println!(
                "E1 setup: {} input / {} internal / {} output, seed {}",
                cfg.organism.n_input_channels, cfg.organism.n_internal, cfg.organism.n_output, cfg.run.seed
            );
            match harness::run(cfg, &config, live) {
                Ok(outcome) => {
                    println!("run dir: {}", outcome.dir.display());
                    println!("ended: {}", outcome.reason);
                    // Auto-report from telemetry.
                    match report::regenerate(&outcome.dir) {
                        Ok(_) => println!("report: {}/report.md", outcome.dir.display()),
                        Err(e) => eprintln!("report generation failed: {e}"),
                    }
                }
                Err(e) => {
                    eprintln!("run failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        Command::Replay { path, port, speed } => {
            if let Err(e) = anima_viz::replay(&path, port, speed) {
                eprintln!("replay failed: {e}");
                std::process::exit(1);
            }
        }
        Command::Report { dir } => match report::regenerate(&dir) {
            Ok(_) => println!("wrote {}/metrics.json and {}/report.md", dir.display(), dir.display()),
            Err(e) => {
                eprintln!("report failed: {e}");
                std::process::exit(1);
            }
        },
    }
}

mod report {
    use std::path::Path;

    use anima_telemetry::report::generate_report;

    /// Analyze a run dir: telemetry.jsonl (+snapshots) → metrics.json + report.md.
    pub fn regenerate(dir: &Path) -> std::io::Result<()> {
        let events = anima_telemetry::read_telemetry(&dir.join("telemetry.jsonl"))?;
        let metrics = anima_telemetry::analyze(&events);
        serde_json::to_writer_pretty(
            std::fs::File::create(dir.join("metrics.json"))?,
            &metrics,
        )
        .map_err(std::io::Error::other)?;

        let exp_id = events
            .first()
            .map(|e| e.exp_id.clone())
            .unwrap_or_else(|| "unknown".into());
        let (protocol_summary, config_summary, follow_up) = protocol_for(&exp_id);
        let md = generate_report(&exp_id, &events, &protocol_summary, &config_summary, &follow_up);
        std::fs::write(dir.join("report.md"), md)?;
        Ok(())
    }

    fn protocol_for(exp_id: &str) -> (String, String, String) {
        match exp_id {
            "e1" => (
                "STDP-driven assembly formation: repeated presentation of \
                 patterns A/B/C forms pattern-selective assemblies (assembly \
                 score + selectivity); a novel pattern D (S2) produces a \
                 response distinguishable from learned patterns (novelty, \
                 instrumentation-computed U6a); S3 re-tests retention."
                    .into(),
                "- Organism: 24 input (A/B/C × 8), 40 internal, 12 output; caps 200/2000.\n\
                 - Curriculum: S0 5 s silence; S1 3×120 interleaved 500 ms/20 Hz + 1.5 s off; \
                 S2 30×D (A+C co-activation); S3 15×A/B/C re-test.\n\
                 - Plasticity: pairwise additive STDP (D7), always-on gate (U4a).".into(),
                "Next: E2 — STDP bound variants A/B (multiplicative vs additive; U2). \
                 Followed by E3 adaptation, E4 birth triggers, E5 learning gates, \
                 E6 reward modulation, E7 memory mechanisms (docs/unknowns-registry.md).".into(),
            ),
            other => (
                format!("See protocol for {other} (docs/)."),
                "- Config summary in telemetry RunStarted payload.".into(),
                "Next: per docs/unknowns-registry.md roadmap.".into(),
            ),
        }
    }
}
