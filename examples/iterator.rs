extern crate itertools;

use itertools::Itertools;

fn main() {
    let nums: Vec<_> = (1..40).collect();

    let mut count: i64 = 0;
    for i in 1..=7 {
        let iter = (0..i).map(|_| nums.iter()).multi_cartesian_product();

        for _item in iter {
            count += 1;
        }
    }

    println!("{}", count);
}
