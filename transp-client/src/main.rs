// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use postcard::from_bytes;
//use std::error::Error;
use url::Url;
//slint::include_modules!();

use wasm_bindgen_futures::spawn_local;

use ewebsock::Options;
use ewebsock::WsMessage as EMess;
use ewebsock::connect;

use transp_common::user as tpu;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn main() {
    spawn_local(async {
        let url = Url::parse("ws://127.0.0.1:8089/ws").unwrap();
        let (mut send, mut rec) = connect(url, Options::default()).unwrap();
        if let Err(e) = send.send(tpu::user::bin_logreq_tg(12, 121010)).await {
            web_sys::console::log_1(&format!("jajan't el servidor SE MURIOOO {e}").into());
            return;
        }
        let user_session = rec.next().await.unwrap();
        match user_session {
            Ok(user_session_msg) => match user_session_msg {
                Message::Binary(b) => {
                    let user_session_msg: tpu::user::Message = from_bytes(&b).unwrap();
                    match user_session_msg {
                        tpu::user::Message::LoginResponse(b) => {
                            let user_session: tpu::user::UserSession = from_bytes(&b).unwrap();
                            web_sys::console::log_1(
                                &format!("the user session {user_session:?}").into(),
                            );
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
