use clap::Parser;
use colored::Colorize;
use rand::prelude::*;
use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time;

const UPDATE_FREQUENCY: usize = 1_000_000;
const REPORT_FREQUENCY: usize = 10;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "3")]
    difficulty: usize,
    #[arg(short = 't', long)]
    num_threads: Option<usize>,
}

macro_rules! hexify {
    ($vec:ident) => {
        $vec.iter().fold(String::new(), |mut out, elem| {
            let _ = write!(&mut out, "{:02x}", elem);
            out
        })
    };
}
/*
 * CONSIDER: not checking for a solution every round.
 * The downside of this, other than slightly higher complication, is a
 * slight inefficiency; if we only check every n times, up to n hashes
 * are "wasted" (per thread). But overall we'd expect a speedup.
 */
fn hunt(
    difficulty: usize,
    solved: Arc<AtomicBool>,
    solution: mpsc::Sender<Vec<u8>>,
    counter: Arc<Mutex<usize>>,
) {
    let mut rng = rand::thread_rng();
    let mut cand = vec![0u8; 64];
    let mut ctr = 0usize;

    while !solved.load(Ordering::Relaxed) {
        rng.fill(&mut cand[..]);
        let test = Sha256::digest(&cand);
        if test[0..difficulty] == cand[0..difficulty] {
            solved.store(true, Ordering::Relaxed);
            solution.send(cand.clone()).unwrap();
        }
        ctr += 1;
        if ctr % UPDATE_FREQUENCY == 0 {
            let mut counter = counter.lock().unwrap();
            *counter += 1;
        }
    }
}

fn report(solved: Arc<AtomicBool>, counter: Arc<Mutex<usize>>) {
    let mut last = 0usize;
    while !solved.load(Ordering::Relaxed) {
        thread::sleep(time::Duration::from_millis(REPORT_FREQUENCY as u64 * 1000));
        let ctr = counter.lock().unwrap();
        let next = *ctr;
        println!("{} MH/s", (next - last) / REPORT_FREQUENCY);
        last = next;
    }
}

fn main() {
    println!("initialising... ");
    let args = Args::parse();
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    let solved = Arc::new(AtomicBool::new(false));
    let counter = Arc::new(Mutex::new(0usize));
    let thread_count = args
        .num_threads
        .unwrap_or(thread::available_parallelism().unwrap().get());
    let difficulty = args.difficulty;
    thread::scope(|s| {
        for _ in 1..thread_count {
            let solution = tx.clone();
            let solved = solved.clone();
            let counter = counter.clone();
            s.spawn(move || hunt(difficulty, solved, solution, counter));
        }
        s.spawn(move || report(solved.clone(), counter.clone()));
    });
    let answer = rx.recv().unwrap();
    println!(
        "{}\npartially matches its own hash.",
        hexify!(answer).yellow()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_hash() {
        let matching = b"d7d0ffe6d449ecd1cb391f7e1c5b348c8762c2f48b59706f11e0fcef40dd6a92248937334a96e81d59db10eea775bd1630f2bacd403f83f2b44cf309876176b2";
        let m = matching
            .chunks(2)
            .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
            .collect::<Vec<u8>>();
        let test = Sha256::digest(&m);
        assert_eq!(test[0..3], m[0..3]);
        let msg = m
            .iter()
            .fold(String::new(), |out, i| format!("{}{:02x}", out, i));
        assert_eq!(matching, msg.as_bytes());
    }

    #[test]
    fn unknown_hash() {
        let solved = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let counter = Arc::new(Mutex::new(0usize));
        hunt(3, solved, tx, counter);
        let m = rx.recv().unwrap();
        let test = Sha256::digest(&m);
        assert_eq!(test[0..3], m[0..3]);
    }
}
