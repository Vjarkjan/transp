use core::ops::Deref;
use postcard::to_stdvec;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum UserCapabilities {
    Admin,
    Driver,
    Monitor,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Status {
    Carga,
    Descarga,
    TransitoCarga,
    TransitoDescarga,
    Parada,
    TransitoPararador,
    Falla,
    Emergencia,
    FueraDeServicio,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Message {
    GetToken,
    Error(Vec<u8>),
    LoginResponse(Vec<u8>),
    Login(Vec<u8>),
    Logout(Vec<u8>),
    NewState(Vec<u8>),
    TokenResponse(Vec<u8>),
}

use axum::extract::ws::Message as AMess;

pub fn bin_msg<T: Serialize>(arg: T) -> AMess {
    axum::extract::ws::Message::binary(to_stdvec(&arg).unwrap())
}
pub fn bin_err<T: Serialize>(arg: T) -> AMess {
    bin_msg(Message::Error(to_stdvec(&arg).unwrap()))
}
pub fn bin_session(ses: &UserSession) -> AMess {
    bin_msg(Message::LoginResponse(to_stdvec(ses).unwrap()))
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct LoginRequest {
    user: u64,
    pass: u64,
}

impl LoginRequest {
    pub fn get_user(&self) -> u64 {
        self.user
    }
    pub fn get_pass(&self) -> u64 {
        self.user
    }
}

#[derive(Serialize, Clone, Copy, Deserialize, Debug, PartialEq)]
pub enum ServerError {
    BadLogin,
    BadMessageType,
    NoUser,
    BadPass,
    FirstLogin,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct UserInternal {
    pub user: u64,
    pub name: String,
    pub capabilites: UserCapabilities,
    pub pass: u64,
}

impl UserInternal {
    pub fn new(user: u64, name: String, capabilites: UserCapabilities, pass: u64) -> Self {
        Self {
            user,
            name,
            capabilites,
            pass,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct UserSession {
    pub user: u64,
    pub token: u64,
    pub role: UserCapabilities,
    pub name: String,
}

impl UserSession {
    pub fn new(token: u64, user: u64, role: UserCapabilities, name: String) -> Self {
        UserSession {
            user,
            token,
            role,
            name,
        }
    }
}
