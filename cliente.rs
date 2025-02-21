use std::net::TcpStream;
use std::io::{self, Write, Read};
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    // Connect to the server
    let mut stream = TcpStream::connect("127.0.0.1:1111")?;
    println!("Connected to server at 127.0.0.1:1111");

    let mut buffer = [0; 512];

    // Read the server's prompt for the password
    let n = stream.read(&mut buffer)?;
    let prompt = String::from_utf8_lossy(&buffer[..n]);
    print!("{}", prompt);

    // Send the password to the server
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    stream.write_all(password.trim().as_bytes())?;

    // Read the server's response to the password
    let n = stream.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..n]);
    println!("{}", response);

    // If the password was incorrect, exit
    if response.contains("Invalid password") {
        return Ok(());
    }

    // Main loop to interact with the server
    loop {
        // Read input from the user
        let mut input = String::new();
        print!("Enter a message (or type FECHAR to disconnect): ");
        io::stdout().flush()?; // Ensure the prompt is displayed
        io::stdin().read_line(&mut input)?;

        // Send the input to the server
        stream.write_all(input.trim().as_bytes())?;

        // If the user typed "FECHAR", exit the loop
        if input.trim() == "FECHAR" {
            println!("Disconnecting from server...");
            break;
        }

        // Read the server's response
        let n = stream.read(&mut buffer)?;
        let response = String::from_utf8_lossy(&buffer[..n]);
        println!("Server response: {}", response);
    }
    let mut stream = TcpStream::connect("127.0.0.1:2222")?;
    println!("Connected to server at 127.0.0.1:2222");

    Ok(())
}