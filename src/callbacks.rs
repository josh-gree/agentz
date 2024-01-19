use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::{conversation::Conversation, messages::Msg};

pub struct Response<RoleType> {
    pub to: RoleType,
    pub msg: String,
}

// Define a type alias for a boxed Future that returns an Option<Response<RoleType>>.
// Pin<Box<dyn Future<Output = Option<Response<RoleType>>> + Send + 'static>> is a common way
// to represent a Future that can be stored and moved around.
pub type CallbackFuture<'a, RoleType> =
    Pin<Box<dyn Future<Output = Option<Response<RoleType>>> + Send + 'a>>;

// Update the Callback type to return the CallbackFuture.
pub type Callback<RoleType> =
    fn(RoleType, Conversation<RoleType>, String) -> CallbackFuture<'static, RoleType>;

pub type Callbacks<RoleType> = HashMap<(usize, RoleType), Callback<RoleType>>;
