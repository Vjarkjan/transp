use axum::{
    Router,
    body::Bytes,
    extract::{
        State,
        ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::any,
};
use axum_extra::{TypedHeader, headers};
use rand::prelude::*;

use postcard::{from_bytes, to_stdvec};
use serde::{Serialize, de::SeqAccess};
use std::{collections::HashMap, fmt::Debug};
use std::{net::SocketAddr, path::PathBuf};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use transp_common::user::{self as tpu, LoginRequest, UserSession};
// Allows extracting the IP of the connecting user
use axum::extract::connect_info::ConnectInfo;
// Allows splitting the websocket stream into separate TX and RX branches
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};

use tpu::Message as tm;
use tpu::ServerError as se;

use tpu::{bin_err, bin_msg, bin_session};

#[derive(Clone, Copy, Debug)]
enum ErrorsWS {
    Sending,
    Receiving,
}

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
struct SessionDatabase {
    pub active_sessions: Arc<Mutex<std::collections::HashMap<u64, u64>>>,
    pub userdatabase: Arc<Mutex<std::collections::HashMap<u64, tpu::UserInternal>>>,
}

#[tokio::main]
async fn main() {
    let mut userbase = HashMap::new();
    userbase.insert(
        10,
        tpu::UserInternal::new(
            10,
            "Rogelion el Cagon".into(),
            tpu::UserCapabilities::Driver,
            101010,
        ),
    );
    userbase.insert(
        11,
        tpu::UserInternal::new(
            11,
            "Carmon el Mamon".into(),
            tpu::UserCapabilities::Driver,
            111010,
        ),
    );
    userbase.insert(
        1,
        tpu::UserInternal::new(
            1,
            "Ramon el Cabron".into(),
            tpu::UserCapabilities::Driver,
            11010,
        ),
    );
    userbase.insert(
        12,
        tpu::UserInternal::new(
            12,
            "Pedron el Puton".into(),
            tpu::UserCapabilities::Driver,
            121010,
        ),
    );

    let mut state = SessionDatabase {
        active_sessions: Arc::new(Mutex::new(HashMap::new())),
        userdatabase: Arc::new(Mutex::new(userbase)),
    };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().include_headers(true)),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8089")
        .await
        .unwrap();

    tracing::debug!("listening on : {}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<SessionDatabase>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(ug)) = user_agent {
        ug.to_string()
    } else {
        "Unknown Browser".to_string()
    };

    tracing::info!("Got connection from {addr} using {user_agent}");

    tracing::warn!("Upgrading websocket connection");

    let _ = ws.on_upgrade(move |socket| handle_socket(socket, addr.clone(), state));
}

use std::any::type_name_of_val;
async fn handle_socket(mut socket: WebSocket, who: SocketAddr, state: SessionDatabase) {
    let mut rng: StdRng = rand::make_rng();
    let session_token = rng.random::<u64>();

    let (mut sender, mut receiver) = socket.split();

    tokio::spawn(receiver_task(
        sender,
        receiver,
        who.clone(),
        session_token,
        state.clone(),
    ));
}

async fn send_ws(
    ws_send: &mut SplitSink<WebSocket, Message>,
    msg: Message,
    who: &SocketAddr,
) -> bool {
    match msg {
        Message::Close(_) => {
            ws_send
                .send(msg)
                .await
                .expect("couldn't say bye bye to {who}");
            false
        }
        _ => {
            if let Err(e) = ws_send.send(msg).await {
                tracing::error!("closing connection with {who} due to {e}");
                false
            } else {
                true
            }
        }
    }
}

enum MessageError {
    NotLogin,
    CouldNotDecode,
}

fn decode_login(bytes: Bytes) -> Result<tpu::LoginRequest, MessageError> {
    match from_bytes(&bytes[..]) {
        Ok(tm::Login(b)) => {
            if let Ok(l) = from_bytes(&b[..]) {
                return Ok(l);
            } else {
                return Err(MessageError::CouldNotDecode);
            }
        }
        Err(_) => return Err(MessageError::CouldNotDecode),
        _ => return Err(MessageError::NotLogin),
    }
}

async fn receiver_task(
    mut ws_send: SplitSink<WebSocket, Message>,
    mut ws_rec: SplitStream<WebSocket>,
    who: SocketAddr,
    token: u64,
    state: SessionDatabase,
) {
    let current_session: UserSession;
    'login: loop {
        //isn't managing WS SENDING ERRORS!!
        if let Some(Ok(m)) = ws_rec.next().await {
            if let Message::Binary(b) = m {
                let msg_val: Result<tpu::LoginRequest, MessageError> = decode_login(b);

                if let Ok(logreq) = msg_val {
                    let usrdb = state.userdatabase.lock().await;

                    match usrdb.get(&logreq.get_user()) {
                        Some(user) => {
                            if user.pass == logreq.get_pass() {
                                let mut sessions = state.active_sessions.lock().await;
                                //add only one session per user
                                sessions.insert(user.user, token);

                                current_session = tpu::UserSession::new(
                                    token,
                                    logreq.get_user(),
                                    user.capabilites,
                                    user.name.clone(),
                                );
                                if !send_ws(&mut ws_send, bin_session(&current_session), &who).await
                                {
                                    return;
                                }
                                break 'login;
                            } else {
                                if !send_ws(&mut ws_send, bin_err(se::BadLogin), &who).await {
                                    return;
                                }
                                continue 'login;
                            }
                        }
                        None => {
                            if !send_ws(&mut ws_send, bin_err(se::NoUser), &who).await {
                                return;
                            }
                            continue 'login;
                        }
                    }
                } else {
                    if !send_ws(&mut ws_send, bin_err(se::FirstLogin), &who).await {
                        return;
                    }
                }
            }
        } else {
            return;
        }
    }
    match current_session.role {
        tpu::UserCapabilities::Admin => {
            manage_admin(current_session, ws_send, ws_rec, state, token).await;
        }
        tpu::UserCapabilities::Driver => {
            manage_driver(current_session, ws_send, ws_rec, state, token).await;
        }
        tpu::UserCapabilities::Monitor => {
            manage_monitor(current_session, ws_send, ws_rec, state, token).await;
        }
    }
}

async fn manage_admin(
    cs: tpu::UserSession,
    mut ws_send: SplitSink<WebSocket, Message>,
    mut ws_rec: SplitStream<WebSocket>,
    state: SessionDatabase,
    token: u64,
) {
    todo!()
}

async fn manage_driver(
    cs: tpu::UserSession,
    mut ws_send: SplitSink<WebSocket, Message>,
    mut ws_rec: SplitStream<WebSocket>,
    state: SessionDatabase,
    token: u64,
) {
    todo!()
}

async fn manage_monitor(
    cs: tpu::UserSession,
    mut ws_send: SplitSink<WebSocket, Message>,
    mut ws_rec: SplitStream<WebSocket>,
    state: SessionDatabase,
    token: u64,
) {
    todo!()
}
