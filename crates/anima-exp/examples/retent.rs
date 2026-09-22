//! Progressive-acquisition retention probe (D-21): per-pattern selectivity
//! at S3 re-exposure vs its acquisition ref, pooled 52-dim internal space,
//! centered cosine (rate-level insensitive).
//!   ref(P) = S1 for A/C; = S1D for D (D never appears in S1).
//!   rho(P) = cos(S3P, ref_P) - cos(S3P, ref_other), other in {A,C}.
use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;
fn mean(v:&[f32])->f32{v.iter().sum::<f32>()/v.len() as f32}
fn cosv(a:&[f32],b:&[f32])->f32{let(am,bm)=(mean(a),mean(b));let(mut n,mut na,mut nb)=(0.0f32,0.0f32,0.0f32);for(x,y)in a.iter().zip(b){let(x,y)=(x-am,y-bm);n+=x*y;na+=x*x;nb+=y*y}if na<=0.0||nb<=0.0{0.0}else{n/(na.sqrt()*nb.sqrt())}}

fn main(){
  for dir in std::env::args().skip(1){
    let r=TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();let idx=r.chunk_index();
    let mut pres:Vec<(u64,String,String)>=vec![];let mut spk:Vec<(u64,u32)>=vec![];
    for c in 0..idx.len(){for row in r.chunk_rows(c).unwrap(){if let Ok(e)=row.envelope("e"){match &e.payload{
      Payload::StimulusPresented{pattern_id,stage}=>pres.push((row.t,pattern_id.clone(),stage.clone())),
      Payload::Spike{n}=>if (24..76).contains(&n.0){spk.push((row.t,n.0))},_=>{}}}}}
    pres.sort_by_key(|p|p.0);
    let mut vecs:Vec<(String,String,Vec<f32>)>=vec![];
    for (t,p,st) in &pres{let mut v=vec![0.0f32;52];for(st2,s)in &spk{if *st2>=*t&&*st2<t+500&&(24..76).contains(s){v[(s-24)as usize]+=1.0}}vecs.push((p.clone(),st.clone(),v));}
    // refs: A/C from S1, D from S1D
    let mut refs:std::collections::BTreeMap<String,Vec<f32>>=Default::default();
    let mut refcnt:std::collections::BTreeMap<String,u32>=Default::default();
    // s3 response means per pattern
    let mut s3:std::collections::BTreeMap<String,Vec<f32>>=Default::default();
    let mut s3cnt:std::collections::BTreeMap<String,u32>=Default::default();
    for (p,st,v) in &vecs{
      let is_ref = (p=="A"||p=="C") && st=="S1" || p=="D" && st=="S1D";
      let is_s3 = st=="S3A"||st=="S3C"||st=="S3D";
      if is_ref { let e=refs.entry(p.clone()).or_insert_with(||vec![0.0f32;52]); for i in 0..52{e[i]+=v[i]}; *refcnt.entry(p.clone()).or_insert(0)+=1; }
      if is_s3 { let e=s3.entry(p.clone()).or_insert_with(||vec![0.0f32;52]); for i in 0..52{e[i]+=v[i]}; *s3cnt.entry(p.clone()).or_insert(0)+=1; }
    }
    for(p,v)in refs.iter_mut(){let c=refcnt[p]as f32;for x in v.iter_mut(){*x/=c}}
    for(p,v)in s3.iter_mut(){let c=s3cnt[p]as f32;for x in v.iter_mut(){*x/=c}}
    let name=dir.rsplit('/').next().unwrap().to_string();
    for p in ["A","C","D"]{
      if !refs.contains_key(p)||!s3.contains_key(p){println!("{name} {}: missing ref/S3 (ref={} s3={})", p, refs.contains_key(p), s3.contains_key(p));continue}
      let others:Vec<&String>=refs.keys().filter(|q| *q!="B" && *q!=p).collect();
      for o in &others{
        let rho=cosv(&s3[p],&refs[p])-cosv(&s3[p],&refs[*o]);
        println!("{name} {} rho(S3{} vs {}): {rho:+.3} (cos-self={:.3})",
          p, p, *o, cosv(&s3[p],&refs[p]));
      }
    }
  }
}