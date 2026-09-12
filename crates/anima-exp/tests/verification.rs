//! Integration verification (plan §Verification):
//! 1. determinism — two identical headless runs ⇒ identical telemetry sha256
//! 2. failure path — crafted cap + birth trigger ⇒ Failure{resource-exhaustion},
//!    RunEnded, files preserved
//! 3. pipeline — short run ⇒ metrics.json + report.md with finite values

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::process::Command;

fn sha256_file(path: &std::path::Path) -> String {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path).expect("read telemetry");
    let mut h = Sha256::new();
    h.update(&bytes);
    format!("{:x}", h.finalize())
}

const SHORT_CONFIG: &str = r#"
[run]
exp_id = "det"
seed = 424242
viz_port = 8791
ticks_per_sec = 1000.0
stats_decimate = 10

[organism]
n_input_channels = 12
group_size = 4
n_internal = 16
n_output = 4
connectivity = 0.038
w_init = 0.2
amplitude = 52.0

[plasticity]
rule = "stdp-pairwise"
tau_plus_ms = 20.0
tau_minus_ms = 20.0
a_plus = 0.005
a_minus = 0.0053
w_min = 0.0
w_max = 1.0
decay = 1e-6
silence_w = 0.02
silence_ticks = 60000
min_age_ticks = 30000
gate = "always"

[structural]
birth_trigger = "none"
dormancy_rate_hz = 0.1
dormancy_ms = 30000
recovery_rate_hz = 1.0
retirement_ms = 300000
wiring_synapses = 5

[resources]
max_neurons = 200
max_synapses = 2000
births_per_window = 4
runaway_rate_hz = 50.0
runaway_sustained_ms = 5000
fragmentation_min_component = 0.6

[[pattern]]
id = "A"
channels = ["A"]
rate_hz = 20.0
duration_ms = 500
jitter_ms = 2.0

[[pattern]]
id = "B"
channels = ["B"]
rate_hz = 20.0
duration_ms = 500
jitter_ms = 2.0

[[pattern]]
id = "D"
channels = ["A", "B"]
rate_hz = 20.0
duration_ms = 500
jitter_ms = 2.0

[[stage]]
id = "S0"
present = []
reps = 0
order = "interleaved"
off_ms = 0
silence_ms = 500

[[stage]]
id = "S1"
present = ["A", "B"]
reps = 3
order = "interleaved"
off_ms = 300

[[stage]]
id = "S2"
present = ["D"]
reps = 2
order = "blocked"
off_ms = 300

[[stage]]
id = "S3"
present = ["A", "B"]
reps = 2
order = "interleaved"
off_ms = 300
"#;

fn run_config(name: &str, toml: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("anima-it-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("runs")).unwrap();
    let cfg_path = dir.join("cfg.toml");
    std::fs::write(&cfg_path, toml).unwrap();
    let bin = env!("CARGO_BIN_EXE_anima-run");
    let out = Command::new(bin)
        .args(["run", "--config", cfg_path.to_str().unwrap()])
        .current_dir(&dir)
        .output()
        .expect("spawn anima-run");
    assert!(out.status.success(), "run failed: {}", String::from_utf8_lossy(&out.stderr));
    let mut run_dir = None;
    for entry in std::fs::read_dir(dir.join("runs")).unwrap().flatten() {
        run_dir = Some(entry.path());
    }
    run_dir.expect("one run dir")
}

#[test]
fn determinism_two_runs_identical_telemetry() {
    let dir1 = run_config("d1", SHORT_CONFIG);
    let dir2 = run_config("d2", SHORT_CONFIG);
    let h1 = sha256_file(&dir1.join("telemetry.jsonl"));
    let h2 = sha256_file(&dir2.join("telemetry.jsonl"));
    assert_eq!(h1, h2, "same seed + same config must produce identical telemetry");
    let _ = std::fs::remove_dir_all(dir1.parent().unwrap());
    let _ = std::fs::remove_dir_all(dir2.parent().unwrap());
}

#[test]
fn failure_path_crafted_config() {
    // Initial 32 neurons, cap 33: the first birth passes admission, the
    // second must trip Failure{resource-exhaustion}. Trigger thresholds are
    // lowered (0.5 Hz / 500 ms) so the short curriculum's bursts fire it.
    let mut cfg = SHORT_CONFIG.to_string();
    cfg = cfg.replace("exp_id = \"det\"", "exp_id = \"fail\"");
    cfg = cfg.replace("max_neurons = 200", "max_neurons = 33");
    cfg = cfg.replace(
        "birth_trigger = \"none\"",
        "birth_trigger = \"homeostatic-saturation\"\ntrigger_rate_hz = 0.5\ntrigger_sustained_ms = 500",
    );
    let dir = run_config("f1", &cfg);
    let telemetry = std::fs::read_to_string(dir.join("telemetry.jsonl")).unwrap();
    let has_failure = telemetry.contains("\"failure\"");
    let has_run_ended = telemetry.contains("run-ended");
    assert!(has_failure, "crafted config must produce a Failure event");
    assert!(has_run_ended, "failure must end the run");
    // files preserved
    assert!(dir.join("snapshots.bin.zst").metadata().unwrap().len() > 0);
    assert!(dir.join("report.md").exists(), "report still generated from failed run");
    let report = std::fs::read_to_string(dir.join("report.md")).unwrap();
    assert!(report.contains("Failure modes") && !report.contains("- None."), "failure section populated");
    let _ = std::fs::remove_dir_all(dir.parent().unwrap());
}

#[test]
fn pipeline_metrics_and_report() {
    let dir = run_config("p1", SHORT_CONFIG);
    let metrics: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("metrics.json")).unwrap(),
    )
    .unwrap();
    // structure populated
    assert!(metrics["structure"]["synapse_created"].as_u64().unwrap() > 0);
    // retention keys exist (values may be None if no response — pipeline, not hypothesis)
    assert!(metrics["retention"].is_object());
    let report = std::fs::read_to_string(dir.join("report.md")).unwrap();
    for section in [
        "## Hypothesis", "## Setup", "## Results", "## Structural timeline",
        "## Resource usage", "## Failure modes", "## Next experiment",
        "auto-generated — review",
    ] {
        assert!(report.contains(section), "report missing {section}");
    }
    let _ = std::fs::remove_dir_all(dir.parent().unwrap());
}
