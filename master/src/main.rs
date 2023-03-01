pub mod config;
pub mod orders;

fn main() {
    println!("Master started");
    
    orders::main();

    loop {
        
    }
}
