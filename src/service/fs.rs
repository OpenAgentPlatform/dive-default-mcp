use crate::service::DiveDefaultService;
use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    tool, tool_router,
};

use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use tokio::fs;
use tokio::io::AsyncReadExt;

#[derive(Deserialize, schemars::JsonSchema)]
struct ReadFileParams {
    /// The path to the file to read
    path: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct WriteFileParams {
    /// The path to the file to write
    path: String,
    /// The content to write to the file
    content: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct ListDirectoryParams {
    /// The path to the directory to list
    path: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct CreateDirectoryParams {
    /// The path to the directory to create
    path: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct DeleteFileParams {
    /// The path to the file to delete
    path: String,
}

/// Check if a file is binary by reading the first 8KB and looking for null bytes
async fn is_binary_file(path: &str) -> Result<bool, std::io::Error> {
    let mut file = fs::File::open(path).await?;
    let mut buffer = vec![0u8; 8192];
    let bytes_read = file.read(&mut buffer).await?;

    // Check for null bytes in the first chunk
    Ok(buffer[..bytes_read].contains(&0))
}

#[tool_router(router = tool_router_fs, vis = "pub")]
impl DiveDefaultService {
    #[tool(description = "Read file content from the specified path")]
    async fn read_file(
        &self,
        Parameters(params): Parameters<ReadFileParams>,
    ) -> Result<CallToolResult, McpError> {
        // Check if file is binary
        let is_binary = match is_binary_file(&params.path).await {
            Ok(is_bin) => is_bin,
            Err(e) => {
                return Err(McpError::new(
                    rmcp::model::ErrorCode::INTERNAL_ERROR,
                    format!("Failed to check file type: {}", e),
                    None,
                ));
            }
        };

        if is_binary {
            // Read binary file and encode as base64
            match fs::read(&params.path).await {
                Ok(bytes) => {
                    let base64_content = general_purpose::STANDARD.encode(&bytes);
                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "[Binary file encoded as base64]\n{}",
                        base64_content
                    ))]))
                }
                Err(e) => Err(McpError::new(
                    rmcp::model::ErrorCode::INTERNAL_ERROR,
                    format!("Failed to read binary file: {}", e),
                    None,
                )),
            }
        } else {
            // Read text file normally
            match fs::read_to_string(&params.path).await {
                Ok(content) => Ok(CallToolResult::success(vec![Content::text(content)])),
                Err(e) => Err(McpError::new(
                    rmcp::model::ErrorCode::INTERNAL_ERROR,
                    format!("Failed to read file: {}", e),
                    None,
                )),
            }
        }
    }

    #[tool(description = "Write content to a file at the specified path")]
    async fn write_file(
        &self,
        Parameters(params): Parameters<WriteFileParams>,
    ) -> Result<CallToolResult, McpError> {
        match fs::write(&params.path, &params.content).await {
            Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Successfully wrote to {}",
                params.path
            ))])),
            Err(e) => Err(McpError::new(
                rmcp::model::ErrorCode::INTERNAL_ERROR,
                format!("Failed to write file: {}", e),
                None,
            )),
        }
    }

    #[tool(description = "List all files and directories in the specified path")]
    async fn list_directory(
        &self,
        Parameters(params): Parameters<ListDirectoryParams>,
    ) -> Result<CallToolResult, McpError> {
        match fs::read_dir(&params.path).await {
            Ok(mut entries) => {
                let mut items = Vec::new();
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        let file_type = if entry.path().is_dir() {
                            "directory"
                        } else {
                            "file"
                        };
                        items.push(format!("{} ({})", file_name, file_type));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text(
                    items.join("\n"),
                )]))
            }
            Err(e) => Err(McpError::new(
                rmcp::model::ErrorCode::INTERNAL_ERROR,
                format!("Failed to list directory: {}", e),
                None,
            )),
        }
    }

    #[tool(description = "Create a new directory at the specified path")]
    async fn create_directory(
        &self,
        Parameters(params): Parameters<CreateDirectoryParams>,
    ) -> Result<CallToolResult, McpError> {
        match fs::create_dir_all(&params.path).await {
            Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Successfully created directory: {}",
                params.path
            ))])),
            Err(e) => Err(McpError::new(
                rmcp::model::ErrorCode::INTERNAL_ERROR,
                format!("Failed to create directory: {}", e),
                None,
            )),
        }
    }

    #[tool(description = "Delete a file at the specified path")]
    async fn delete_file(
        &self,
        Parameters(params): Parameters<DeleteFileParams>,
    ) -> Result<CallToolResult, McpError> {
        match fs::remove_file(&params.path).await {
            Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Successfully deleted file: {}",
                params.path
            ))])),
            Err(e) => Err(McpError::new(
                rmcp::model::ErrorCode::INTERNAL_ERROR,
                format!("Failed to delete file: {}", e),
                None,
            )),
        }
    }
}
