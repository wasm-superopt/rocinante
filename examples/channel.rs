use rand::prelude::*;
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Duration;

fn main() {
    let num_workers = 10;

    let (sender, receiver) = sync_channel(num_workers);

    let mut rng = thread_rng();
    for i in 0..num_workers {
        let second = rng.gen_range(0, 30);
        let sender_i = sender.clone();
        thread::spawn(move || {
            println!("worker {}, sleeping for {} seconds", i, second);
            thread::sleep(Duration::from_secs(second));
            sender_i.send(i).unwrap();
        });
    }

    for _ in 0..num_workers {
        println!("worker {} terminated", receiver.recv().unwrap());
    }
}
