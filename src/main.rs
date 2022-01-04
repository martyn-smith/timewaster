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

/*
 * CONSIDER: not checking for a solution every round.
 * The downside of this, other than slightly higher complication, is a
 * slight inefficiency; if we only check every n times, up to n hashes
 * are "wasted" (per thread). But overall we'd expect a speedup.
 */
fn hunt(tx: mpsc::Sender<Vec<u8>>, solved: Arc<AtomicBool>, difficulty: usize) {
    let mut rng = rand::thread_rng();
    let mut cand = vec![0u8; 64];

    loop {
        if solved.load(Ordering::SeqCst) {
            break;
        }
        rng.fill(&mut cand[..]);
        let test = Sha256::digest(&cand);
        if test[0..difficulty] == cand[0..difficulty] {
            solved.store(true, Ordering::SeqCst);
            tx.send(cand).unwrap();
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
            thread::spawn(move || hunt(tx, solved, difficulty))
        })
        .collect::<Vec<thread::JoinHandle<_>>>();
    for h in handles {
        h.join().unwrap();
    }
    let answer = rx.recv().unwrap();
    let msg = answer.iter().fold(String::new(), |out, i| format!("{}{:02x}", out, i));
    println!("{}\tpartially matches its own hash.", msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_hash() {
        let matching = "d7d0ffe6d449ecd1cb391f7e1c5b348c8762c2f48b59706f11e0fcef40dd6a92248937334a96e81d59db10eea775bd1630f2bacd403f83f2b44cf309876176b2";
        let m = matching.as_bytes()
                        .chunks(2)
                        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
                        .collect::<Vec<u8>>();
        let test = Sha256::digest(&m);
        assert_eq!(test[0..3], m[0..3]);
        let msg = m.iter().fold(String::new(), |out, i| format!("{}{:02x}", out, i));
        assert_eq!(matching, msg);
    }

    #[test]
    fn unknown_hash() {
        let solved = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        hunt(tx, solved, 3);
        let m = rx.recv().unwrap();
        let test = Sha256::digest(&m);
        assert_eq!(test[0..3], m[0..3]);
    }

}
