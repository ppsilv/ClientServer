use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::io::ErrorKind;
use std::fs::OpenOptions;
use lazy_static::lazy_static;


//use syslog::{Facility, Formatter3164, BasicLogger};
pub mod config;
use config::Config;

lazy_static! {
    static ref RECEIVED_ID: Mutex<String> = Mutex::new("".to_string());
}

// Struct to store client data
#[derive(Debug, Clone)]
struct ClientData {
    id: String,
    ip: String,
    status: String, // "active" or "inactive"
    port: String, //Port of client
}

static mut COUNTER: u64 = 0;
static MSGCODE100: u16 = 100;
//static MSGCODE101: u16 = 101;
//static mut padded_string: String = "Message to cliente".to_string();
static PADDED_STRING100: &str = ": keep alive ";
//static PADDED_STRING101: &str = "Shutdown: Message ";

fn find_client_by_id(clients: &Arc<Mutex<Vec<ClientData>>>, target_id: &str) -> u8 {
    // Acquire the lock on the Mutex
    let clients = clients.lock().unwrap();
    // Iterate over the clients and check for a matching ID
    for client in clients.iter() {
        if client.id == target_id {
            if client.status == "active" {
                return 1
            }
            if client.status == "inactive" {
                return 2
            }
        }
    }
    // Return false if no matching client is found
    return 0
}
fn update_client_status(
    clients: &Arc<Mutex<Vec<ClientData>>>,
    target_id: &str,
    new_status: &str,
) -> bool {
    let clients1: String = list_connected_clients(&clients);
    println!("Clientes {}",clients1);

    // Lock the Mutex and handle poisoning
    let mut clients_lock = match clients.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Mutex was poisoned, recovering data...");
            poisoned.into_inner()
        }
    };

    // Search for the client with the matching ID
    if let Some(client) = clients_lock.iter_mut().find(|c| {
        c.id == target_id
    }) {
        // Update the status
        client.status = new_status.to_string();
        true // Return true if the client was found and updated
    } else {
        false // Return false if the client was not found
    }
    
}

fn _update_client_port(clients: &mut Vec<ClientData>, target_id: &str, new_port: &str) -> bool {
    // Search for the client with the matching ID
    if let Some(client) = clients.iter_mut().find(|c| c.id == target_id) {
        // Update the port
        client.port = new_port.to_string();
        true // Return true if the client was found and updated
    } else {
        false // Return false if the client was not found
    }
}
// Function to handle client connections
fn handle_client(mut stream: TcpStream,config: Config ,clients: Arc<Mutex<Vec<ClientData>>>) {
    let mut buffer = [0; 512];
    let client_ip = stream.peer_addr().unwrap().to_string();
    log::info!("Server: Thread spawned for client: {}", client_ip);

    // Ask the client to send the password
    if let Err(e) = stream.write(b"Please send the SENHA:\n") {
        log::error!("Server: Failed to write to socket: {}", e);
        return;
    }

    // Read the password from the client
    let n = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(e) => {
            log::error!("Server: Failed to read from socket: {}", e);
            return;
        }
    };

    // Convert the received data to a string and trim whitespace
    let received_password = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

    // Predefined password (e.g., "1234")
    let correct_password = config::get_password(&config); //"1234";

    // Validate the password
    if received_password == correct_password {
        if let Err(e) = stream.write(b"Password correct. Please send your ID:\n") {
            log::error!("Failed to write to socket: {}", e);
            return;
        }
    } else {
        if let Err(e) = stream.write(b"Invalid password. Closing connection.\n") {
            log::error!("Failed to write to socket: {}", e);
        }
        log::error!("Client {} provided an incorrect password.", client_ip);
        return; // Close the connection
    }

    // Read the client's ID
    let n = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(e) => {
            log::error!("Failed to read from socket: {}", e);
            return;
        }
    };

    let mut received_id = RECEIVED_ID.lock().unwrap();
    let client_id = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
    *received_id = client_id.clone();

    let client_result: u8 = find_client_by_id(&clients, client_id.as_str());
    if client_result == 1 {
        log::error!("Already has a client with this ID {}",client_id);
        if let Err(e) = stream.write(b"You are now DESconnected.\n") {
            log::error!("Failed to write to socket: {}", e);
        }
        stream.shutdown(Shutdown::Both).unwrap();
        return;
    }else if client_result == 0 {
        // Save the client's data to the list
        let client_data = ClientData {
            id: client_id.clone(),
            ip: client_ip.clone(),
            status: String::from("active"), // Set status to "active"
            port: String::from("none"),
        };
        clients.lock().unwrap().push(client_data);
    }else if client_result == 2 {
        update_client_status(&clients, client_id.as_str(),"active");
    }
    log::info!("Server: Client connected - ID: {}, IP: {}", client_id, client_ip);
    list_connected_clients(&clients);

    if let Err(e) = stream.write(b"Thank you! You are now connected.\n") {
        log::error!("Server: Failed to write to socket: {}", e);
        return;
    }
    // Main loop to handle client requests
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                // Connection was closed by the client
                log::info!("Server: Port1 disconnected: {}", client_ip);
                break;
            }
            Ok(n) => {
                // Convert the received data to a string
                let received_data = String::from_utf8_lossy(&buffer[..n]);
                //println!("Received from {} ({}): {}", client_id, client_ip, received_data);

                // Handle the LISTAR command
                if received_data.trim() == "LISTAR" {
                    let clients_list = list_connected_clients(&clients);
                   
                    if let Err(e) = stream.write(clients_list.as_bytes()) {
                        log::error!("Failed to write to socket: {}", e);
                        break;
                    }
                    continue;
                }                
                // Check if the received data contains the "FECHAR" command
                if received_data.trim() == "FECHAR" {
                    log::info!("Server: Closing connection with client: {} ({})", client_id, client_ip);
                    break; // Exit the loop to close the connection
                }

                // Echo the data back to the client
                if let Err(e) = stream.write(&buffer[..n]) {
                    log::error!("Server: Failed to write to socket: {}", e);
                    break;
                }
            }
            Err(e) => {
                log::error!("Server: Failed to read from socket: {}", e);
                break;
            }
        }
    }
}
fn handle_port1( listener: TcpListener,config: Config , clients: Arc<Mutex<Vec<ClientData>>>){
    // Accept connections in a loop
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                //println!("New connection from: {:?} on port 1", stream.peer_addr());
                let clients = Arc::clone(&clients);
                let config: Config =config.clone();
                thread::spawn(move || {
                    handle_client(stream,config,clients);
                });
            }
            Err(e) => {
                log::error!("Failed to accept connection: {}", e);
            }
        }
    }

}
fn log_client_disconnect(client_addr: std::net::SocketAddr, reason: &str) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("client_log.txt")
        .unwrap();

    writeln!(file, "Client {} disconnected: {}", client_addr, reason).unwrap();
}
fn handle_write_client_port2(mut stream: TcpStream, clients: Arc<Mutex<Vec<ClientData>>>) -> Result<(), std::io::Error> {
   // let mut buffer = [0; 512];
    let client_addr = stream.peer_addr().unwrap();
    let client_ip = stream.peer_addr().unwrap().to_string();

    // Ask the client to send the ID
    if let Err(e) = stream.write(b"Please send the ID:\n") {
        log::error!("Failed to write to socket: {}", e);
        return Ok(());
    }

    let received_id = RECEIVED_ID.lock().unwrap();


    //let  received_id: String = "SERVER".to_string(); 

    log::info!("Server: Thread spawned for client: {} Id: {}", client_ip, received_id);

    // Main loop to handle client
    loop {
        unsafe{         
            let keep_alive =  format!("{}{} {} {}",MSGCODE100, PADDED_STRING100, received_id ,COUNTER);   
            match stream.write( keep_alive.as_bytes()  ){
                Ok(_) => {
                } // Success
                Err(e) if e.kind() == ErrorKind::BrokenPipe => {
                    log::error!("Server: Client {} ID {} disconnected abruptly.", client_addr, received_id);
                    log_client_disconnect(client_addr, "broken pipe");
                    // Update the client's status to "inactive"
                    //let mut clients: &Arc<Mutex<Vec<ClientData>>> = clients.lock().unwrap();
                
                    if update_client_status( &clients,received_id.as_str(),"inactive"){
                        log::info!("Server: Cliente Id {} inativado...",received_id);     
                    }
             
                    break Ok(());
                }
                Err(e) => {
                    log::error!("Server: Failed to write to client {}: {}", client_addr, e);
                    log_client_disconnect(client_addr, &format!("error: {}", e));
                    // Update the client's status to "inactive"
                  //  let mut clients = clients.lock().unwrap();
                  //  if let Some(client) = clients.iter_mut().find(|c| c.id == received_id) {
                  //      client.status = String::from("inactive");
                  //  }                
                    break Err(e);
                }
            }
            println!("Server: Message sent {}",keep_alive);
        }
        thread::sleep(Duration::from_secs(5));
        unsafe{
            COUNTER+=1;
        }
               
    }
}
fn _handle_read_client_port2(mut stream: TcpStream, clients: Arc<Mutex<Vec<ClientData>>>) -> Result<(), std::io::Error> {
    let mut buffer = [0; 512];
    println!("New client connected: {}", stream.peer_addr().unwrap());

    thread::sleep(Duration::from_secs(5));
    /*
    The line below is in thread handle_write_client_port2 and handle_read_client_port2
    and it locks both of threa so I put a delay of 5 seconds above this is durty and quick, 
    I'm too lazy to deal with deadlock and on top of that I'm not even using the port2 socket read
    */
    let received_id = RECEIVED_ID.try_lock().unwrap();
    loop {
        // Read data from the client 
        match stream.read(&mut buffer) {
            Ok(0) => {
                //stream.shutdown(Shutdown::Both).unwrap();
                // Client disconnected
                log::error!("Client ID {} disconnected .", *received_id);
                // Update the client's status to "inactive"
                if update_client_status( &clients,received_id.as_str(),"inactive"){
                    log::info!("Server: Cliente Id {} inativado...",*received_id);     
                }
            break;
            }
            Ok(n) => {
                // Echo the data back to the client
                let mut received_id = RECEIVED_ID.lock().unwrap();
                let received_data = String::from_utf8_lossy(&buffer[..n]);
                log::info!("Server: Received: {}", received_data);
                *received_id = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

                if let Err(e) = stream.write(&buffer[..n]) {
                    log::error!("Server: Failed to write to client: {}", e);
                    break;
                }
            }
            Err(e) => {
                log::error!("Server: Failed to read from client: {}", e);
                break;
            }
        }

        // Simulate some processing time
        thread::sleep(Duration::from_secs(1));
    }

    println!("Client handler thread exiting for Id: {}", received_id);

    Ok(())
}
fn handle_port2(listener: TcpListener, clients: Arc<Mutex<Vec<ClientData>>>) {
    // Accept connections in a loop
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                log::info!("Server: New connection from: {:?} in port 2", stream.peer_addr());
                let clients1: String = list_connected_clients(&clients);
                log::info!("Server: {}", clients1);
/*
                // Clone the TcpStream for the second thread
                let stream_clone = match stream.try_clone() {
                    Ok(clone) => clone,
                    Err(e) => {
                        log::error!("Server: Failed to clone TcpStream: {}", e);
                        continue;
                    }
                };
*/
                // Clone the Arc for the threads
                let clients_write = Arc::clone(&clients);
                //let clients_read = Arc::clone(&clients);

                // Spawn a thread for writing to the client
                log::info!("Server: Starting server thread write client port2");
                thread::spawn(move || {
                    if let Err(e) = handle_write_client_port2(stream, clients_write) {
                        log::error!("Error in write handler: {}", e);
                    }
                });
                /*
                // Spawn a thread for reading from the client
                thread::spawn(move || {
                    if let Err(e) = handle_read_client_port2(stream_clone, clients_read) {
                        log::error!("Error in read handler: {}", e);
                    }
                });
                */
            }
            Err(e) => {
                log::error!("Failed to accept connection: {}", e);
            }
        }
    }
    loop {
        log::info!("Server: Sleepiong"); 
        thread::sleep(Duration::from_secs(60));
    }
}
// Function to list connected clients
fn list_connected_clients(clients: &Arc<Mutex<Vec<ClientData>>>) -> String {
    let clients = clients.lock().unwrap();
    if clients.is_empty() {
        return "No clients connected.\n".to_string();
    }

    let mut result = String::from("Connected clients:\n");
    for client in clients.iter() {
        result.push_str(&format!("ID: {}, IP: {}, Status: {} Port2: {}\n", client.id, client.ip, client.status, client.port));
    }
    result
}

pub fn servidor() -> std::io::Result<()> {
    let conf: Config = config::get_configuration();
    log::info!("Server: Server running...");

    let hostip_port1: String = config::get_hostip(&conf)+":"+&config::get_port1(&conf) ;
    log::info!("Server: Server listening on port {}",hostip_port1);
    let listener1 = TcpListener::bind(hostip_port1)?;

    let hostip_port2: String = config::get_hostip(&conf)+":"+&config::get_port2(&conf) ;
    log::info!("Server: Server listening on port {}",hostip_port2);
    let listener2 = TcpListener::bind(hostip_port2)?;

    let hostip_port3: String = config::get_hostip(&conf)+":"+&config::get_port3(&conf) ;
    log::info!("Server: Server listening on port {}",hostip_port3);
    let listener3 = TcpListener::bind(hostip_port3)?;


    // Shared list of clients (thread-safe)

    let clients = Arc::new(Mutex::new(Vec::<ClientData>::new()));
    let clients2 = Arc::clone(&clients);
    let clients3 = Arc::clone(&clients);
    thread::spawn(move || {
        handle_port1(listener1,conf.clone(), clients);
    });
    thread::spawn(move || {
        handle_port2(listener2, clients2);
    });
    thread::spawn(move || {
        let ( stream, _addr) = listener3.accept().unwrap();
        handle_read_client_port3(stream, clients3).unwrap();
    });

    log::info!("Server: Server waiting for connections...!");
    loop {
        thread::sleep(Duration::from_secs(11));
    }
}

fn handle_read_client_port3(mut stream: TcpStream, clients: Arc<Mutex<Vec<ClientData>>>) -> Result<(), std::io::Error> {
    let mut buffer = [0; 512];
    println!("New client connected: {}", stream.peer_addr().unwrap());

    loop {
        // Read data from the client
        println!("Reading socket...");
        match stream.read(&mut buffer) {
            Ok(0) => {
                // Client disconnected
                log::error!("Client backdoor disconnected .");
            break;
            }
            Ok(n) => {
                // Echo the data back to the client
                let received_data = String::from_utf8_lossy(&buffer[..n]);
                log::info!("Server: Received: {}", received_data);
                // Handle the LISTAR command
                if received_data.trim() == "LISTAR" {
                    let clients_list = list_connected_clients(&clients);
                   
                    if let Err(e) = stream.write(clients_list.as_bytes()) {
                        log::error!("Failed to write to socket: {}", e);
                        break;
                    }
                    continue;
                }                
                
//                if let Err(e) = stream.write(&buffer[..n]) {
//                    log::error!("Server: Failed to write to client: {}", e);
//                    break;
//                }
            }
            Err(e) => {
                log::error!("Server: Failed to read from client: {}", e);
                break;
            }
        }

        // Simulate some processing time
        thread::sleep(Duration::from_secs(1));
    }
    println!("client backdood desconecting...");
    Ok(())
}
