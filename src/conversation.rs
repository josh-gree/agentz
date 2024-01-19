use std::{collections::HashMap, fmt::Debug};

use serde_json::{json, Value};

use crate::messages::{self, Msg};

#[derive(Debug, Clone)]
pub struct Conversations<RoleType: Clone + Debug>(HashMap<usize, Conversation<RoleType>>);

#[derive(Debug, Clone)]
pub enum ConversationEntry<RoleType: Clone + Debug> {
    Sent(Msg<RoleType>),
    Received(Msg<RoleType>),
}

impl<RoleType: Clone + Debug> ConversationEntry<RoleType> {
    pub fn get_content(&self) -> String {
        match self {
            Self::Sent(msg) => msg.msg.clone(),
            Self::Received(msg) => msg.msg.clone(),
        }
    }

    pub fn get_from_role(&self) -> RoleType {
        match self {
            Self::Sent(msg) => msg.from.clone(),
            Self::Received(msg) => msg.from.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Conversation<RoleType: Clone + Debug> {
    pub messages: Vec<ConversationEntry<RoleType>>,
}

impl<RoleType: Clone + Debug> Default for Conversation<RoleType> {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
}

impl<RoleType: Clone + Debug> Conversation<RoleType> {
    pub fn format_for_oai(&self, system_prompt: String) -> Value {
        let mut messages: Vec<HashMap<String, Value>> = self
            .messages
            .iter()
            .map(|msg| {
                HashMap::from_iter(vec![
                    ("role".to_string(), json!("user")),
                    ("content".to_string(), json!(msg.get_content())),
                    (
                        "name".to_string(),
                        json!(format!("{:?}", msg.get_from_role())),
                    ),
                ])
            })
            .collect();

        messages.insert(
            0,
            HashMap::from_iter(vec![
                ("role".to_string(), json!("system")),
                ("content".to_string(), json!(system_prompt)),
            ]),
        );
        let out = json!({
            "model": "gpt-4-1106-preview",
            "messages": messages
        });

        out
    }
}

impl<RoleType: Clone + Debug> Conversation<RoleType> {
    pub fn add_message(&mut self, msg: ConversationEntry<RoleType>) {
        self.messages.push(msg);
    }
}

impl<RoleType: Clone + Debug> Default for Conversations<RoleType> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<RoleType: Clone + Debug> Conversations<RoleType> {
    pub fn get_or_create_conversation(&mut self, id: usize) -> &mut Conversation<RoleType> {
        self.0.entry(id).or_insert_with(|| Conversation::default())
    }
}
