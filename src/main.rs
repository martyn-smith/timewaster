use rand::prelude::*;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

const DIFFICULTY: usize = 3;
const NUM_THREADS: usize = 4;

fn try_hash(tx: mpsc::Sender<Vec<u8>>, solved: Arc<AtomicBool>) {
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
        if g.finalize()[0..DIFFICULTY] == x[0..DIFFICULTY] {
            solved.store(true, Ordering::SeqCst);
            tx.send(x).unwrap();
            break;
        }
    }
}

fn main() {
    println!("initialising... ");
    let solved = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    let handles = (0..NUM_THREADS - 1)
        .map(|_| {
            let tx = tx.clone();
            let solved = solved.clone();
            thread::spawn(move || try_hash(tx, solved))
        })
        .collect::<Vec<thread::JoinHandle<_>>>();
    for h in handles {
        h.join().unwrap();
    }
    let answer = rx.recv().unwrap();
    println!("{:x?} partially matches its own hash.", answer);
}
