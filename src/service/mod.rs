use rmcp::{ServerHandler, handler::server::tool::ToolRouter, model::*, tool_handler, tool_router};

mod echo;
mod fetch;
mod fs;

#[derive(Clone)]
pub struct DiveDefaultService {
    http_client: reqwest::Client,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl DiveDefaultService {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            tool_router: Self::tool_router_echo()
                + Self::tool_router_fetch()
                + Self::tool_router_fs(),
        }
    }
}

#[tool_handler]
impl ServerHandler for DiveDefaultService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("default mcp server for dive client".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_tool_list_changed()
                .build(),
            ..Default::default()
        }
    }
}
