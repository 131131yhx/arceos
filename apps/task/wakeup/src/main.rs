#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;

use alloc::sync::Arc;
use alloc::vec::Vec;
use libax::thread;
use libax::time::Duration;

struct TaskParam {
    data_len: usize,
    value: u64,
    nice: isize,
}
const PAYLOAD_KIND: usize = 32000;

fn load(n: &u64) -> u64 {
    // time consuming is linear with *n
    let mut sum: u64 = *n;
    for i in 0..*n {
        sum += ((i ^ (i * 3)) ^ (i + *n)) / (i + 1);
    }
    sum
}

#[no_mangle]
fn main() {
    thread::set_priority(-20);
    let mut expect: u64 = 0;

    let sleep_dur = Duration::new(3, 0);
    let start_time = libax::time::Instant::now();
    let wakeup_time = start_time.as_duration() + sleep_dur;

    let mut tasks = Vec::with_capacity(PAYLOAD_KIND);
    for i in 0..PAYLOAD_KIND {
        tasks.push(thread::Builder::new().stack_size(0x1000).spawn(move || {
            thread::set_priority(19);

            thread::sleep_until(wakeup_time);
            //let partial_sum: u64 = vec[left..right].iter().map(load).sum();
            let leave_time = (start_time.elapsed() - sleep_dur).as_millis() as u64;

            //println!("part {}: {:?} finished", i, thread::current().id());
            (0, leave_time)
        }).expect("???"));
    }

    let (results, leave_times): (Vec<_>, Vec<_>) =
        tasks.into_iter().map(|t| t.join().unwrap()).unzip();
    let actual = results.iter().sum();

    println!("sum = {}", actual);
    //println!("leave time:");
    //for (i, time) in leave_times.iter().enumerate() {
    //    println!("task {} = {}ms", i, time);
    //}
    println!("max leave time: {}ms", leave_times.iter().max().unwrap());

    assert_eq!(expect, actual);

    println!("Unbalance tests run OK!");
}
