use std::env;


pub mod servidor;
pub mod cliente;

/*
   how to call module 
   servidor::servidor().unwrap();
*/ 
fn main() -> std::io::Result<()>{
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <name> [server|client]", args[0]);
        std::process::exit(1);
    }

    let name = &args[1];
    
    println!("Arg 1..: {}",name);
    
    if name.contains("server") {
        servidor::servidor().unwrap();
    }else{
        cliente::cliente().unwrap();
    }

    Ok(())  
}