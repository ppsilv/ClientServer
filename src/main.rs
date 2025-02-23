use std::env;
use syslog::{Facility, Formatter3164, BasicLogger};
use std::process;

pub mod servidor;
pub mod cliente;

/*
   how to call module 
   servidor::servidor().unwrap();
*/ 
fn main() -> std::io::Result<()>{
    let args: Vec<String> = env::args().collect();
    // Create a formatter for the log messages
    let hname = hostname::get().expect("Failed to get hostname");
    let hostname: String = hname.to_string_lossy().into_owned();
    let pid: u32 = process::id() as u32; // Cast to i32 if needed
    println!("Hostname: {}", hostname);
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: Some(hostname),
        process: "EnergyServer".into(),
        pid: pid,
    };

    // Initialize the logger
    let logger = syslog::unix(formatter).expect("could not connect to syslog");

    // Set the logger as the global logger
    log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
        .map(|()| log::set_max_level(log::LevelFilter::Info))
        .expect("could not set logger");


    if args.len() < 2 {
        eprintln!("Usage: {} <name> [server|client]", args[0]);
        std::process::exit(1);
    }

    let name = &args[1];
    
    if name.contains("server") {
        // Log a message
        log::info!("Starting Energy server monitor...");
        servidor::servidor().unwrap();
    }else{
        // Log a message
        log::info!("Starting Energy client monitor...");
        cliente::cliente().unwrap();
    }

    Ok(())  
}