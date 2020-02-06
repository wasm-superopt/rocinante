extern crate bus;
extern crate clap;
extern crate num_cpus;

use clap::{App, Arg};
use rand::prelude::*;
use std::sync::mpsc::{sync_channel, TryRecvError};
use std::thread;
use std::time::Duration;

fn main() {
    let matches = App::new(
        "Programming excercise using multi-producer
         single consumer channel, and broadcast channel.",
    )
    .arg(
        Arg::with_name("wait_for_all")
            .help("If set waits for all workers to terminate.")
            .short("w"),
    )
    .get_matches();
    // Get the number of logical cores available on the machine.
    let num_workers = num_cpus::get();

    let wait_for_all = matches.is_present("w");

    // A buffered channel to receive computation results from workers.
    let (res_sender, res_receiver) = sync_channel(num_workers);
    // A broadcast channel to terminate workers.
    let mut bus = bus::Bus::new(num_workers);

    let mut rng = thread_rng();
    for i in 0..num_workers {
        let second = rng.gen_range(0, 30);
        let res_sender_i = res_sender.clone();
        let mut bus_rx_i = bus.add_rx();
        thread::spawn(move || {
            println!("worker {}, sleeping for {} seconds", i, second);
            let mut t = 0;
            while t < second {
                match bus_rx_i.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => {
                        break;
                    }
                    Err(TryRecvError::Empty) => {
                        thread::sleep(Duration::from_secs(1));
                    }
                }
                t += 1;
            }
            res_sender_i.send((i, t == second)).unwrap();
        });
    }

    for _ in 0..num_workers {
        let (i, completed) = res_receiver.recv().unwrap();
        if completed {
            println!("Worker {} completed its workload.", i);
        } else {
            println!("Worker {} terminated by broadcast.", i);
        }
        if !wait_for_all && completed {
            // NOTE(taegyunkim): Not sure what would happen if this broadcast is called multiple
            // times. It seems to be ok for above simple example.
            bus.broadcast(());
        }
    }
}
