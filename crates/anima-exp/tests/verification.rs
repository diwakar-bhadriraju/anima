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
    // v2 storage: telemetry lives in chunk files under telemetry/.
    let h1 = sha256_dir(&dir1.join("telemetry"));
    let h2 = sha256_dir(&dir2.join("telemetry"));
    assert_eq!(h1, h2, "same seed + same config must produce identical telemetry");
    let _ = std::fs::remove_dir_all(dir1.parent().unwrap());
    let _ = std::fs::remove_dir_all(dir2.parent().unwrap());
}

/// Deterministic hash over every chunk file's bytes (index.json is stable:
/// chunk names/t-ranges derive from deterministic events).
fn sha256_dir(dir: &std::path::Path) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .expect("telemetry dir")
        .flatten()
        .map(|e| e.path())
        .collect();
    files.sort();
    for f in files {
        h.update(f.file_name().unwrap().to_string_lossy().as_bytes());
        h.update(&std::fs::read(&f).expect("read chunk"));
    }
    format!("{:x}", h.finalize())
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
    // v2: stream chunks and check event kinds.
    let reader = anima_telemetry::TelemetryReader::open(&dir.join("telemetry")).unwrap();
    let mut has_failure = false;
    let mut has_run_ended = false;
    for c in 0..reader.chunk_index().len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                17 => has_failure = true, // Failure
                1 => has_run_ended = true, // RunEnded
                _ => {}
            }
        }
    }
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

// ---- ANIMA E6 integration tests (docs/anima-e6-protocol.md §7 / C0) ----

/// Short v2-enabled config with a rate-skewed curriculum (ch0 in both A
/// and B ⇒ 2:1 duty cycle vs ch1-only) — exactly the regime that makes
/// β ≠ 1. `e6` controls the [e6] section text.
const E6_TOML: &str = r#"
[run]
exp_id = "e6it"
seed = 20260912
viz_port = 8793
ticks_per_sec = 1000.0
stats_decimate = 10

[organism]
n_input_channels = 4
group_size = 2
n_internal = 12
n_output = 4
connectivity = 0.038
w_init = 0.2
amplitude = 52.0
adaptation_tau_ms = 200.0
adaptation_gain = 0.05

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

[v2]
enabled = true
p_in = 0.5
w_in_lo = 0.02
w_in_hi = 0.06
p_rec = 0.2
w_rec_lo = 0.005
w_rec_hi = 0.02
t_e = 0.8
c_slots = 6
w_c_init = 0.01
delta_perm = 0.01
decay_c = 0.99
theta_permanent = 0.05
w_c_permanent = 0.02
theta_die = 0.005
p_cand_in = 0.5
p_cand_rec = 0.5
theta_prune = 0.005
prune_windows = 10
b_e = 40
b_i = 10
p_inh = 0.3
w_inh_lo = 0.01
w_inh_hi = 0.03
a_inh = 0.005
decay_inh = 0.98
w_inh_max = 0.10
window_ticks = 100
__E6__

[[pattern]]
id = "A"
channels = []
channel_ids = [0, 1]
rate_hz = 20.0
duration_ms = 300
jitter_ms = 2.0

[[pattern]]
id = "B"
channels = []
channel_ids = [1, 2]
rate_hz = 20.0
duration_ms = 300
jitter_ms = 2.0

[[stage]]
id = "S0"
present = []
reps = 0
order = "interleaved"
off_ms = 0
silence_ms = 300

[[stage]]
id = "S1"
present = ["A", "B"]
reps = 8
order = "interleaved"
off_ms = 300

[[stage]]
id = "S2"
present = ["B"]
reps = 2
order = "blocked"
off_ms = 300
"#;

const E6_DISABLED: &str = "[e6]\nenable = false\n";
const E6_ENABLED: &str = "[e6]\nenable = true\nalpha = 0.04\nphi_init = 0.02\nphi_min = 0.001\nbeta_min = 0.1\nbeta_max = 10.0\n";

/// C0: [e6] present-but-disabled must be behaviorally byte-identical to
/// the same config without the [e6] section: identical telemetry event
/// stream, EXCLUDING the RunStarted record (which by design carries the
/// raw config hash — different file text, same behavior).
#[test]
fn e6_disabled_byte_identical_short_run() {
    let with = E6_TOML.replace("__E6__", E6_DISABLED);
    let without = E6_TOML.replace("__E6__", "");
    let d1 = run_config("e6off", &with);
    let d2 = run_config("e6no", &without);
    let stream = |dir: &std::path::Path| -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let reader = anima_telemetry::TelemetryReader::open(&dir.join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut h = DefaultHasher::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                let env = row.envelope("e6").unwrap();
                if env.kind == anima_telemetry::events::EventKind::RunStarted {
                    continue; // config identity metadata, not behavior
                }
                serde_json::to_string(&env).unwrap().hash(&mut h);
            }
        }
        format!("{:x}", h.finish())
    };
    assert_eq!(stream(&d1), stream(&d2), "e6 disabled must be behaviorally identical to no-e6");
    let _ = std::fs::remove_dir_all(d1.parent().unwrap());
    let _ = std::fs::remove_dir_all(d2.parent().unwrap());
}

/// Determinism: two identical e6-ENABLED runs ⇒ identical telemetry sha256.
#[test]
fn e6_deterministic_repeated_run() {
    let toml = E6_TOML.replace("__E6__", E6_ENABLED);
    let d1 = run_config("e6d1", &toml);
    let d2 = run_config("e6d2", &toml);
    let h1 = sha256_dir(&d1.join("telemetry"));
    let h2 = sha256_dir(&d2.join("telemetry"));
    assert_eq!(h1, h2, "same seed + e6 => byte-identical telemetry");
    let _ = std::fs::remove_dir_all(d1.parent().unwrap());
    let _ = std::fs::remove_dir_all(d2.parent().unwrap());
}

/// Engagement: under a rate-skewed curriculum (ch1 in both patterns),
/// e6-enabled must NOT be byte-identical to e6-disabled — the mechanism
/// must actually alter plasticity (β ≠ 1).
#[test]
fn e6_enabled_changes_telemetry_under_rate_skew() {
    let on = E6_TOML.replace("__E6__", E6_ENABLED);
    let off = E6_TOML.replace("__E6__", E6_DISABLED);
    let d1 = run_config("e6on", &on);
    let d2 = run_config("e6of2", &off);
    let h1 = sha256_dir(&d1.join("telemetry"));
    let h2 = sha256_dir(&d2.join("telemetry"));
    assert_ne!(h1, h2, "rate-skewed curriculum must engage E6 (β ≠ 1)");
    let _ = std::fs::remove_dir_all(d1.parent().unwrap());
    let _ = std::fs::remove_dir_all(d2.parent().unwrap());
}

// ---- E7 (docs/anima-e7-protocol.md §10.4): deterministic repeat on the
// ---- absence-layout curriculum (24 ch; X = {0-15}, Y = {0-7} subset).

const E7_SHORT_TOML: &str = r#"
[run]
exp_id = "e7it"
seed = 20260912
viz_port = 8795
ticks_per_sec = 1000.0
stats_decimate = 10

[organism]
n_input_channels = 24
group_size = 8
n_internal = 12
n_output = 4
connectivity = 0.038
w_init = 0.2
amplitude = 52.0
adaptation_tau_ms = 200.0
adaptation_gain = 0.05

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

[v2]
enabled = true
p_in = 0.5
w_in_lo = 0.02
w_in_hi = 0.06
p_rec = 0.2
w_rec_lo = 0.005
w_rec_hi = 0.02
t_e = 0.8
c_slots = 6
w_c_init = 0.01
delta_perm = 0.01
decay_c = 0.99
theta_permanent = 0.05
w_c_permanent = 0.02
theta_die = 0.005
p_cand_in = 0.5
p_cand_rec = 0.5
theta_prune = 0.005
prune_windows = 10
b_e = 40
b_i = 10
p_inh = 0.3
w_inh_lo = 0.01
w_inh_hi = 0.03
a_inh = 0.005
decay_inh = 0.98
w_inh_max = 0.10
window_ticks = 100

[e6]
enable = true
alpha = 0.04
phi_init = 0.02
phi_min = 0.001
beta_min = 0.1
beta_max = 10.0

[[pattern]]
id = "X"
channels = []
channel_ids = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
rate_hz = 20.0
duration_ms = 300
jitter_ms = 2.0

[[pattern]]
id = "Y"
channels = []
channel_ids = [0, 1, 2, 3, 4, 5, 6, 7]
rate_hz = 20.0
duration_ms = 300
jitter_ms = 2.0

[[stage]]
id = "S0"
present = []
reps = 0
order = "interleaved"
off_ms = 0
silence_ms = 300

[[stage]]
id = "S1"
present = ["X", "Y"]
reps = 4
order = "interleaved"
off_ms = 300

[[stage]]
id = "S3"
present = ["X", "Y"]
reps = 2
order = "interleaved"
off_ms = 300
"#;

// ---- E8 (docs/anima-e8-protocol.md §9.5): deterministic repeat on the
// ---- 3-way overlap layout (A = {0-7}, B = {4-11}, C = {8-15}).

const E8_SHORT_TOML: &str = r#"
[run]
exp_id = "e8it"
seed = 20260912
viz_port = 8797
ticks_per_sec = 1000.0
stats_decimate = 10

[organism]
n_input_channels = 24
group_size = 8
n_internal = 12
n_output = 4
connectivity = 0.038
w_init = 0.2
amplitude = 52.0
adaptation_tau_ms = 200.0
adaptation_gain = 0.05

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

[v2]
enabled = true
p_in = 0.5
w_in_lo = 0.02
w_in_hi = 0.06
p_rec = 0.2
w_rec_lo = 0.005
w_rec_hi = 0.02
t_e = 0.8
c_slots = 6
w_c_init = 0.01
delta_perm = 0.01
decay_c = 0.99
theta_permanent = 0.05
w_c_permanent = 0.02
theta_die = 0.005
p_cand_in = 0.5
p_cand_rec = 0.5
theta_prune = 0.005
prune_windows = 10
b_e = 40
b_i = 10
p_inh = 0.3
w_inh_lo = 0.01
w_inh_hi = 0.03
a_inh = 0.005
decay_inh = 0.98
w_inh_max = 0.10
window_ticks = 100

[e6]
enable = true
alpha = 0.04
phi_init = 0.02
phi_min = 0.001
beta_min = 0.1
beta_max = 10.0

[[pattern]]
id = "A"
channels = []
channel_ids = [0, 1, 2, 3, 4, 5, 6, 7]
rate_hz = 20.0
duration_ms = 300
jitter_ms = 2.0

[[pattern]]
id = "B"
channels = []
channel_ids = [4, 5, 6, 7, 8, 9, 10, 11]
rate_hz = 20.0
duration_ms = 300
jitter_ms = 2.0

[[pattern]]
id = "C"
channels = []
channel_ids = [8, 9, 10, 11, 12, 13, 14, 15]
rate_hz = 20.0
duration_ms = 300
jitter_ms = 2.0

[[stage]]
id = "S0"
present = []
reps = 0
order = "interleaved"
off_ms = 0
silence_ms = 300

[[stage]]
id = "S1"
present = ["A", "B", "C"]
reps = 4
order = "interleaved"
off_ms = 300

[[stage]]
id = "S3"
present = ["A", "B", "C"]
reps = 2
order = "interleaved"
off_ms = 300
"#;

// ---- E9 (docs/anima-e9-protocol.md §10.5): deterministic repeat on the
// ---- phase/SEQ layout (B phases 40 Hz x 250 ms).

const E9_SHORT_TOML: &str = r#"
[run]
exp_id = "e9it"
seed = 20260912
viz_port = 8799
ticks_per_sec = 1000.0
stats_decimate = 10

[organism]
n_input_channels = 24
group_size = 8
n_internal = 12
n_output = 4
connectivity = 0.038
w_init = 0.2
amplitude = 52.0
adaptation_tau_ms = 200.0
adaptation_gain = 0.05

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

[v2]
enabled = true
p_in = 0.5
w_in_lo = 0.02
w_in_hi = 0.06
p_rec = 0.2
w_rec_lo = 0.005
w_rec_hi = 0.02
t_e = 0.8
c_slots = 6
w_c_init = 0.01
delta_perm = 0.01
decay_c = 0.99
theta_permanent = 0.05
w_c_permanent = 0.02
theta_die = 0.005
p_cand_in = 0.5
p_cand_rec = 0.5
theta_prune = 0.005
prune_windows = 10
b_e = 40
b_i = 10
p_inh = 0.3
w_inh_lo = 0.01
w_inh_hi = 0.03
a_inh = 0.005
decay_inh = 0.98
w_inh_max = 0.10
window_ticks = 100

[e6]
enable = true
alpha = 0.04
phi_init = 0.02
phi_min = 0.001
beta_min = 0.1
beta_max = 10.0

[[pattern]]
id = "A"
channels = []
channel_ids = [0, 1, 2, 3, 4, 5, 6, 7]
rate_hz = 20.0
duration_ms = 300
jitter_ms = 2.0

[[pattern]]
id = "B"
channels = []
rate_hz = 40.0
duration_ms = 300
jitter_ms = 2.0

[[pattern.phases]]
from_ms = 0
to_ms = 150
channel_ids = [4, 5, 6, 7]
rate_hz = 40.0

[[pattern.phases]]
from_ms = 150
to_ms = 300
channel_ids = [8, 9, 10, 11]
rate_hz = 40.0

[[pattern]]
id = "C"
channels = []
channel_ids = [8, 9, 10, 11, 12, 13, 14, 15]
rate_hz = 20.0
duration_ms = 300
jitter_ms = 2.0

[[stage]]
id = "S0"
present = []
reps = 0
order = "interleaved"
off_ms = 0
silence_ms = 300

[[stage]]
id = "S1"
present = ["A", "B", "C"]
reps = 4
order = "interleaved"
off_ms = 300

[[stage]]
id = "S3"
present = ["A", "B", "C"]
reps = 2
order = "interleaved"
off_ms = 300
"#;

// ---- E11 (docs/anima-e11-protocol.md §7.6): deterministic repeat on
// ---- the counterbalanced phase-variant layout.

const E11_SHORT_TOML: &str = r#"
[run]
exp_id = "e11it"
seed = 20260912
viz_port = 8801
ticks_per_sec = 1000.0
stats_decimate = 10

[organism]
n_input_channels = 24
group_size = 8
n_internal = 12
n_output = 4
connectivity = 0.038
w_init = 0.2
amplitude = 52.0
adaptation_tau_ms = 200.0
adaptation_gain = 0.05

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

[v2]
enabled = true
p_in = 0.5
w_in_lo = 0.02
w_in_hi = 0.06
p_rec = 0.2
w_rec_lo = 0.005
w_rec_hi = 0.02
t_e = 0.8
c_slots = 6
w_c_init = 0.01
delta_perm = 0.01
decay_c = 0.99
theta_permanent = 0.05
w_c_permanent = 0.02
theta_die = 0.005
p_cand_in = 0.5
p_cand_rec = 0.5
theta_prune = 0.005
prune_windows = 10
b_e = 40
b_i = 10
p_inh = 0.3
w_inh_lo = 0.01
w_inh_hi = 0.03
a_inh = 0.005
decay_inh = 0.98
w_inh_max = 0.10
window_ticks = 100

[e6]
enable = true
alpha = 0.04
phi_init = 0.02
phi_min = 0.001
beta_min = 0.1
beta_max = 10.0

[[pattern]]
id = "A"
channels = []
channel_ids = [0, 1, 2, 3, 4, 5, 6, 7]
rate_hz = 20.0
duration_ms = 300
jitter_ms = 2.0

[[pattern]]
id = "B"
channels = []
rate_hz = 40.0
duration_ms = 300
jitter_ms = 2.0

[[pattern.phase_variants]]
[[pattern.phase_variants.phases]]
from_ms = 0
to_ms = 150
channel_ids = [4, 5, 6, 7]
rate_hz = 40.0

[[pattern.phase_variants.phases]]
from_ms = 150
to_ms = 300
channel_ids = [8, 9, 10, 11]
rate_hz = 40.0

[[pattern.phase_variants]]
[[pattern.phase_variants.phases]]
from_ms = 0
to_ms = 150
channel_ids = [8, 9, 10, 11]
rate_hz = 40.0

[[pattern.phase_variants.phases]]
from_ms = 150
to_ms = 300
channel_ids = [4, 5, 6, 7]
rate_hz = 40.0

[[pattern]]
id = "C"
channels = []
channel_ids = [8, 9, 10, 11, 12, 13, 14, 15]
rate_hz = 20.0
duration_ms = 300
jitter_ms = 2.0

[[stage]]
id = "S0"
present = []
reps = 0
order = "interleaved"
off_ms = 0
silence_ms = 300

[[stage]]
id = "S1"
present = ["A", "B", "C"]
reps = 4
order = "interleaved"
off_ms = 300

[[stage]]
id = "S3"
present = ["A", "B", "C"]
reps = 2
order = "interleaved"
off_ms = 300
"#;

// ---- E12 (docs/anima-e12-protocol.md §9.6): deterministic repeat on
// ---- the blocked phase-variant layout (variant_block = 60).

const E12_SHORT_TOML: &str = r#"
[run]
exp_id = "e12it"
seed = 20260912
viz_port = 8803
ticks_per_sec = 1000.0
stats_decimate = 10

[organism]
n_input_channels = 24
group_size = 8
n_internal = 12
n_output = 4
connectivity = 0.038
w_init = 0.2
amplitude = 52.0
adaptation_tau_ms = 200.0
adaptation_gain = 0.05

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

[v2]
enabled = true
p_in = 0.5
w_in_lo = 0.02
w_in_hi = 0.06
p_rec = 0.2
w_rec_lo = 0.005
w_rec_hi = 0.02
t_e = 0.8
c_slots = 6
w_c_init = 0.01
delta_perm = 0.01
decay_c = 0.99
theta_permanent = 0.05
w_c_permanent = 0.02
theta_die = 0.005
p_cand_in = 0.5
p_cand_rec = 0.5
theta_prune = 0.005
prune_windows = 10
b_e = 40
b_i = 10
p_inh = 0.3
w_inh_lo = 0.01
w_inh_hi = 0.03
a_inh = 0.005
decay_inh = 0.98
w_inh_max = 0.10
window_ticks = 100

[e6]
enable = true
alpha = 0.04
phi_init = 0.02
phi_min = 0.001
beta_min = 0.1
beta_max = 10.0

[[pattern]]
id = "A"
channels = []
channel_ids = [0, 1, 2, 3, 4, 5, 6, 7]
rate_hz = 20.0
duration_ms = 300
jitter_ms = 2.0

[[pattern]]
id = "B"
channels = []
rate_hz = 40.0
duration_ms = 300
jitter_ms = 2.0
variant_block = 2

[[pattern.phase_variants]]
[[pattern.phase_variants.phases]]
from_ms = 0
to_ms = 150
channel_ids = [4, 5, 6, 7]
rate_hz = 40.0

[[pattern.phase_variants.phases]]
from_ms = 150
to_ms = 300
channel_ids = [8, 9, 10, 11]
rate_hz = 40.0

[[pattern.phase_variants]]
[[pattern.phase_variants.phases]]
from_ms = 0
to_ms = 150
channel_ids = [8, 9, 10, 11]
rate_hz = 40.0

[[pattern.phase_variants.phases]]
from_ms = 150
to_ms = 300
channel_ids = [4, 5, 6, 7]
rate_hz = 40.0

[[pattern]]
id = "C"
channels = []
channel_ids = [8, 9, 10, 11, 12, 13, 14, 15]
rate_hz = 20.0
duration_ms = 300
jitter_ms = 2.0

[[stage]]
id = "S0"
present = []
reps = 0
order = "interleaved"
off_ms = 0
silence_ms = 300

[[stage]]
id = "S1"
present = ["A", "B", "C"]
reps = 4
order = "interleaved"
off_ms = 300

[[stage]]
id = "S3"
present = ["A", "B", "C"]
reps = 2
order = "interleaved"
off_ms = 300
"#;

#[test]
fn e12_blocked_layout_deterministic_repeated_run() {
    let d1 = run_config("e12d1", E12_SHORT_TOML);
    let d2 = run_config("e12d2", E12_SHORT_TOML);
    let h1 = sha256_dir(&d1.join("telemetry"));
    let h2 = sha256_dir(&d2.join("telemetry"));
    assert_eq!(h1, h2, "same seed + e12 blocked layout => byte-identical telemetry");
    let _ = std::fs::remove_dir_all(d1.parent().unwrap());
    let _ = std::fs::remove_dir_all(d2.parent().unwrap());
}

#[test]
fn e11_variant_layout_deterministic_repeated_run() {
    let d1 = run_config("e11d1", E11_SHORT_TOML);
    let d2 = run_config("e11d2", E11_SHORT_TOML);
    let h1 = sha256_dir(&d1.join("telemetry"));
    let h2 = sha256_dir(&d2.join("telemetry"));
    assert_eq!(h1, h2, "same seed + e11 variant layout => byte-identical telemetry");
    let _ = std::fs::remove_dir_all(d1.parent().unwrap());
    let _ = std::fs::remove_dir_all(d2.parent().unwrap());
}

#[test]
fn e9_phase_layout_deterministic_repeated_run() {
    let d1 = run_config("e9d1", E9_SHORT_TOML);
    let d2 = run_config("e9d2", E9_SHORT_TOML);
    let h1 = sha256_dir(&d1.join("telemetry"));
    let h2 = sha256_dir(&d2.join("telemetry"));
    assert_eq!(h1, h2, "same seed + e9 phase layout => byte-identical telemetry");
    let _ = std::fs::remove_dir_all(d1.parent().unwrap());
    let _ = std::fs::remove_dir_all(d2.parent().unwrap());
}

#[test]
fn e8_overlap_layout_deterministic_repeated_run() {
    let d1 = run_config("e8d1", E8_SHORT_TOML);
    let d2 = run_config("e8d2", E8_SHORT_TOML);
    let h1 = sha256_dir(&d1.join("telemetry"));
    let h2 = sha256_dir(&d2.join("telemetry"));
    assert_eq!(h1, h2, "same seed + e8 overlap layout => byte-identical telemetry");
    let _ = std::fs::remove_dir_all(d1.parent().unwrap());
    let _ = std::fs::remove_dir_all(d2.parent().unwrap());
}

#[test]
fn e7_abs_layout_deterministic_repeated_run() {
    let d1 = run_config("e7d1", E7_SHORT_TOML);
    let d2 = run_config("e7d2", E7_SHORT_TOML);
    let h1 = sha256_dir(&d1.join("telemetry"));
    let h2 = sha256_dir(&d2.join("telemetry"));
    assert_eq!(h1, h2, "same seed + e7 absence layout => byte-identical telemetry");
    let _ = std::fs::remove_dir_all(d1.parent().unwrap());
    let _ = std::fs::remove_dir_all(d2.parent().unwrap());
}
