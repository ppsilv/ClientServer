
use std::io::{self, Write, Read};
use std::thread;
use std::time::Duration;
use std::net::{TcpStream, Shutdown};

fn main() -> io::Result<()> {
    // Connect to the server
    let mut stream = TcpStream::connect("127.0.0.1:1111")?;
    println!("Connected to server at 127.0.0.1:1111");

    let mut buffer = [0; 512];

    // Read the server's prompt for the password
    let n = stream.read(&mut buffer)?;
    let prompt = String::from_utf8_lossy(&buffer[..n]);
    print!("{}", prompt);

    // If the password was incorrect, exit
    if prompt.contains("Please send the SENHA") {
        // Send the password to the server
        let mut password = "1234";
        //io::stdin().read_line(&mut password)?;
        stream.write_all(password.trim().as_bytes())?;   
        println!("Password sent...");     
    }

    // Read the server's response to the password
    let n = stream.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..n]);
    print!("{}", response);

    // If the password was incorrect, exit
    if response.contains("Invalid password") {
        return Ok(());
    }
    if response.contains("Password correct. Please send your ID") {
        // Send the password to the server
        let mut ID = "1234";
        //io::stdin().read_line(&mut password)?;
        stream.write_all(ID.trim().as_bytes())?;   
        println!("ID sent...");     
    }

    // Read the server's response
    let n = stream.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..n]);
    print!("Server response: {}", response);

    stream.shutdown(Shutdown::Both)?;

    if response.contains("You are now connected") {
        println!("The server autorized my connection");
    }else{
        return Ok(());
    }

    let mut stream = TcpStream::connect("127.0.0.1:2222")?;
    println!("Connected to server at 127.0.0.1:2222");

    loop{
        let n = stream.read(&mut buffer)?;
        let response = String::from_utf8_lossy(&buffer[..n]);
        if response.contains("100 Shutdown:") {
            println!("100 Shutdown: received");
        }
    }    
    Ok(())
}