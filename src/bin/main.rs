use actix::prelude::*;
use agentz::ai_chat_system::AiChatSystem;

fn main() {
    let sys = System::new();
    let debate_topic = "the Treaty of Versailles was the main cause of World War II".to_string();
    sys.block_on(AiChatSystem::run(debate_topic));
    sys.run().unwrap();
}
