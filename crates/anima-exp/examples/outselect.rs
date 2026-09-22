//! r2 Slice-0 gate: is the trained E-nogain organism's OUTPUT stimulus-
//! selective (distinguishable A vs C response)? If not, no action channel
//! exists to close a loop on. Uses committed base telemetry.
use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;
fn cosv(a:&[f32],b:&[f32])->f32{let(mut n,mut na,mut nb)=(0.0f32,0.0f32,0.0f32);for(x,y)in a.iter().zip(b){n+=x*y;na+=x*x;nb+=y*y}if na<=0.0||nb<=0.0{0.0}else{n/(na.sqrt()*nb.sqrt())}}
fn main(){
  for dir in std::env::args().skip(1){
    let r=TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();let idx=r.chunk_index();
    let mut pres=vec![];let mut spk=vec![];
    for c in 0..idx.len(){for row in r.chunk_rows(c).unwrap(){if let Ok(e)=row.envelope("e"){match &e.payload{
      Payload::StimulusPresented{pattern_id,..}=>pres.push((row.t,pattern_id.clone())),
      Payload::Spike{n}=>if (64..76).contains(&n.0){spk.push((row.t,n.0))}, _=>{}}}}}
    pres.sort_by_key(|p|p.0);
    let mut refs:std::collections::BTreeMap<String,Vec<f32>>=Default::default();
    let mut cnt:std::collections::BTreeMap<String,u32>=Default::default();
    for (t,p) in &pres{let mut v=vec![0.0f32;12];for(st,s)in &spk{if *st>=*t&&*st<t+500&&(64..76).contains(s){v[(s-64)as usize]+=1.0}}let e=refs.entry(p.clone()).or_insert_with(||vec![0.0f32;12]);for i in 0..12{e[i]+=v[i]};*cnt.entry(p.clone()).or_insert(0)+=1;}
    for(p,v)in refs.iter_mut(){let c=cnt[p]as f32;for x in v.iter_mut(){*x/=c}}
    let name=dir.rsplit('/').next().unwrap().to_string();
    print!("{name} out-ref:");
    for(p,v)in &refs{print!(" {p}=[{}]", v.iter().map(|x|format!("{x:.0}")).collect::<Vec<_>>().join(","));}
    let kv:Vec<(&String,&Vec<f32>)>=refs.iter().collect();
    if kv.len()>=2{let c=cosv(kv[0].1,kv[1].1);println!("  cos(OA_ref,OC_ref)={c:.3}  (high=NOT selective -> no action space)");}
    else{println!();}
  }}
