use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use json::JsonValue;
use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::http::{Request, Response};

pub struct Server {
    verbose: bool,
    thread_count: u16,
    thread_pool: ThreadPool,
    port: u16,
    web_root: String,
    pages: Vec<Page>,
    config: JsonValue,
}

impl Server {
    pub(crate) fn new(config: &JsonValue) -> Server {
        
        // Load the config and return a new server instance.
        Self::load_cfg(config)
    }
    
    fn load_cfg(config: &JsonValue) -> Server {
        
        // Get the verbose flag.
        let verbose = config["verbose"].as_bool();
        
        // Check if the verbose flag is valid.
        if verbose.is_none() {
            panic!("Invalid verbose flag, must be a boolean!");
        }
        
        let verbose = verbose.unwrap();
        
        // Get the thread count.
        let thread_count = config["thread_count"].as_u16();
        
        // Check if the thread count is valid.
        if thread_count.is_none() || thread_count.unwrap() < 1 {
            panic!("Invalid thread count, must be a number greater than 0!");
        }
        
        let thread_count = thread_count.unwrap();
        
        // Create a new thread pool.
        let thread_pool = ThreadPoolBuilder::new().num_threads(thread_count as usize).build().unwrap();
        
        // Get the port number.
        let port = config["port"].as_u16();
        
        // Check if the port is valid.
        if port.is_none() || port.unwrap() < 1_024 || port.unwrap() == 65_535 {
            panic!("Invalid port, must be a number between 1.024 and 65.535!");
        }
        
        let port = port.unwrap();
        
        // If the web_root is not specified, use the default value.
        let web_root = config["web_root"].as_str();
        
        if web_root.is_none() {
            panic!("Invalid web_root, must be a string!");
        }
        
        let web_root = web_root.unwrap();
        
        // Check if the web_root directory exists.
        if !fs::metadata(web_root).is_ok() {
            // Create the web_root directory.
            match fs::create_dir(web_root) {
                Ok(_) => {
                    if verbose {
                        println!("Created web root directory: {}", web_root)
                    }
                }
                Err(_) => panic!("Failed to create web root directory: {}", web_root),
            }
        }
        
        // Get the pages array.
        let pages_from_file = config["pages"].members();
        
        // Make sure the pages array is not empty.
        if pages_from_file.len() == 0 {
            if verbose {
                println!("No pages found, creating an index.html file...");
            }
            
            // Create the file.
            let page = create_file(format!("{}/{}", web_root, "index.html"), verbose);
            
            // Return a new server instance.
            return Server {
                verbose,
                thread_count,
                thread_pool,
                port,
                web_root: web_root.to_string(),
                pages: vec!(page),
                config: config.clone(),
            };
        }
        
        let mut pages: Vec<Page> = Vec::new();
        
        // Iterate over the pages from the config file.
        for page in pages_from_file {
            
            // Get the page name.
            let name = page["name"].as_str();
            
            // Check if the page name is valid.
            if name.is_none() {
                panic!("Invalid page name, must be a string!");
            }
            
            let name = name.unwrap();
            
            // Get the page path.
            let path = page["path"].as_str();
            
            // Check if the page path is valid.
            if path.is_none() {
                panic!("Invalid page path, must be a string!");
            }
            
            let path = path.unwrap();
            
            // Make sure the file exists.
            if fs::metadata(format!("{}/{}", web_root, path)).is_err() {
                // Create the file.
                let page = create_file(format!("{}/{}", web_root, path), verbose);
                
                // Add the page to the pages vector.
                pages.push(page);
                
                continue;
            }
            
            // Get the page contents from the file.
            let contents = fs::read_to_string(format!("{}/{}", web_root, path));
            
            // Check if the page contents are valid.
            if contents.is_err() {
                panic!("Invalid page contents, must be a string!");
            }
            
            let contents = contents.unwrap();
            
            // Create a new page instance.
            let page = Page::new(name, path, &contents);
            
            // Add the page to the pages vector.
            pages.push(page);
        }
        
        // Return a new server instance.
        Server {
            verbose,
            thread_count,
            thread_pool,
            port,
            web_root: web_root.to_string(),
            pages,
            config: config.clone(),
        }
    }
    
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }
    
    pub fn get_thread_count(&self) -> u16 {
        self.thread_count
    }
    
    pub fn get_thread_pool(&self) -> &ThreadPool {
        &self.thread_pool
    }
    
    pub fn get_port(&self) -> u16 {
        self.port
    }
    
    pub fn get_web_root(&self) -> &str {
        &self.web_root
    }
    
    pub fn get_pages(&self) -> &Vec<Page> {
        &self.pages
    }
    
    pub fn get_config(&self) -> &JsonValue {
        &self.config
    }
    
    pub fn listen(&self) {
        if self.verbose {
            println!("Listening on port {}...", self.port);
        }
        
        // Create a new TcpListener instance on a random IP address.
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port));
        
        // Check if the listener is valid.
        if listener.is_err() {
            panic!("Failed to bind to port {}!", self.port);
        }
        
        let listener = listener.unwrap();
        
        // Accept incoming connections.
        for stream in listener.incoming() {
            // Check if the stream is valid.
            if stream.is_err() {
                panic!("Failed to accept incoming connection!");
            }
            
            let stream = stream.unwrap();
            
            // Use a thread from the thread pool to handle the connection.
            self.thread_pool.install(|| {
                self.handle_connection(stream);
            });
        }
    }
    
    fn handle_connection(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        
        // Read the request from the stream.
        let bytes_read = stream.read(&mut buffer);
        
        // Check if the bytes_read is valid.
        if bytes_read.is_err() {
            panic!("Failed to read from stream!");
        }
        
        let bytes_read = bytes_read.unwrap();
        
        // Convert the buffer to a string.
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        // Create a new Request instance.
        let request = Request::new(&request);
        
        // Find the page.
        let page = self.find_page(&request);
        
        // Send the response.
        let mut response = Response::new("1.1", 200, "OK");
        response.set_body(page.get_contents());
        
        // Write the response to the stream.
        stream
            .write_all(response.to_string().as_bytes())
            .expect("An error occurred while writing to the stream!");
        
        // Flush the stream.
        stream.flush().unwrap();
        
        if self.verbose {
            println!("Served request to {}!", stream.peer_addr().unwrap());
        }
    }
    
    fn find_page(&self, request: &Request) -> &Page {
        // Iterate over the pages.
        for page in &self.pages {
            // Check if the page name matches the request path.
            if page.get_name() == request.get_path() {
                return page;
            }
        }
        
        // Return the index page if no page was found.
        &self.pages[0]
    }
}

fn create_file(path: String, verbose: bool) -> Page {
    
    // Make sure all the directories exist before creating the file.
    if fs::metadata(path.replace(path.split('/').last().unwrap(), "")).is_err() {
        // Create the directories.
        match fs::create_dir_all(path.replace(path.split('/').last().unwrap(), "")) {
            Ok(_) => {
                if verbose {
                    println!("Created directories: {}", path.clone().replace(path.split('/').last().unwrap(), ""));
                }
            }
            Err(_) => panic!("Failed to create directories: {}", path.clone().replace(path.split('/').last().unwrap(), "")),
        }
    }
    
    // Create an empty file.
    match fs::write(&path, "") {
        Ok(_) => {
            if verbose {
                println!("Created file: {}", path);
            }
        }
        Err(_) => panic!("Failed to create file: {}", path),
    }
    
    let name = path.split('/').last().unwrap();
    
    // Return a new page instance.
    Page::new(name, &path, "")
}

pub struct Page {
    name: String,
    path: String,
    contents: String,
}

impl Page {
    fn new(name: &str, path: &str, contents: &str) -> Page {
        Page {
            name: name.to_string(),
            path: path.to_string(),
            contents: contents.to_string(),
        }
    }
    
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
    pub fn get_path(&self) -> &str {
        &self.path
    }
    
    pub fn get_contents(&self) -> &str {
        &self.contents
    }
}
