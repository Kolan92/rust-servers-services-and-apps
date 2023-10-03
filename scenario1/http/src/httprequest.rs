use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Resource {
    Path(String),
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: Method,
    pub version: Version,
    pub resource: Resource,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl From<&str> for HttpRequest {
    fn from(value: &str) -> Self {
        let mut lines = value.lines();

        let request = lines.next().unwrap();
        let (method, resource, version) = parse_request_line(request);

        let headers = lines
            .by_ref()
            .take_while(|line| line.len() != 0)
            .map(|header| parse_header(header))
            .collect();

        let body = lines.fold(String::new(), |a, b| a + b + "\r\n");

        HttpRequest {
            method,
            version,
            headers,
            resource,
            body,
        }
    }
}

fn parse_header(header: &str) -> (String, String) {
    let header: Vec<&str> = header.split(":").collect();
    let key = header.get(0).unwrap_or(&"").to_string();
    let value = header.get(1).unwrap_or(&"").to_string();
    (key, value)
}

fn parse_request_line(request: &str) -> (Method, Resource, Version) {
    let request: Vec<&str> = request.split(" ").collect();
    let method: Method = request[0].into();
    let resource = Resource::Path(request[1].into());
    let version: Version = request[2].into();

    (method, resource, version)
}

#[derive(Debug, PartialEq)]
pub enum Method {
    Get,
    Post,
    Uninitialized,
}

impl From<&str> for Method {
    fn from(value: &str) -> Self {
        match value {
            "GET" => Method::Get,
            "POST" => Method::Post,
            _ => Method::Uninitialized,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Version {
    V1_1,
    V2_0,
    Uninitialized,
}

impl From<&str> for Version {
    fn from(value: &str) -> Self {
        match value {
            "HTTP/1.1" => Version::V1_1,
            _ => Version::Uninitialized,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_into() {
        let m: Method = "GET".into();
        assert_eq!(m, Method::Get);
    }

    #[test]
    fn test_version_into() {
        let v: Version = "HTTP/1.1".into();
        assert_eq!(v, Version::V1_1);
    }

    #[test]
    fn test_read_http() {
        let s = "POST /greeting HTTP/1.1\r\nHost:localhost:3000\r\nUser-Agent:curl/7.64.1\r\nAccept:*/*\r\n\r\n{\r\n\"id\":1\r\n}\r\n";
        let mut headers_expected = HashMap::new();
        headers_expected.insert("Host".into(), "localhost".into());
        headers_expected.insert("Accept".into(), "*/*".into());
        headers_expected.insert("User-Agent".into(), "curl/7.64.1".into());
        let req: HttpRequest = s.into();
        assert_eq!(Method::Post, req.method);
        assert_eq!(Version::V1_1, req.version);
        assert_eq!(Resource::Path("/greeting".to_string()), req.resource);
        assert_eq!(headers_expected, req.headers);
        assert_eq!("{\r\n\"id\":1\r\n}\r\n", req.body);
    }
}
