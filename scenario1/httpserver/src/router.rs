use http::httprequest::{HttpRequest, Method, Resource};

use crate::handler::{Handler, PageNotFoundHandler, StaticPageHandler, WebServiceHandler};

pub struct Router {}
impl Router {
    pub fn route(req: HttpRequest, stream: &mut std::net::TcpStream) {
        match req.method {
            Method::Get => match &req.resource {
                Resource::Path(path) => {
                    let route: Vec<&str> = path.split("/").collect();

                    match route[1] {
                        "api" => WebServiceHandler::handle(&req).send_response(stream),
                        _ => StaticPageHandler::handle(&req).send_response(stream),
                    }
                }
            },
            _ => PageNotFoundHandler::handle(&req).send_response(stream),
        }
    }
}
