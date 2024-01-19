use std::collections::HashMap;
use std::fmt::Debug;

use actix::prelude::*;

use crate::callbacks::{self, Callback, Callbacks, Response};
use crate::conversation::{Conversation, ConversationEntry, Conversations};
use crate::messages::Msg;
use crate::registry::PeerRegistry;

#[derive(Debug, Clone, Copy)]
pub enum AgentType {
    Proxy,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct Agent<RoleType: std::marker::Unpin + 'static + Eq + PartialEq + Clone + Debug> {
    pub peers: Option<PeerRegistry<RoleType>>,
    pub conversations: Conversations<RoleType>,
    pub id: usize,
    pub typ: AgentType,
    pub role: RoleType,
    pub callbacks: Callbacks<RoleType>,
    pub max_auto_responses: Option<usize>,
    pub response_count: usize,
    pub system_prompt: Option<String>,
}

impl<RoleType: std::marker::Unpin + 'static + Eq + PartialEq + std::hash::Hash + Clone + Debug>
    Agent<RoleType>
{
    pub fn register_callback(
        &mut self,
        roles: Vec<RoleType>,
        priority: usize,
        callback: Callback<RoleType>,
    ) {
        for role in roles {
            self.callbacks.insert((priority, role), callback);
        }
    }
}

impl<RoleType: std::marker::Unpin + 'static + Eq + PartialEq + Clone + Debug> Actor
    for Agent<RoleType>
{
    type Context = Context<Self>;
}

impl<
        RoleType: std::marker::Unpin
            + 'static
            + Eq
            + PartialEq
            + std::hash::Hash
            + Clone
            + std::marker::Send
            + Copy
            + Debug,
    > Handler<Msg<RoleType>> for Agent<RoleType>
{
    type Result = ResponseActFuture<Self, Result<(), ()>>;

    fn handle(&mut self, msg: Msg<RoleType>, _ctx: &mut Self::Context) -> Self::Result {
        println!("\n\n{}\n\n", msg.msg);
        let conv = self.conversations.get_or_create_conversation(msg.conv_id);
        conv.add_message(ConversationEntry::Received(msg.clone()));

        let from_role = msg.from;
        let _my_role = self.role;

        let binding = self.callbacks.clone();

        let role = self.role.clone();
        let system_prompt = self.system_prompt.clone().unwrap_or_default();
        let conv = conv.clone();

        Box::pin(
            async move {
                let mut callbacks: Vec<_> = binding
                    .iter()
                    .filter(|((_, role), _)| *role == from_role)
                    .collect();

                callbacks.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));
                let mut resp: Option<Response<RoleType>> = None;
                let mut out_msg: Option<Msg<RoleType>> = None;

                for callback in callbacks {
                    resp = callback.1(role, conv.clone(), system_prompt.clone()).await;

                    if let Some(resp) = resp {
                        out_msg = Some(Msg {
                            conv_id: msg.conv_id,
                            from: role,
                            to: resp.to,
                            msg: resp.msg,
                        });
                    }
                }
                if let Some(out_msg) = out_msg {
                    out_msg
                } else {
                    panic!()
                }
            }
            .into_actor(self)
            .map(|res, act, _ctx| {
                let (_, addr) = act.peers.as_ref().unwrap().get(res.to).unwrap();
                addr.do_send(res.clone());
                act.response_count += 1;

                let conv = act.conversations.get_or_create_conversation(res.conv_id);
                conv.add_message(ConversationEntry::Sent(res.clone()));

                if let Some(max) = act.max_auto_responses {
                    if act.response_count >= max {
                        System::current().stop();
                    }
                }
                Ok(())
            }),
        )
    }
}
