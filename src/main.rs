use std::fs;
use std::path::Path;

use crate::server::Server;

mod server;
mod http;

const CONFIG_PATH: &str = "config.json";

fn main() {
    // Check if the config.json file exists in current directory.
    if !Path::new(CONFIG_PATH).exists() {
        println!("Configuration file not found, creating a new one...");
        
        init_cfg();
    }
    
    // Read the config.json file.
    let config = fs::read_to_string(CONFIG_PATH).unwrap();
    
    // Parse the config.json file.
    let config = json::parse(&config).unwrap();
    
    // Create a new server instance.
    let server = Server::new(&config);
    
    // Print the server configuration.
    println!("================ CONFIG ================");
    println!("Verbose Output:\t{}", server.is_verbose());
    println!("Thread Count:\t{}", server.get_thread_count());
    println!("Port:\t\t\t{}", server.get_port());
    println!("Web Root:\t\t{}", server.get_web_root());
    println!("Page Count:\t\t{}", server.get_pages().len());
    println!("========================================");
    println!();
    
    // Start listening for incoming connections on the specified port.
    server.listen();
}

fn init_cfg() {
    // Create the config.json file.
    let default_config = json::parse(r#"
    {
      "thread_count": 1,
      "verbose": true,
      "port": 8080,
      "web_root": "web",
      "pages": [
        {
          "name": "Main Page",
          "path": "index.html"
        }
      ]
    }
    "#).unwrap();
    
    // Write the config.json file.
    fs::write(CONFIG_PATH, default_config.dump()).unwrap();
}
