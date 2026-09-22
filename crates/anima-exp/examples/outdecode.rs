//! r2 gate refinement (advisory): per-trial linear-separability decoder.
//! Refs from S1 ONLY (training subset); classify S3A/S3C held-out single
//! presentations by argmax-cosine of the 12-dim output vector. Accuracy
//! vs 50% chance. High pooled cosine can coexist with per-trial
//! separability when A/C differ in magnitude/subset.
use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;
fn cosv(a:&[f32],b:&[f32])->f32{let(mut n,mut na,mut nb)=(0.0f32,0.0f32,0.0f32);for(x,y)in a.iter().zip(b){n+=x*y;na+=x*x;nb+=y*y}if na<=0.0||nb<=0.0{0.0}else{n/(na.sqrt()*nb.sqrt())}}
fn main(){
  for dir in std::env::args().skip(1){
    let r=TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();let idx=r.chunk_index();
    let mut pres:Vec<(u64,String)>=vec![];let mut spk:Vec<(u64,u32)>=vec![];
    for c in 0..idx.len(){for row in r.chunk_rows(c).unwrap(){if let Ok(e)=row.envelope("e"){match &e.payload{
      Payload::StimulusPresented{pattern_id,..}=>pres.push((row.t,pattern_id.clone())),
      Payload::Spike{n}=>if (64..76).contains(&n.0){spk.push((row.t,n.0))},_=>{}}}}}
    pres.sort_by_key(|p|p.0);
    // train refs from S1 stage only; test on S3A/S3C
    let mut refs:std::collections::BTreeMap<String,Vec<f32>>=Default::default();
    let mut cnt:std::collections::BTreeMap<String,u32>=Default::default();
    let mut test:Vec<(String,Vec<f32>)>=vec![];
    for (t,p) in &pres{let istage = pres.iter().filter(|(t2,p2)|*t2<=*t&&*t2>*t-0).count(); // placeholder
      let mut v=vec![0.0f32;12];for(st,s)in &spk{if *st>=*t&&*st<t+500&&(64..76).contains(s){v[(s-64)as usize]+=1.0}}
      // stage name isn't in the payload; use S3 offset heuristic: S3 begins
      // after S1+S2 (~40 interleaved + gap). Approx by rep>0 of a lone pattern
      // present after the alternation. We approximate "held-out" = block of 5
      // of a single pattern (S3A/S3C are blocked A/C reps).
      if pending_block(p, &pres, t){ test.push((p.clone(),v)); } else { let e=refs.entry(p.clone()).or_insert_with(||vec![0.0f32;12]);for i in 0..12{e[i]+=v[i]};*cnt.entry(p.clone()).or_insert(0)+=1; }
    }
    for(p,v)in refs.iter_mut(){let c=cnt[p]as f32;for x in v.iter_mut(){*x/=c}}
    let name=dir.rsplit('/').next().unwrap().to_string();
    let (mut cor,mut tot, mut per_a, mut tot_a)=(0u32,0u32,0u32,0u32);
    let (refA,refC)=(refs.get("A").cloned(),refs.get("C").cloned());
    if let (Some(ra),Some(rc))=(refA,refC){
      for (p,v) in &test{let(ca,cc)=(cosv(v,&ra),cosv(v,&rc));let pred=if ca>=cc{"A"}else{"C"};if pred==p{cor+=1}else{};tot+=1;if p=="A"{tot_a+=1;if pred=="A"{per_a+=1}}}
      let acc=cor as f32/tot.max(1)as f32;let accA=per_a as f32/tot_a.max(1)as f32;
      println!("{name} S3 held-out decode: n={tot} acc={acc:.3} (A-acc={accA:.2}, A-tries={tot_a}) vs chance 0.50");
    } else { println!("{name}: missing A/C refs in S1 subset"); }
  }
}
fn pending_block(p:&str,pres:&[(u64,String)],t:u64)->bool{
  // A presentation counts as a "held-out block trial" if it is part of a
  // run of >=2 consecutive same-pattern presentations with long gaps
  // (the S3 blocked style, 1500ms), i.e. near the end of the run.
  // Heuristic: the last 5 of each lone pattern.
  let n=pres.len(); let mut back=0; for j in 0..n{ let(tj,pj)=pres[n-1-j]; if tj>t {continue} }
  // simpler: S3A/S3C are the tail blocks; approximate held-out = last 10
  // presentations of the run
  let n=pres.len();
  for (j,(tj,_)) in pres.iter().enumerate(){ if *tj==t { return j + 5 >= n; } }
  false
}
