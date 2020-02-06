extern crate bus;
extern crate num_cpus;

use rand::prelude::*;
use std::sync::mpsc::{sync_channel, TryRecvError};
use std::thread;
use std::time::Duration;

fn main() {
    let num_workers = num_cpus::get();

    let wait_for_all = false;

    let (res_sender, res_receiver) = sync_channel(num_workers);
    let mut bus = bus::Bus::new(num_workers);

    let mut rng = thread_rng();
    for i in 0..num_workers {
        let second = rng.gen_range(0, 30);
        let res_sender_i = res_sender.clone();
        let mut bus_rx_i = bus.add_rx();
        thread::spawn(move || {
            println!("worker {}, sleeping for {} seconds", i, second);
            for _ in 0..second {
                match bus_rx_i.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => {
                        println!("worker {} received signal to terminate.", i);
                        break;
                    }
                    Err(TryRecvError::Empty) => {
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
            res_sender_i.send(i).unwrap();
        });
    }

    for _ in 0..num_workers {
        println!("worker {} terminated", res_receiver.recv().unwrap());
        if !wait_for_all {
            break;
        }
    }

    // broadcast termination.
    bus.broadcast(());

    // Wait for workers to terminate
    thread::sleep(Duration::from_secs(5));
}
