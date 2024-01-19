use std::{collections::HashMap, fmt::Debug};

use crate::actors::Agent;
use actix::prelude::*;

#[derive(Debug, Clone)]
pub struct PeerRegistry<RoleType: std::marker::Unpin + 'static + Eq + PartialEq + Clone + Debug> {
    peers: HashMap<RoleType, (Agent<RoleType>, Addr<Agent<RoleType>>)>,
}

impl<RoleType: std::marker::Unpin + 'static + Eq + PartialEq + std::hash::Hash + Clone + Debug>
    PeerRegistry<RoleType>
{
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }
    pub fn register(
        &mut self,
        role: RoleType,
        agent: Agent<RoleType>,
        addr: Addr<Agent<RoleType>>,
    ) {
        self.peers.insert(role, (agent, addr));
    }
    pub fn get(&self, role: RoleType) -> Option<&(Agent<RoleType>, Addr<Agent<RoleType>>)> {
        self.peers.get(&role)
    }
}

impl<RoleType: std::marker::Unpin + 'static + Eq + PartialEq + std::hash::Hash + Clone + Debug>
    Default for PeerRegistry<RoleType>
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdatePeers<RoleType: std::marker::Unpin + 'static + Eq + PartialEq + Clone + Debug> {
    pub peers: PeerRegistry<RoleType>,
}

impl<RoleType: std::marker::Unpin + 'static + Eq + PartialEq + Clone + Debug>
    Handler<UpdatePeers<RoleType>> for Agent<RoleType>
{
    type Result = ();

    fn handle(&mut self, msg: UpdatePeers<RoleType>, _ctx: &mut Self::Context) -> Self::Result {
        self.peers = Some(msg.peers);
    }
}
