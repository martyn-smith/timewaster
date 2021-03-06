use rand::prelude::*;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "3")]
    difficulty: usize,
    #[structopt(short = "t", long, default_value = "4")]
    num_threads: usize,
}

fn try_hash(tx: mpsc::Sender<Vec<u8>>, solved: Arc<AtomicBool>, difficulty: usize) {
    let h = Sha256::new();
    let mut rng = rand::thread_rng();
    loop {
        if solved.load(Ordering::SeqCst) == true {
            break;
        }
        let x: Vec<u8> = (0..64).map(|_| rng.gen::<u8>()).collect();
        //println!("trying: {:?}", &x[0..64]);
        let mut g = h.clone();
        g.update(&x);
        if g.finalize()[0..difficulty] == x[0..difficulty] {
            solved.store(true, Ordering::SeqCst);
            tx.send(x).unwrap();
            break;
        }
    }
}

fn main() {
    println!("initialising... ");
    let opt = Opt::from_args();
    let solved = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    let handles = (1..opt.num_threads)
        .map(|_| {
            let tx = tx.clone();
            let solved = solved.clone();
            let difficulty = opt.difficulty;
            thread::spawn(move || try_hash(tx, solved, difficulty))
        })
        .collect::<Vec<thread::JoinHandle<_>>>();
    for h in handles {
        h.join().unwrap();
    }
    let answer = rx.recv().unwrap();
    println!("{:x?} partially matches its own hash.", answer);
}
