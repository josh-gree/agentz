use actix::prelude::*;

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<(), ()>")]
pub struct Msg<RoleType: Clone> {
    pub from: RoleType,
    pub to: RoleType,
    pub msg: String,
    pub conv_id: usize,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct SeedMsg {
    pub msg: String,
}
