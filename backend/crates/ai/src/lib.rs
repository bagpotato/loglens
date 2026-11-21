mod chat;
mod explain;
mod factory;
mod masking;
mod prompts;
mod provider;
mod providers;
mod query_planner;
mod stream_lines;

pub use chat::{chat, ChatEvent};
pub use explain::{explain, CONTEXT_LINES};
pub use factory::build_provider;
pub use masking::{unmask_stream, Masker};
pub use prompts::{CHAT_PROMPT, CLASSIFY_PROMPT, EXPLAIN_PROMPT, PLAN_QUERY_PROMPT};
pub use provider::{AiError, AiProvider, ChatMessage, Role, TextStream};
pub use providers::{OllamaProvider, OpenAiProvider};
pub use query_planner::{fallback_filters, parse_plan, plan_query, MAX_CHAT_LOGS};
