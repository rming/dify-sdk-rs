pub use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 发送对话消息的请求
/// 创建会话消息。
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ChatMessageRequest {
    /// 允许传入 App 定义的各变量值。  
    /// inputs 参数包含了多组键值对（Key/Value pairs），每组的键对应一个特定变量，每组的值则是该变量的具体值。  
    /// 默认 {}  
    pub inputs: HashMap<String, String>,
    /// 用户输入/提问内容。
    pub query: String,
    /// 响应模式  
    /// * streaming 流式模式（推荐）。基于 SSE（Server-Sent Events）实现类似打字机输出方式的流式返回。
    /// * blocking 阻塞模式，等待执行完毕后返回结果。（请求若流程较长可能会被中断）。  
    /// 由于 Cloudflare 限制，请求会在 100 秒超时无返回后中断。
    pub response_mode: ResponseMode,
    /// 用户标识，用于定义终端用户的身份，方便检索、统计。  
    /// 由开发者定义规则，需保证用户标识在应用内唯一。  
    pub user: String,
    /// 会话 ID（选填），需要基于之前的聊天记录继续对话，必须传之前消息的 conversation_id。
    pub conversation_id: String,
    /// 上传的文件。
    pub files: Vec<ChatMessageFile>,
    /// 自动生成标题（选填），默认 true。  
    /// 若设置为 false，则可通过调用会话重命名接口并设置 auto_generate 为 true 实现异步生成标题。
    pub auto_generate_name: bool,
}

/// 响应模式
/// * streaming 流式模式（推荐）。基于 SSE（Server-Sent Events）实现类似打字机输出方式的流式返回。
/// * blocking 阻塞模式，等待执行完毕后返回结果。（请求若流程较长可能会被中断）。  
/// 由于 Cloudflare 限制，请求会在 100 秒超时无返回后中断。
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseMode {
    /// 阻塞模式
    #[default]
    Blocking,
    /// 流式模式
    Streaming,
}

/// 文件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    Image,
}

/// 上传的文件
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "transfer_method")]
pub enum ChatMessageFile {
    /// 图片地址方式传递
    RemoteUrl {
        /// 文件类型
        #[serde(rename = "type")]
        type_: FileType,
        /// 图片地址
        url: String,
    },
    /// 上传文件方式传递
    LocalFile {
        /// 文件类型
        #[serde(rename = "type")]
        type_: FileType,
        /// 上传文件 ID
        upload_file_id: String,
    },
}

/// 停止响应请求
#[derive(Debug, Serialize, Deserialize)]
pub struct StreamTaskStopRequest {
    /// 任务 ID，可在流式返回 Chunk 中获取
    pub task_id: String,
    /// 用户标识，用于定义终端用户的身份，必须和发送消息接口传入 user 保持一致。
    pub user: String,
}

/// 获取下一轮建议问题列表请求
#[derive(Debug, Serialize, Deserialize)]
pub struct MessagesSuggestedRequest {
    /// Message ID
    pub message_id: String,
}

/// 消息反馈请求
/// 消息终端用户反馈、点赞，方便应用开发者优化输出预期。
#[derive(Debug, Serialize, Deserialize)]
pub struct MessagesFeedbacksRequest {
    /// 消息 ID
    pub message_id: String,
    /// 点赞 Like, 点踩 Dislike, 撤销点赞 None
    pub rating: Option<Feedback>,
    /// 用户标识，由开发者定义规则，需保证用户标识在应用内唯一。
    pub user: String,
}

/// 消息反馈
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feedback {
    /// 点赞
    Like,
    /// 点踩
    Dislike,
}

/// 获取会话历史消息的请求
/// 滚动加载形式返回历史聊天记录，第一页返回最新 limit 条，即：倒序返回。
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MessagesRequest {
    /// 会话 ID
    pub conversation_id: String,
    /// 用户标识，由开发者定义规则，需保证用户标识在应用内唯一。
    pub user: String,
    /// 当前页第一条聊天记录的 ID，默认 None
    pub first_id: Option<String>,
    /// 一次请求返回多少条聊天记录，默认 20 条。
    pub limit: Option<u32>,
}

/// 获取会话列表的请求
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ConversationsRequest {
    /// 用户标识，由开发者定义规则，需保证用户标识在应用内唯一。
    pub user: String,
    /// 当前页最后面一条记录的 ID，默认 None
    pub last_id: Option<String>,
    /// 一次请求返回多少条记录
    pub limit: Option<u32>,
    /// 只返回置顶 true，只返回非置顶 false
    pub pinned: bool,
}

/// 获取应用配置信息的请求
#[derive(Debug, Deserialize, Serialize)]
pub struct ParametersRequest {
    /// 用户标识，由开发者定义规则，需保证用户标识在应用内唯一。
    pub user: String,
}

/// 获取应用Meta信息的请求
#[derive(Debug, Deserialize, Serialize)]
pub struct MetaRequest {
    /// 用户标识，由开发者定义规则，需保证用户标识在应用内唯一。
    pub user: String,
}

/// 会话重命名请求
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct ConversationsRenameRequest {
    /// 会话 ID
    pub conversation_id: String,
    /// 名称，若 auto_generate 为 true 时，该参数可不传
    pub name: Option<String>,
    /// 自动生成标题，默认 false。
    pub auto_generate: bool,
    /// 用户标识，由开发者定义规则，需保证用户标识在应用内唯一。
    pub user: String,
}

/// 删除会话请求
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct ConversationsDeleteRequest {
    /// 会话 ID
    pub conversation_id: String,
    /// 用户标识，由开发者定义规则，需保证用户标识在应用内唯一。
    pub user: String,
}

/// 文字转语音请求
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct TextToAudioRequest {
    /// 语音生成内容。
    pub text: String,
    /// 用户标识，由开发者定义规则，需保证用户标识在应用内唯一。
    pub user: String,
    /// 是否启用流式输出true、false。
    pub streaming: bool,
}

/// 语音转文字请求
#[derive(Default, Debug)]
pub struct AudioToTextRequest {
    /// 语音文件。   
    /// 支持格式：['mp3', 'mp4', 'mpeg', 'mpga', 'm4a', 'wav', 'webm'] 文件大小限制：15MB
    pub file: Bytes,
    /// 用户标识，由开发者定义规则，需保证用户标识在应用内唯一。
    pub user: String,
}

/// 上传文件请求  
#[derive(Default, Debug)]
pub struct FilesUploadRequest {
    /// 要上传的文件。
    pub file: Bytes,
    /// 用户标识，用于定义终端用户的身份，必须和发送消息接口传入 user 保持一致。
    pub user: String,
}

/// 执行 workflow 请求
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WorkflowsRunRequest {
    /// 允许传入 App 定义的各变量值。  
    /// inputs 参数包含了多组键值对（Key/Value pairs），每组的键对应一个特定变量，每组的值则是该变量的具体值。  
    /// 默认 {}  
    pub inputs: HashMap<String, String>,
    /// 响应模式  
    /// * streaming 流式模式（推荐）。基于 SSE（Server-Sent Events）实现类似打字机输出方式的流式返回。
    /// * blocking 阻塞模式，等待执行完毕后返回结果。（请求若流程较长可能会被中断）。  
    /// 由于 Cloudflare 限制，请求会在 100 秒超时无返回后中断。
    pub response_mode: ResponseMode,
    /// 用户标识，用于定义终端用户的身份，方便检索、统计。  
    /// 由开发者定义规则，需保证用户标识在应用内唯一。  
    pub user: String,
    /// 文件列表，适用于传入文件（图片）结合文本理解并回答问题，仅当模型支持 Vision 能力时可用。
    pub files: Vec<ChatMessageFile>,
}

/// 文本生成请求
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CompletionMessagesRequest {
    /// 允许传入 App 定义的各变量值。  
    /// inputs 参数包含了多组键值对（Key/Value pairs），每组的键对应一个特定变量，每组的值则是该变量的具体值。  
    /// 默认 {}  
    pub inputs: HashMap<String, String>,
    /// 响应模式  
    /// * streaming 流式模式（推荐）。基于 SSE（Server-Sent Events）实现类似打字机输出方式的流式返回。
    /// * blocking 阻塞模式，等待执行完毕后返回结果。（请求若流程较长可能会被中断）。  
    /// 由于 Cloudflare 限制，请求会在 100 秒超时无返回后中断。
    pub response_mode: ResponseMode,
    /// 用户标识，用于定义终端用户的身份，方便检索、统计。  
    /// 由开发者定义规则，需保证用户标识在应用内唯一。  
    pub user: String,
    /// 会话 ID（选填），需要基于之前的聊天记录继续对话，必须传之前消息的 conversation_id。
    pub conversation_id: String,
    /// 文件列表，适用于传入文件（图片）结合文本理解并回答问题，仅当模型支持 Vision 能力时可用。
    pub files: Vec<ChatMessageFile>,
}
