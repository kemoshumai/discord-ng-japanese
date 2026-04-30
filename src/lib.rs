use twilight_model::gateway::payload::incoming::MessageCreate;

pub mod actors;
pub mod discord_loop;

pub type Message = Box<MessageCreate>;
