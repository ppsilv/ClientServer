use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::io::ErrorKind;
use std::fs::OpenOptions;

//use syslog::{Facility, Formatter3164, BasicLogger};
pub mod config;
use config::Config;

// Struct to store client data
#[derive(Debug, Clone)]
struct ClientData {
    id: u16,
    ip: String,
    status: String, // "active" or "inactive"
    port: String, //Port of client
}

static mut COUNTER: u64 = 0;
static MSGCODE100: u16 = 100;
static PADDED_STRING100: &str = ": keep alive ";


fn save_client_data(clients: &Arc<Mutex<Vec<ClientData>>>, client_id: u16, client_ip: String,client_port: String ){
    let client_data = ClientData {
        id: client_id.clone(),
        ip: client_ip.clone(),
        status: String::from("active"), // Set status to "active"
        port: client_port,
    };
    clients.lock().unwrap().push(client_data);
}
/// Finds the first inactive client and returns its ID.
/// Returns 0 if no inactive clients are found.
fn find_first_inactive(clients: &Arc<Mutex<Vec<ClientData>>>) -> u16 {
    let clients = clients.lock().unwrap();
    for client in clients.iter()  {
        if client.status == "inactive" {
            return client.id; // Return the ID of the first inactive client
        }
    }
    0 // Return 0 if no inactive clients are found
}
fn find_client_by_id(clients: &Arc<Mutex<Vec<ClientData>>>, target_id: u16) -> u8 {
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
fn update_client_status(clients: &Arc<Mutex<Vec<ClientData>>>,target_id: u16, new_status: &str) -> bool {
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

fn _update_client_port(clients: &mut Vec<ClientData>, target_id: u16, new_port: &str) -> bool {
    // Search for the client with the matching ID
    if let Some(client) = clients.iter_mut().find(|c| c.id == target_id) {
        // Update the port
        client.port = new_port.to_string();
        true // Return true if the client was found and updated
    } else {
        false // Return false if the client was not found
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

// Function to handle client connections
fn handle_client(mut stream: TcpStream,client_id: u16,config: Config ,clients: Arc<Mutex<Vec<ClientData>>>){
    let mut buffer = [0; 512];
    let client_addr = stream.peer_addr().unwrap();
    let clientip = stream.peer_addr().unwrap().to_string();

    // Store the result of `split_once` in a temporary variable
    let (ip, port) = if let Some((ip, port)) = clientip.split_once(':') {
        (ip, port)
    } else {
        eprintln!("Invalid client IP format");
        return; // Or handle the error appropriately
    };

    // Now you can safely mutate `client_ip` and `client_port`
    let client_ip = ip.to_string();
    let client_port = port.to_string();

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
        log::info!("Server: Cliente autorized");
    } else {
        if let Err(e) = stream.write(b"Invalid password. Closing connection.\n") {
            log::error!("Failed to write to socket: {}", e);
        }
        log::error!("Client {} provided an incorrect password.", client_ip);
        return ;
    }

    let client_result: u8 = find_client_by_id(&clients, client_id);
    if client_result == 1 {
        log::error!("Already has a client with this ID {}",client_id);
        if let Err(e) = stream.write(b"You are now DESconnected.\n") {
            log::error!("Failed to write to socket: {}", e);
        }
        stream.shutdown(Shutdown::Both).unwrap();
        return;
    }else if client_result == 0 {
        // Save the client's data to the list
        save_client_data(&clients, client_id, client_ip, client_port );
    }else if client_result == 2 {
        update_client_status(&clients, client_id,"active");
    }
   // log::info!("Server: Client connected - ID: {}, IP: {}", client_id, client_ip);
    list_connected_clients(&clients);

    if let Err(e) = stream.write(b"Thank you! You are now connected.\n") {
        log::error!("Server: Failed to write to socket: {}", e);
        return;
    }

    // Main loop to handle client
    loop {
        unsafe{         
            let keep_alive =  format!("{}{} {} {}",MSGCODE100, PADDED_STRING100, client_id ,COUNTER);   
            match stream.write( keep_alive.as_bytes()  ){
                Ok(_) => {
                } // Success
                Err(e) if e.kind() == ErrorKind::BrokenPipe => {
                    log::error!("Server: Client {} ID {} disconnected abruptly.", client_addr, client_id);
                    log_client_disconnect(client_addr, "broken pipe");
                
                    if update_client_status( &clients,client_id,"inactive"){
                        log::info!("Server: Cliente Id {} inactivated...",client_id);     
                    }
                    break;
                }
                Err(e) => {
                    log::error!("Server: Failed to write to client {}: {}", client_addr, e);
                    log_client_disconnect(client_addr, &format!("error: {}", e));
                    break ;
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
fn handle_port( listener1: TcpListener,mut client_id: u16,config: Config , clients: Arc<Mutex<Vec<ClientData>>>){
    // Accept connections in a loop
    for stream in listener1.incoming() {
        match stream {
            Ok(stream) => {
                //println!("New connection from: {:?} on port 1", stream.peer_addr());
                let clients = Arc::clone(&clients);
                let config: Config =config.clone();
                // Spawn a thread that panics
                println!("Lets spawn thread handle_client..");

                let new_client = find_first_inactive(&clients);
                if new_client > 0 {
                    client_id = new_client;
                } 
                thread::spawn(move|| {
                    handle_client(stream,client_id,config,clients);
                });
              
            }
            Err(e) => {
                log::error!("Failed to accept connection: {}", e);
            }
        }
        client_id += 10;
    }
}


pub fn servidor() -> std::io::Result<()> {
    let conf: Config = config::get_configuration();
    log::info!("Server: Server running...");

    let hostip_port1: String = config::get_hostip(&conf)+":"+&config::get_port1(&conf) ;
    log::info!("Server: Server listening on port {}",hostip_port1);
    let listener1 = TcpListener::bind(hostip_port1)?;

    let hostip_port3: String = config::get_hostip(&conf)+":"+&config::get_port3(&conf) ;
    log::info!("Server: Server listening on port {}",hostip_port3);
    let listener3 = TcpListener::bind(hostip_port3)?;


    // Shared list of clients (thread-safe)

    let clients = Arc::new(Mutex::new(Vec::<ClientData>::new()));
    //let clients2 = Arc::clone(&clients);
    let clients3 = Arc::clone(&clients);
    let client_id: u16 = 1000;
    thread::spawn(move || {
        handle_port(listener1,client_id,conf.clone(),clients);
    });

    thread::spawn(move || {
        let ( stream, _addr) = listener3.accept().unwrap();
        handle_read_client_port3(stream, clients3).unwrap();
    });


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
