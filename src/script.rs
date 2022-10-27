//! Given a command in the environment variable `WANNADO_SCRIPT`, execute it
//! in a sub-process every five minutes. This can connect to the app's
//! database, perform cleanup, insert recurring items, etc.
//!
//! Note the program does no shell processing; the argument is passed
//! straight to `std::process::Command`. If you need to run a shell command
//! you can save it in an executable script.
//!
//! Interval can be customized by setting the environment variable
//! `WANNADO_SCRIPT_INTERVAL_IN_SECONDS`.

use std::{process::Command, thread::sleep, time::Duration};

pub fn start_recurring_script() {
    if let Ok(cmd) = std::env::var("WANNADO_SCRIPT") {
        println!("Starting recurring script");
        std::thread::spawn(move || recurring_script(cmd));
    } else {
        println!("No recurring script");
    }
}

fn recurring_script(cmd: String) {
    let timeout = if let Ok(src) = std::env::var("WANNADO_SCRIPT_INTERVAL_IN_SECONDS") {
        let seconds = src.parse::<u64>().expect(
            "Invalid script timeout, expected WANNADO_SCRIPT_INTERVAL_IN_SECOND to be an integer",
        );
        let t = Duration::from_secs(seconds);
        println!("Using custon script interval: {:?}", t);
        t
    } else {
        let t = Duration::from_secs(300);
        println!("Using default script interval: {:?}", t);
        t
    };
    loop {
        if let Err(e) = Command::new(&cmd).spawn() {
            eprintln!("Error in script execution: {:?}", e);
        }
        sleep(timeout);
    }
}
