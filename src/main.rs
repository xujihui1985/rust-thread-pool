use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use threaddemo::List::{Cons, Nil};
use threaddemo::ThreadPool;

fn main() {
    let handle = thread::spawn(|| {
        for i in 1..10 {
            println!("number {} from spawned thread", i);
            thread::sleep(Duration::from_millis(1));
        }
    });

    let v = vec![1, 2, 3];
    let v2 = vec![1, 2, 3];
    thread::spawn(move || { // v was moved into thread
        println!("{:?}", v);
    });
    // println!("{:?}", v); // this will cause compile error as v has beem moved into thread
    println!("{:?}", v2);

    for i in 1..5 {
        println!("number {} from main thread", i);
        thread::sleep(Duration::from_millis(1));
    }
    handle.join().unwrap();

    // multiple produce single consumer
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let v = String::from("hello from thread111");
        tx.send(v);
    });

    let recived = rx.recv().unwrap();
    println!("received from thread111 ({})", recived);

    let pool = ThreadPool::new(10).unwrap();
    for id in 0..20 {
        pool.execute(move || {
            println!("hello from {} thread pool", id);
        });
    }

    thread::sleep(Duration::from_secs(1));

    let l = Cons(123,
                 Box::new(Cons(2,
                               Box::new(Cons(3,
                                             Box::new(Nil))))));
    match l {
        Nil => println!("nil"),
        Cons(i, bl) => println!("item is {}", i),
    }
}
