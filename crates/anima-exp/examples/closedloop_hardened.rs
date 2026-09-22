//! r2 Slice-0/1 hardened analysis (settles D-18's asserted claims by
//! measurement on existing telemetry; no new runs):
//!  (a) per-trial argmax-cos decoder accuracy (Slice-0 gate completion):
//!      refs from S1, classify S3 re-exposure presentations vs 50% chance;
//!  (b) per-presentation vote fractions + margin distribution for the two
//!      closed-loop transitions (from A, from C) - fragility of 2-cycles;
//!  (c) tag-split robustness sweep (4/8,5/7,6/6,7/5,8/4) - does the
//!      attractor persist (organism structure) or flip (boundary artifact)?
use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;
fn cosv(a:&[f32],b:&[f32])->f32{let(mut n,mut na,mut nb)=(0.0f32,0.0f32,0.0f32);for(x,y)in a.iter().zip(b){n+=x*y;na+=x*x;nb+=y*y}if na<=0.0||nb<=0.0{0.0}else{n/(na.sqrt()*nb.sqrt())}}
fn attractor_for(split:usize, ra:&[f32], rc:&[f32])->Vec<&'static str>{
  // map: next = C if tag[pre_lo..split] <= tag[split..] else A
  let next_of=|res:&[f32]|->&'static str{let ta:f32=res[..split].iter().sum();let tc:f32=res[split..].iter().sum();if ta<=tc{"C"}else{"A"}};
  let mut seen=std::collections::BTreeMap::new();let mut cur:&'static str="A";
  for _ in 0..8u32{if seen.insert(cur,()).is_some(){break}cur=next_of(if cur=="A"{ra}else{rc});}
  seen.into_keys().collect()
}
fn main(){
  for dir in std::env::args().skip(1){
    let r=TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();let idx=r.chunk_index();
    let mut pres:Vec<(u64,String,String)>=vec![];let mut spk:Vec<(u64,u32)>=vec![];
    for c in 0..idx.len(){for row in r.chunk_rows(c).unwrap(){if let Ok(e)=row.envelope("e"){match &e.payload{
      Payload::StimulusPresented{pattern_id,stage}=>pres.push((row.t,pattern_id.clone(),stage.clone())),
      Payload::Spike{n}=>if (64..76).contains(&n.0){spk.push((row.t,n.0))},_=>{}}}}}
    pres.sort_by_key(|p|p.0);
    let mut vecs:Vec<(String,String,Vec<f32>)>=vec![];
    for (t,p,st) in &pres{let mut v=vec![0.0f32;12];for(st2,s)in &spk{if *st2>=*t&&*st2<t+500&&(64..76).contains(s){v[(s-64)as usize]+=1.0}}vecs.push((p.clone(),st.clone(),v));}
    // refs from S1
    let mut refs:std::collections::BTreeMap<String,Vec<f32>>=Default::default();let mut cnt:std::collections::BTreeMap<String,u32>=Default::default();
    for (p,st,v) in &vecs{if st=="S1"{let e=refs.entry(p.clone()).or_insert_with(||vec![0.0f32;12]);for i in 0..12{e[i]+=v[i]};*cnt.entry(p.clone()).or_insert(0)+=1;}}
    for(p,v)in refs.iter_mut(){let c=cnt[p]as f32;for x in v.iter_mut(){*x/=c}}
    let name=dir.rsplit('/').next().unwrap().to_string();
    // (a) per-trial decode on S3 (stage S3A/S3C)
    let (mut cor,mut tot)=(0u32,0u32);
    if let(Some(ra),Some(rc))=(refs.get("A"),refs.get("C")){
      for (p,st,v) in &vecs{if st=="S3A"||st=="S3C"{let(ca,cc)=(cosv(v,ra),cosv(v,rc));let pred=if ca>=cc{"A"}else{"C"};if pred==p{cor+=1}tot+=1;}}
      println!("{name} (a) S3 held-out decode: acc={:.3} (n={tot}) vs 0.50", cor as f32/tot.max(1)as f32);
    }
    // (b) per-presentation vote fractions for both transitions (tag 6/6)
    for from in ["A","C"]{
      let mut vote_c=0u32;let mut n=0u32;let mut min_marg:f32=9e9;
      let (ra,rc)=(refs.get("A").unwrap_or(&vec![0.0;12]).clone(),refs.get("C").unwrap_or(&vec![0.0;12]).clone());
      for (p,st,v) in &vecs{if st=="S1"&&p==from{let ta:f32=v[..6].iter().sum();let tc:f32=v[6..].iter().sum();let marg=(ta-tc).abs();min_marg=min_marg.min(marg);if ta<=tc{vote_c+=1}n+=1;}}
      println!("{name} (b) from {from}: per-presentation votes C={vote_c}/{n} (f={:.2}), min-margin={min_marg:.1}", vote_c as f32/n.max(1)as f32);
    }
    // (c) tag-split sweep
    let ra=refs.get("A").unwrap().clone();let rc=refs.get("C").unwrap().clone();
    let mut out=Vec::new();for sp in [4usize,5,6,7,8]{let a=attractor_for(sp,&ra,&rc);out.push(format!("{}/{}:{{{}}}",sp,12-sp,a.join(",")));}
    println!("{name} (c) tag-split sweep attrs: {}", out.join("  "));
  }
}