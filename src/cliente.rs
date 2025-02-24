
use std::io::{self, Write, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::net::{TcpStream, Shutdown};
//use serde::Deserialize;
pub mod config;
use config::Config;

static mut CTRL_SIGNAL: u8 = 0;

pub fn cliente() -> io::Result<()> {

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Set up the Ctrl+C handler
    ctrlc::set_handler(move || {
        log::info!("Client: Ctrl+C received! Shutting down...");
        r.store(false, Ordering::SeqCst); // Set the flag to false
        unsafe{
            CTRL_SIGNAL = 255;
        }
    }).expect("Error setting Ctrl+C handler");

    // Connect to the server
    let conf: Config = config::get_configuration();
    let hostip_port1: String = config::get_hostip(&conf)+":"+&config::get_port1(&conf) ;
    log::info!("Connected to server at {}",hostip_port1);
    let mut stream = TcpStream::connect(hostip_port1)?;

    let mut buffer = [0; 512];

    // Read the server's prompt for the password
    let n = stream.read(&mut buffer)?;
    let prompt = String::from_utf8_lossy(&buffer[..n]);

    // If the password was incorrect, exit
    if prompt.contains("Please send the SENHA") {
        // Send the password to the server
        let password =  config::get_password(&conf);
        //io::stdin().read_line(&mut password)?;
        stream.write_all(password.trim().as_bytes())?;       
        log::info!("Client: Password sent.");
    }

    // Read the server's response to the password
    let n = stream.read(&mut buffer)?;
    let response: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&buffer[..n]);

    // If the password was incorrect, exit
    if response.contains("Invalid password") {
        return Ok(());
    }

    // Read the server's response
    //let n = stream.read(&mut buffer)?;
    //let response = String::from_utf8_lossy(&buffer[..n]);
//    stream.shutdown(Shutdown::Both)?;

    if response.contains("You are now connected") {
        log::info!("Client: The server autorized my connection");
    }else{
        log::info!("Client: Erro verificando se connected");
        return Ok(());
    }

    
    loop{        
        //TODO: 🌞 Put a timeout if sever does not send a message in time, shutdown client.
        let n = stream.read(&mut buffer)?;
        let response = String::from_utf8_lossy(&buffer[..n]);
        unsafe{
            if CTRL_SIGNAL == 255 {
                stream.shutdown(Shutdown::Both).unwrap();
                log::info!("Client: Finishing the client Id: {}",config::get_id(&conf));
                break
            }
        }
        if response.contains("100:") {
            log::info!("Client:{} msg {}",config::get_id(&conf),response);
        }
        if response.contains("110:") {
            log::info!("Client:{} msg {}",config::get_id(&conf),response);
            stream.write_all(config::get_id(&conf).trim().as_bytes())?;       
            log::info!("Client: id sent.");            
        }
    }    
    Ok(())
}