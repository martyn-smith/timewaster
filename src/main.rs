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
    let mut rng = rand::thread_rng();
    let mut x = vec![0u8; 64];

    loop {
        if solved.load(Ordering::SeqCst) {
            break;
        }
        rng.fill(&mut x[..]);
        let cand = Sha256::digest(&x);
        if cand[0..difficulty] == x[0..difficulty] {
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
