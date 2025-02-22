use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

pub mod config;
use config::Config;

// Struct to store client data
#[derive(Debug, Clone)]
struct ClientData {
    id: String,
    ip: String,
    status: String, // "active" or "inactive"
}

static mut COUNTER: u64 = 0;
static mut MSGCODE: u16 = 100;
//static mut padded_string: String = "Message to cliente".to_string();
static mut PADDED_STRING: &str = "Shutdown: Message ";

//Search client by ID
/*
fn find_client_by_id(clients: &[ClientData], target_id: &str) -> Option<&ClientData> {
    for client in clients {
        if client.id == target_id {
            return Some(client);
        }
    }
    None // Return None if no matching client is found
}
*/
fn find_client_by_id(clients: &Arc<Mutex<Vec<ClientData>>>, target_id: &str) -> bool {
    // Acquire the lock on the Mutex
    let clients = clients.lock().unwrap();

    // Iterate over the clients and check for a matching ID
    for client in clients.iter() {
        if client.id == target_id {
            return true;
        }
    }

    // Return false if no matching client is found
    false
}

// Function to handle client connections
fn handle_client(mut stream: TcpStream,config: Config ,clients: Arc<Mutex<Vec<ClientData>>>) {
    let mut buffer = [0; 512];
    let client_ip = stream.peer_addr().unwrap().to_string();
    println!("Thread spawned for client: {}", client_ip);

    // Ask the client to send the password
    if let Err(e) = stream.write(b"Please send the SENHA:\n") {
        eprintln!("Failed to write to socket: {}", e);
        return;
    }

    // Read the password from the client
    let n = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to read from socket: {}", e);
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
            eprintln!("Failed to write to socket: {}", e);
            return;
        }
    } else {
        if let Err(e) = stream.write(b"Invalid password. Closing connection.\n") {
            eprintln!("Failed to write to socket: {}", e);
        }
        println!("Client {} provided an incorrect password.", client_ip);
        return; // Close the connection
    }

    // Read the client's ID
    let n = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to read from socket: {}", e);
            return;
        }
    };
    let client_id = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

    if find_client_by_id(&clients, &client_id){
        println!("Already has a client with this ID {}",client_id);
        if let Err(e) = stream.write(b"You are now DESconnected.\n") {
            eprintln!("Failed to write to socket: {}", e);
        }
        stream.shutdown(Shutdown::Both).unwrap();
        return;
    } 

    // Save the client's data to the list
    let client_data = ClientData {
        id: client_id.clone(),
        ip: client_ip.clone(),
        status: String::from("active"), // Set status to "active"
    };
    clients.lock().unwrap().push(client_data);

    println!("Client connected - ID: {}, IP: {}", client_id, client_ip);


    if let Err(e) = stream.write(b"Thank you! You are now connected.\n") {
        eprintln!("Failed to write to socket: {}", e);
        return;
    }
// Main loop to handle client requests
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                // Connection was closed by the client
                println!("Client disconnected: {}", client_ip);
                // Update the client's status to "inactive"
                let mut clients = clients.lock().unwrap();
                if let Some(client) = clients.iter_mut().find(|c| c.ip == client_ip) {
                    client.status = String::from("inactive");
                }                
                break;
            }
            Ok(n) => {
                // Convert the received data to a string
                let received_data = String::from_utf8_lossy(&buffer[..n]);
                println!("Received from {} ({}): {}", client_id, client_ip, received_data);

                // Handle the LISTAR command
                if received_data.trim() == "LISTAR" {
                    let clients_list = list_connected_clients(&clients);
                   
                    if let Err(e) = stream.write(clients_list.as_bytes()) {
                        eprintln!("Failed to write to socket: {}", e);
                        break;
                    }
                    continue;
                }                
                // Check if the received data contains the "FECHAR" command
                if received_data.trim() == "FECHAR" {
                    println!("Closing connection with client: {} ({})", client_id, client_ip);
                    break; // Exit the loop to close the connection
                }

                // Echo the data back to the client
                if let Err(e) = stream.write(&buffer[..n]) {
                    eprintln!("Failed to write to socket: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read from socket: {}", e);
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
                println!("New connection from: {:?} on port 1", stream.peer_addr());
                // Spawn a new thread to handle the connection
                let clients = Arc::clone(&clients);
                let config: Config =config.clone();
                thread::spawn(move || {
                    handle_client(stream,config,clients);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }

}

fn handle_client_for_port2(mut stream: TcpStream) -> ! {
    let client_ip = stream.peer_addr().unwrap().to_string();
    println!("Thread spawned for client: {}", client_ip);
    // Main loop to handle client
    loop {
        
        unsafe{         
            let resultado =  format!("{} {} {}",MSGCODE, PADDED_STRING, COUNTER);   
            stream.write( resultado.as_bytes()  ).unwrap();
        }
    thread::sleep(Duration::from_secs(10));
    }
}

fn handle_port2( listener: TcpListener){
    // Accept connections in a loop
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection from: {:?} in port 2", stream.peer_addr());
                // Spawn a new thread to handle the connection

                thread::spawn(move || {
                    handle_client_for_port2(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
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
        result.push_str(&format!("ID: {}, IP: {}, Status: {}\n", client.id, client.ip, client.status));
    }
    result
}

pub fn servidor() -> std::io::Result<()> {

    config::helper();
    let conf: Config = config::get_configuration();

    //println!("passwd:   {}", config::get_password(&conf));
    //println!("Host ip:  {}", config::get_hostip(&conf));
    //println!("port  1:  {}", config::get_port1(&conf));
    //println!("port  2:  {}", config::get_port2(&conf));

    let hostip_port1: String = config::get_hostip(&conf)+":"+&config::get_port1(&conf) ;
    println!("Server listening on port {}",hostip_port1);
    let listener1 = TcpListener::bind(hostip_port1)?;

    let hostip_port2: String = config::get_hostip(&conf)+":"+&config::get_port2(&conf) ;
    println!("Server listening on port {}",hostip_port2);
    let listener2 = TcpListener::bind(hostip_port2)?;

    // Shared list of clients (thread-safe)

    let clients = Arc::new(Mutex::new(Vec::<ClientData>::new()));
    thread::spawn(move || {
        handle_port1(listener1,conf.clone(), clients);
    });

    thread::spawn(move || {
        handle_port2(listener2);
    });

    println!("Servidor operacional!");
    loop {
        unsafe{
            COUNTER+=1;
        }
        thread::sleep(Duration::from_secs(11));
    }
}

