use natal_chart_solver::search;
use std::time::Instant;
fn main() {
    for (name, start, end, ids, angles) in [
        ("sun200",2415020.5,2488069.5,vec![10],vec![0.0]),
        ("sunmoon200",2415020.5,2488069.5,vec![10,11],vec![150.0,0.0]),
        ("mercury200",2415020.5,2488069.5,vec![0],vec![0.0]),
    ] {
        for run in 1..=2 {
            let t=Instant::now();
            let result=search(start,end,&ids,&angles,&vec![30.0;ids.len()]).unwrap();
            println!("{name} run={run} ms={:.3} windows={} checksum={:.9}",t.elapsed().as_secs_f64()*1000.0,result.len()/2,result.iter().sum::<f64>());
        }
    }
}
