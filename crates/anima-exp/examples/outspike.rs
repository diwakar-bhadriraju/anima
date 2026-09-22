use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;
fn main(){
  for d in std::env::args().skip(1){
    let r=TelemetryReader::open(&std::path::Path::new(&d).join("telemetry")).unwrap();let idx=r.chunk_index();
    let mut pres: Vec<(u64,String)> = Vec::new(); let mut out: Vec<(u64,u32)> = Vec::new();
    for c in 0..idx.len(){for row in r.chunk_rows(c).unwrap(){
      if let Ok(e)=row.envelope("e"){match &e.payload{
        Payload::StimulusPresented{pattern_id,..}=>pres.push((row.t,pattern_id.clone())),
        Payload::Spike{n}=>if (64..76).contains(&n.0){out.push((row.t,n.0))},
        _=>{}}}}}
    pres.sort_by_key(|p|p.0);
    let mut in_pres:i64=0;let mut n_p=0;let mut in_gap:i64=0;let mut n_g=0;
    for i in 0..pres.len().saturating_sub(1){
      let(t0,_)=pres[i];let(t1,_)=pres[i+1];if t1<=t0||t1-t0>2100{continue}
      in_pres += out.iter().filter(|(t,_)|*t>=t0&&*t<t0+500).count() as i64; n_p+=1;
      in_gap += out.iter().filter(|(t,_)|*t>=t1-500&&*t<t1).count() as i64; n_g+=1;}
    let name=d.rsplit('/').next().unwrap();
    println!("{name}: out spikes in-pres {in_pres}/{n_p} = {:.2}/pres  | in-gap {in_gap}/{n_g} = {:.2}/gap",
      in_pres as f32/n_p.max(1) as f32, in_gap as f32/n_g.max(1) as f32);
  }}
