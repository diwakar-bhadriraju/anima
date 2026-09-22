//! kindhist: read-only telemetry payload-kind histogram (audit use).
use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;
fn main(){
    for dir in std::env::args().skip(1){
        let r = TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = r.chunk_index();
        let mut c: std::collections::BTreeMap<String, usize> = Default::default();
        for ci in 0..idx.len() {
            for row in r.chunk_rows(ci).unwrap(){
                if let Ok(e) = row.envelope("e"){
                    let k = match &e.payload {
                        Payload::Spike{..} => "Spike",
                        Payload::StimulusPresented{..} => "Pres",
                        Payload::SynapseCreated{..} => "SC",
                        Payload::SynapsePruned{..} => "SP",
                        Payload::SynapseStrengthened{..} => "SW+",
                        Payload::SynapseWeakened{..} => "SW-",
                        Payload::NeuronCreated{..} => "NC",
                        _ => "Other",
                    }.to_string();
                    *c.entry(k).or_default() += 1;
                }
            }
        }
        let n: usize = c.values().sum();
        println!("{} total={} {:?}", dir.rsplit('/').next().unwrap(), n, c);
    }
}
