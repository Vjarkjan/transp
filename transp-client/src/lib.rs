// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use postcard::from_bytes;
//use std::error::Error;
use url::Url;
//slint::include_modules!();

use wasm_bindgen_futures::spawn_local;

use async_wsocket::message::Message as WMess;
use async_wsocket::{ConnectionMode, WebSocket};
use futures_util::{SinkExt, StreamExt};
use transp_common::user as tpu;
use web_sys::console;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn start_client() {
    spawn_local(async {
        console_error_panic_hook::set_once();
        let url = Url::parse("ws://127.0.0.1:8089/ws").unwrap();

        let (mut send, mut rec) = WebSocket::connect(&url, &ConnectionMode::direct())
            .await
            .unwrap()
            .split();
        send.send(tpu::bin_logreq_tg(12, 121010)).await.unwrap();
        let user_session = rec.next().await.unwrap();
        match user_session {
            Ok(user_session_msg) => match user_session_msg {
                WMess::Binary(b) => {
                    let user_session_msg: tpu::Message = from_bytes(&b).unwrap();
                    match user_session_msg {
                        tpu::Message::LoginResponse(b) => {
                            let user_session: tpu::UserSession = from_bytes(&b).unwrap();
                            console::log_1(&format!("the user session {user_session:?}").into());
                        }
                        _ => {}
                    }
                }
                _ => {
                    web_sys::console::log_1(&format!("the server is doing WEIRD STUFF").into());
                }
            },
            _ => {
                web_sys::console::log_1(&format!("El server Otra Vez VALIO KAKA").into());
            }
        }
    });

    // let ui = AppWindow::new()?;

    // ui.on_request_increase_value({
    //     let ui_handle = ui.as_weak();
    //     move || {
    //         let ui = ui_handle.unwrap();
    //         ui.set_counter(ui.get_counter() + 1);
    //     }
    // });

    // ui.run()?;

    // Ok(())
}
