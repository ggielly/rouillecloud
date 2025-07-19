//! WebSocket API for real-time updates

use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;
use actix::{Actor, StreamHandler, Message};
use serde::{Deserialize, Serialize};

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

pub struct WebSocketSession {
    // TODO: Add session state
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                // TODO: Handle incoming WebSocket messages
                ctx.text(format!("Echo: {}", text));
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse> {
    let resp = ws::start(WebSocketSession {}, &req, stream);
    resp
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/ws", web::get().to(websocket_handler));
}
