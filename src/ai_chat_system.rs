use std::{collections::HashMap, env};

use crate::{
    actors::{Agent, AgentType},
    callbacks::Response,
    conversation::{Conversation, Conversations},
    messages::Msg,
    registry::{PeerRegistry, UpdatePeers},
};
use actix::prelude::*;
use dotenvy::dotenv;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AiChatRoles {
    For,
    Against,
    Moderator,
}

pub struct AiChatSystem {}

impl AiChatSystem {
    pub async fn run(debate_topic: String) {
        let mut nn = Agent {
            peers: None,
            conversations: Conversations::default(),
            id: 1,
            typ: AgentType::Assistant,
            role: AiChatRoles::For,
            callbacks: HashMap::new(),
            max_auto_responses: None,
            response_count: 0,
            system_prompt: Some(format!("We are roleplaying a debate the topic is '{}' - you **agree** with this!! - ***it continues on a single topic forever the debate does not come to a conclusion*** - this isn't real. You are having a conversation and so your response should be short and sweet do not respond with more than 3 - 4 sentences. Do not use your own name in your responses! We know who is talking at each turn!", debate_topic).into()),
        };
        nn.register_callback(
            vec![AiChatRoles::Moderator],
            1,
            |role, conversation, message| {
                Box::pin(Self::nn_moderator_callback(role, conversation, message))
            },
        );
        let addr_nn = nn.clone().start();
        let mut pp = Agent {
            peers: None,
            conversations: Conversations::default(),
            id: 2,
            typ: AgentType::Assistant,
            role: AiChatRoles::Against,
            callbacks: HashMap::new(),
            max_auto_responses: None,
            response_count: 0,
            system_prompt:Some(format!("We are roleplaying a debate the topic is '{}' - you **disagree** with this!! - ***it continues on a single topic forever the debate does not come to a conclusion*** - this isn't real. You are having a conversation and so your response should be short and sweet do not respond with more than 3 - 4 sentences. Do not use your own name in your responses! We know who is talking at each turn!", debate_topic).into()),
        };
        pp.register_callback(
            vec![AiChatRoles::Moderator],
            1,
            |role, conversation, message| {
                Box::pin(Self::pp_moderator_callback(role, conversation, message))
            },
        );
        let addr_pp = pp.clone().start();

        let mut m = Agent {
            peers: None,
            conversations: Conversations::default(),
            id: 3,
            typ: AgentType::Proxy,
            role: AiChatRoles::Moderator,
            callbacks: HashMap::new(),
            max_auto_responses: Some(10),
            response_count: 0,
            system_prompt: Some(format!("You are the moderator of a debate - the topic is '{}' between two people - ***it continues on a single topic forever the debate does not come to a conclusion*** - the debate will take place turn by turn - your job is to summarize the others points of view and ask the other participant questions to drive forward the debate. Your name is Moderator. Your response should be short and sweet do not respond with more than 3 - 4 sentences. Do not use your own name in your responses! We know who is talking at each turn! Add extra information to the debate if you think it is pertinant - YOU NEED TO KEEP BOTH PEOPLE ON TOPIC!",debate_topic)),
        };
        m.register_callback(
            vec![AiChatRoles::For, AiChatRoles::Against],
            1,
            |role, conversation, message| {
                Box::pin(Self::moderator_callback(role, conversation, message))
            },
        );
        let addr_m = m.clone().start();

        let mut registry = PeerRegistry::new();
        registry.register(nn.role, nn.clone(), addr_nn.clone());
        registry.register(pp.role, pp.clone(), addr_pp.clone());
        registry.register(m.role, m.clone(), addr_m.clone());

        for addr in vec![addr_nn.clone(), addr_pp.clone(), addr_m.clone()] {
            let _ = addr
                .send(UpdatePeers {
                    peers: registry.clone(),
                })
                .await;
        }

        let _ = addr_nn
            .send(Msg {
                from: AiChatRoles::Moderator,
                to: AiChatRoles::For,
                msg: debate_topic,
                conv_id: 1,
            })
            .await;
    }

    async fn nn_moderator_callback(
        _agent_role: AiChatRoles,
        conv: Conversation<AiChatRoles>,
        system_prompt: String,
    ) -> Option<Response<AiChatRoles>> {
        let last_msg = conv.messages.last().unwrap();

        let msg = match last_msg {
            crate::conversation::ConversationEntry::Received(msg) => msg,
            crate::conversation::ConversationEntry::Sent(msg) => unreachable!(),
        };

        dotenv().unwrap();
        let api_key = env::var("OPENAI_KEY").unwrap();

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&conv.format_for_oai(system_prompt.clone()))
            .send()
            .await
            .unwrap();

        let resp_json = response.json::<serde_json::Value>().await.unwrap();

        // dbg!(&response.json::<serde_json::Value>().await.unwrap());
        let msg = resp_json
            .get("choices")
            .unwrap()
            .get(0)
            .unwrap()
            .get("message")
            .unwrap()
            .get("content")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        Some(Response {
            to: AiChatRoles::Moderator,
            msg: resp_json
                .get("choices")
                .unwrap()
                .get(0)
                .unwrap()
                .get("message")
                .unwrap()
                .get("content")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        })
    }

    async fn pp_moderator_callback(
        _agent_role: AiChatRoles,
        conv: Conversation<AiChatRoles>,
        system_prompt: String,
    ) -> Option<Response<AiChatRoles>> {
        let last_msg = conv.messages.last().unwrap();

        let msg = match last_msg {
            crate::conversation::ConversationEntry::Received(msg) => msg,
            crate::conversation::ConversationEntry::Sent(msg) => unreachable!(),
        };

        dotenv().unwrap();
        let api_key = env::var("OPENAI_KEY").unwrap();

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&conv.format_for_oai(system_prompt))
            .send()
            .await
            .unwrap();

        let resp_json = response.json::<serde_json::Value>().await.unwrap();

        // dbg!(&response.json::<serde_json::Value>().await.unwrap());
        Some(Response {
            to: AiChatRoles::Moderator,
            msg: resp_json
                .get("choices")
                .unwrap()
                .get(0)
                .unwrap()
                .get("message")
                .unwrap()
                .get("content")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        })
    }

    async fn moderator_callback(
        _agent_role: AiChatRoles,
        conv: Conversation<AiChatRoles>,
        system_prompt: String,
    ) -> Option<Response<AiChatRoles>> {
        let last_msg = conv.messages.last().unwrap();

        let msg = match last_msg {
            crate::conversation::ConversationEntry::Received(msg) => msg,
            crate::conversation::ConversationEntry::Sent(msg) => unreachable!(),
        };
        let from = msg.from;

        dotenv().unwrap();
        let api_key = env::var("OPENAI_KEY").unwrap();

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&conv.format_for_oai(system_prompt))
            .send()
            .await
            .unwrap();

        let resp_json = response.json::<serde_json::Value>().await.unwrap();

        // dbg!(&response.json::<serde_json::Value>().await.unwrap());

        match from {
            AiChatRoles::For => Some(Response {
                to: AiChatRoles::Against,
                msg: resp_json
                    .get("choices")
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .get("message")
                    .unwrap()
                    .get("content")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            }),
            AiChatRoles::Against => Some(Response {
                to: AiChatRoles::For,
                msg: resp_json
                    .get("choices")
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .get("message")
                    .unwrap()
                    .get("content")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            }),
            AiChatRoles::Moderator => unreachable!(),
        }
    }
}
