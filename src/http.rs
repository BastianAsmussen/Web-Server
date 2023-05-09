use std::fmt;

pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

pub struct Request {
    method: Method,
    path: String,
    version: String,
    headers: Vec<String>,
    body: String,
}

impl Request {
    pub fn new(request: &str) -> Request {
        let mut headers = Vec::new();
        
        // Split the request into lines.
        let lines = request.split("\r\n");
        
        // Iterate over the lines.
        for line in lines {
            // Check if the line is empty.
            if line.is_empty() {
                break;
            }
            
            // Add the line to the headers vector.
            headers.push(line.to_string());
        }
        
        // Split the first line into words.
        let words = headers[0].split(' ');
        
        // Create a new request instance.
        Request {
            method: match words.clone().next().unwrap() {
                "GET" => Method::Get,
                "POST" => Method::Post,
                "PUT" => Method::Put,
                "DELETE" => Method::Delete,
                _ => panic!("Invalid method: {}", words.clone().next().unwrap()),
            },
            path: words.clone().nth(1).unwrap().to_string(),
            version: words.clone().nth(2).unwrap().to_string(),
            headers,
            body: "".to_string(),
        }
    }
    
    pub fn get_method(&self) -> &Method {
        &self.method
    }
    
    pub fn get_path(&self) -> &str {
        &self.path
    }
    
    pub fn get_version(&self) -> &str {
        &self.version
    }
    
    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
    
    pub fn get_body(&self) -> &str {
        &self.body
    }
}

pub struct Response {
    version: String,
    status_code: u16,
    status_message: String,
    headers: Vec<String>,
    body: String,
}

impl Response {
    pub fn new(version: &str, status_code: u16, status_message: &str) -> Response {
        // Create a new response instance.
        Response {
            version: version.to_string(),
            status_code,
            status_message: status_message.to_string(),
            headers: Vec::new(),
            body: "".to_string(),
        }
    }
    
    pub fn get_version(&self) -> &str {
        &self.version
    }
    
    pub fn get_status_code(&self) -> u16 {
        self.status_code
    }
    
    pub fn get_status_message(&self) -> &str {
        &self.status_message
    }
    
    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
    
    pub fn get_body(&self) -> &str {
        &self.body
    }
    
    pub fn set_status_code(&mut self, status_code: u16) {
        self.status_code = status_code;
    }
    
    pub fn set_status_message(&mut self, status_message: &str) {
        self.status_message = status_message.to_string();
    }
    
    pub fn set_body(&mut self, body: &str) {
        self.body = body.to_string();
    }
    
    pub fn add_header(&mut self, header: &str) {
        self.headers.push(header.to_string());
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut response = format!("HTTP/{} {} {}\r\n", self.version, self.status_code, self.status_message);
        
        for header in &self.headers {
            response += &format!("{}\r\n", header);
        }
        
        response += "\r\n";
        response += &self.body;
        
        response.fmt(f)
    }
}
