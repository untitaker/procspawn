use std::env;
use std::thread;
use std::time::Duration;

use procspawn::{self, spawn};

procspawn::enable_test_support!();

#[test]
fn test_basic() {
    let handle = spawn(true, |b| !b);
    let value = handle.join().unwrap();

    assert!(!value);
}

#[test]
fn test_panic() {
    let handle = spawn((), |()| panic!("something went wrong"));
    let err = handle.join().unwrap_err();

    let panic_info = err.panic_info().unwrap();
    assert_eq!(panic_info.message(), "something went wrong");
    assert!(panic_info.backtrace().is_some());

    let loc = panic_info.location().unwrap();
    assert_eq!(loc.line(), 19);
    assert_eq!(loc.column(), 33);
    assert!(loc.file().contains("test_basic.rs"));
}

#[test]
fn test_kill() {
    let mut handle = spawn((), |()| {
        thread::sleep(Duration::from_secs(10));
    });
    handle.kill().unwrap();
    let err = handle.join().unwrap_err();
    dbg!(&err);
    assert!(err.is_remote_close());
}

#[test]
fn test_envvar() {
    let val = procspawn::Builder::new()
        .env("FOO", "42")
        .spawn(23, |val| {
            env::var("FOO").unwrap().parse::<i32>().unwrap() + val
        })
        .join()
        .unwrap();
    assert_eq!(val, 42 + 23);
}

#[test]
fn test_nested() {
    let five = spawn(5, |x| {
        println!("1");
        let x = spawn(x, |y| {
            println!("2");
            y
        })
        .join()
        .unwrap();
        println!("3");
        x
    })
    .join()
    .unwrap();
    println!("4");
    assert_eq!(five, 5);
}

#[test]
fn test_timeout() {
    let handle = spawn((), |()| {
        thread::sleep(Duration::from_secs(10));
    });

    let err = handle.join_timeout(Duration::from_millis(100)).unwrap_err();
    assert!(err.is_timeout());

    let handle = spawn((), |()| {
        thread::sleep(Duration::from_millis(100));
        42
    });

    let val = handle.join_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(val, 42);
}
