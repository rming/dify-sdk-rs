//! This module contains the response structures used in the Dify SDK.
//!
//! It includes error responses, result responses, chat message responses, and more.
//! These structures are used to deserialize JSON responses from the Dify API.
//!
//! # Examples
//!
//! Deserialize an error response:
//!
//! ```no_run
//! use dify_client::response::ErrorResponse;
//!
//! let json = r#"
//!     {
//!         "code": "unknown_error",
//!         "message": "An unknown error occurred",
//!         "status": 503
//!     }
//! "#;
//!
//! let error_response: ErrorResponse = serde_json::from_str(json).unwrap();
//!
//! assert_eq!(error_response.code, "unknown_error");
//! assert_eq!(error_response.message, "An unknown error occurred");
//! assert_eq!(error_response.status, 503);
//! ```
//!
//! Deserialize a chat message response:
//!
//! ```no_run
//! use dify_client::response::{ChatMessagesResponse, AppMode};
//! use serde_json::json;
//! use std::collections::HashMap;
//!
//! let json = r#"
//!     {
//!         "base": {
//!             "message_id": "12345",
//!             "conversation_id": "67890",
//!             "created_at": 1705395332
//!         },
//!         "event": "message",
//!         "mode": "chat",
//!         "answer": "Hello, how can I help you?",
//!         "metadata": {
//!             "key1": "value1",
//!             "key2": "value2"
//!         }
//!     }
//! "#;
//!
//! let mut metadata = HashMap::new();
//! metadata.insert("key1".to_string(), json!("value1"));
//! metadata.insert("key2".to_string(), json!("value2"));
//!
//! let chat_response: ChatMessagesResponse = serde_json::from_str(json).unwrap();
//!
//! assert_eq!(chat_response.base.message_id, "12345");
//! assert_eq!(chat_response.base.conversation_id.unwrap(), "67890");
//! assert_eq!(chat_response.base.created_at, 1705395332);
//! assert_eq!(chat_response.event, "message");
//! assert_eq!(chat_response.mode, AppMode::Chat);
//! assert_eq!(chat_response.answer, "Hello, how can I help you?");
//! assert_eq!(chat_response.metadata, metadata);
//! ```
//!
use super::request::{Feedback, FileType};
use anyhow::{anyhow, bail, Result as AnyResult};
use eventsource_stream::EventStream;
use futures::Stream;
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_with::{serde_as, EnumMap};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter, Result as FmtResult},
    pin::Pin,
    task::{Context, Poll},
};

/// 错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub status: u32,
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl ErrorResponse {
    pub fn unknown<T>(message: T) -> Self
    where
        T: ToString,
    {
        ErrorResponse {
            code: "unknown_error".into(),
            message: message.to_string(),
            status: 503,
        }
    }
}

/// 通用结果响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultResponse {
    pub result: String,
}

/// 对话基础信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBase {
    /// 消息唯一 ID
    pub message_id: String,
    /// 会话 ID
    pub conversation_id: Option<String>,
    /// 创建时间戳，如：1705395332
    pub created_at: u64,
}

/// 发送对话消息的响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessagesResponse {
    /// 消息基础信息
    #[serde(flatten)]
    pub base: MessageBase,
    /// 事件
    pub event: String,
    /// App 模式
    pub mode: AppMode,
    /// 完整回复内容
    pub answer: String,
    /// 元数据
    pub metadata: HashMap<String, JsonValue>,
}

/// 流式模式分块数据事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SseMessageEvent {
    /// LLM 返回文本块事件，即：完整的文本以分块的方式输出。
    Message {
        /// 消息基础信息
        #[serde(flatten)]
        base: Option<MessageBase>,
        /// 消息 ID
        id: String,
        /// 任务 ID，用于请求跟踪和下方的停止响应接口
        task_id: String,
        /// LLM 返回文本块内容
        answer: String,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },
    /// 文件事件，表示有新文件需要展示
    MessageFile {
        /// 消息基础信息
        #[serde(flatten)]
        base: Option<MessageBase>,
        /// 文件唯一 ID
        id: String,
        /// 文件类型，目前仅为 image
        #[serde(rename = "type")]
        type_: FileType,
        /// 文件归属，user 或 assistant，该接口返回仅为 assistant
        belongs_to: BelongsTo,
        /// 文件访问地址
        url: String,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },
    /// 消息结束事件，收到此事件则代表流式返回结束。
    MessageEnd {
        /// 消息基础信息
        #[serde(flatten)]
        base: Option<MessageBase>,
        /// 消息 ID
        id: String,
        /// 任务 ID，用于请求跟踪和下方的停止响应接口
        task_id: String,
        /// 元数据
        metadata: HashMap<String, JsonValue>,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },
    /// 消息内容替换事件  
    /// 开启内容审查和审查输出内容时，若命中了审查条件，则会通过此事件替换消息内容为预设回复。
    MessageReplace {
        /// 消息基础信息
        #[serde(flatten)]
        base: Option<MessageBase>,
        /// 任务 ID，用于请求跟踪和下方的停止响应接口
        task_id: String,
        /// 替换内容（直接替换 LLM 所有回复文本）
        answer: String,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },
    /// workflow 开始执行
    WorkflowStarted {
        /// 消息基础信息
        #[serde(flatten)]
        base: Option<MessageBase>,
        /// 任务 ID，用于请求跟踪和下方的停止响应接口
        task_id: String,
        /// workflow 执行 ID
        workflow_run_id: String,
        /// workflow 详细内容
        data: WorkflowStartedData,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },

    /// node 执行开始
    NodeStarted {
        /// 消息基础信息
        #[serde(flatten)]
        base: Option<MessageBase>,
        /// 任务 ID，用于请求跟踪和下方的停止响应接口
        task_id: String,
        /// workflow 执行 ID
        workflow_run_id: String,
        /// node 详细内容
        data: NodeStartedData,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },
    /// node 执行结束, 成功失败同一事件中不同状态
    NodeFinished {
        /// 消息基础信息
        #[serde(flatten)]
        base: Option<MessageBase>,
        /// 任务 ID，用于请求跟踪和下方的停止响应接口
        task_id: String,
        /// workflow 执行 ID
        workflow_run_id: String,
        /// node 执行结束详细内容
        data: NodeFinishedData,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },
    /// workflow 执行结束，成功失败同一事件中不同状态
    WorkflowFinished {
        /// 消息基础信息
        #[serde(flatten)]
        base: Option<MessageBase>,
        /// 任务 ID，用于请求跟踪和下方的停止响应接口
        task_id: String,
        /// workflow 执行 ID
        workflow_run_id: String,
        /// workflow 详细内容
        data: WorkflowFinishedData,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },
    /// Agent模式下返回文本块事件，即：在Agent模式下，文章的文本以分块的方式输出（仅Agent模式下使用）
    AgentMessage {
        /// 消息基础信息
        #[serde(flatten)]
        base: Option<MessageBase>,
        /// 消息 ID
        id: String,
        /// 任务 ID，用于请求跟踪和下方的停止响应接口
        task_id: String,
        /// LLM 返回文本块内容
        answer: String,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },
    /// Agent模式下有关Agent思考步骤的相关内容，涉及到工具调用（仅Agent模式下使用）
    AgentThought {
        /// 消息基础信息
        #[serde(flatten)]
        base: Option<MessageBase>,
        /// 消息 ID
        id: String,
        /// 任务 ID，用于请求跟踪和下方的停止响应接口
        task_id: String,
        /// agent_thought在消息中的位置，如第一轮迭代position为1
        position: u32,
        /// agent的思考内容
        thought: String,
        /// 工具调用的返回结果
        observation: String,
        /// 使用的工具列表，以 ; 分割多个工具
        tool: String,
        /// 工具的标签
        tool_labels: JsonValue,
        /// 工具的输入，JSON格式的字符串
        tool_input: String,
        /// 当前 agent_thought 关联的文件ID
        message_files: Vec<String>,
    },
    /// 流式输出过程中出现的异常会以 stream event 形式输出，收到异常事件后即结束。
    Error {
        /// 消息基础信息
        #[serde(flatten)]
        base: Option<MessageBase>,
        /// HTTP 状态码
        status: u32,
        /// 错误码
        code: String,
        /// 错误消息
        message: String,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },
    // 每 10s 一次的 ping 事件，保持连接存活。
    Ping,
}

/// workflow 详细内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStartedData {
    /// workflow 执行 ID
    pub id: String,
    /// 关联 Workflow ID
    pub workflow_id: String,
    /// 自增序号，App 内自增，从 1 开始
    pub sequence_number: u32,
    /// 输入数据
    pub inputs: JsonValue,
    /// 开始时间
    pub created_at: u64,
}

/// workflow 执行结束详细内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFinishedData {
    /// workflow 执行 ID
    pub id: String,
    /// 关联 Workflow ID
    pub workflow_id: String,
    /// 执行状态 running / succeeded / failed / stopped
    pub status: FinishedStatus,
    /// 输出内容
    pub outputs: Option<JsonValue>,
    /// 错误原因
    pub error: Option<String>,
    /// 耗时(s)
    pub elapsed_time: Option<f64>,
    /// 总使用 tokens
    pub total_tokens: Option<u32>,
    /// 总步数（冗余），默认 0
    pub total_steps: u32,
    /// 开始时间
    pub created_at: u64,
    /// 结束时间
    pub finished_at: u64,
    #[serde(flatten)]
    pub extra: HashMap<String, JsonValue>,
}

/// node 详细内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStartedData {
    /// workflow 执行 ID
    pub id: String,
    /// 节点 ID
    pub node_id: String,
    /// 节点类型
    pub node_type: String,
    /// 节点名称
    pub title: String,
    /// 执行序号，用于展示 Tracing Node 顺序
    pub index: u32,
    /// 前置节点 ID，用于画布展示执行路径
    pub predecessor_node_id: Option<String>,
    /// 节点中所有使用到的前置节点变量内容
    pub inputs: Option<JsonValue>,
    /// 开始时间
    pub created_at: u64,
}

/// node 执行结束详细内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeFinishedData {
    /// node 执行 ID
    pub id: String,
    /// 节点 ID
    pub node_id: String,
    /// 执行序号，用于展示 Tracing Node 顺序
    pub index: u32,
    /// 前置节点 ID，用于画布展示执行路径
    pub predecessor_node_id: Option<String>,
    /// 节点中所有使用到的前置节点变量内容
    pub inputs: Option<JsonValue>,
    /// 节点过程数据
    pub process_data: Option<JsonValue>,
    /// 输出内容
    pub outputs: Option<JsonValue>,
    /// 执行状态 running / succeeded / failed / stopped
    pub status: FinishedStatus,
    /// 错误原因
    pub error: Option<String>,
    /// 耗时(s)
    pub elapsed_time: Option<f64>,
    /// 执行节点元数据
    pub execution_metadata: Option<ExecutionMetadata>,
    /// 开始时间
    pub created_at: u64,
    #[serde(flatten)]
    pub extra: HashMap<String, JsonValue>,
}

/// 执行结束状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishedStatus {
    Running,
    Succeeded,
    Failed,
    Stopped,
}

/// 执行节点元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// 总使用 tokens
    pub total_tokens: Option<u32>,
    /// 总费用
    pub total_price: Option<String>,
    /// 货币，如 USD / RMB
    pub currency: Option<String>,
}

/// 应用类型
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum AppMode {
    Completion,
    Workflow,
    Chat,
    AdvancedChat,
    AgentChat,
    Channel,
}

/// 获取下一轮建议问题列表的响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessagesSuggestedResponse {
    pub result: String,
    /// 建议问题列表
    pub data: Vec<String>,
}

/// 获取会话历史消息的响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessagesResponse {
    /// 返回条数，若传入超过系统限制，返回系统限制数量
    pub limit: u32,
    /// 是否存在下一页
    pub has_more: bool,
    /// 消息列表
    pub data: Vec<MessageData>,
}

/// 历史消息数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageData {
    /// 消息 ID
    pub id: String,
    /// 会话 ID
    pub conversation_id: String,
    /// 用户输入参数。
    pub inputs: JsonValue,
    /// 用户输入 / 提问内容。
    pub query: String,
    /// 回答消息内容
    pub answer: String,
    /// 消息文件
    pub message_files: Vec<MessageFile>,
    /// 反馈信息
    pub feedback: Option<MessageFeedback>,
    /// 引用和归属分段列表
    pub retriever_resources: Vec<JsonValue>,
    /// 创建时间
    pub created_at: u64,
}

/// 历史消息数据中的文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFile {
    /// ID
    pub id: String,
    /// 文件类型，image 图片
    #[serde(rename = "type")]
    pub type_: FileType,
    /// 预览图片地址
    pub url: String,
    /// 文件归属方，user 或 assistant
    pub belongs_to: BelongsTo,
}

/// 文件归属方
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BelongsTo {
    User,
    Assistant,
}

/// 历史消息数据中的反馈信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageFeedback {
    /// 点赞 like / 点踩 dislike
    pub rating: Feedback,
}

/// 获取会话列表的响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConversationsResponse {
    /// 是否存在下一页
    pub has_more: bool,
    /// 返回条数，若传入超过系统限制，返回系统限制数量
    pub limit: u32,
    /// 会话列表
    pub data: Vec<ConversationData>,
}

/// 会话数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConversationData {
    /// 会话 ID
    pub id: String,
    /// 会话名称，默认为会话中用户最开始问题的截取。
    pub name: String,
    /// 用户输入参数。
    pub inputs: HashMap<String, String>,
    /// 开场白
    pub introduction: String,
    /// 创建时间
    pub created_at: u64,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
/// 获取应用配置信息的响应
pub struct ParametersResponse {
    /// 开场白
    pub opening_statement: String,
    /// 开场推荐问题列表
    pub suggested_questions: Vec<String>,
    /// 启用回答后给出推荐问题。
    pub suggested_questions_after_answer: ParameterSuggestedQuestionsAfterAnswer,
    /// 语音转文本
    pub speech_to_text: ParameterSpeechToText,
    /// 引用和归属
    pub retriever_resource: ParameterRetrieverResource,
    /// 标记回复
    pub annotation_reply: ParameterAnnotationReply,
    /// 用户输入表单配置
    pub user_input_form: Vec<ParameterUserInputFormItem>,
    /// 文件上传配置
    #[serde_as(as = "EnumMap")]
    pub file_upload: Vec<ParameterFileUploadItem>,
    pub system_parameters: SystemParameters,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// 启用回答后给出推荐问题。
pub struct ParameterSuggestedQuestionsAfterAnswer {
    /// 是否开启
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// 语音转文本
pub struct ParameterSpeechToText {
    /// 是否开启
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// 引用和归属
pub struct ParameterRetrieverResource {
    /// 是否开启
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// 标记回复
pub struct ParameterAnnotationReply {
    /// 是否开启
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
/// 用户输入表单配置
pub enum ParameterUserInputFormItem {
    /// 文本输入控件
    #[serde(rename = "text-input")]
    TextInput {
        /// 控件展示标签名
        label: String,
        /// 控件 ID
        variable: String,
        /// 是否必填
        required: bool,
    },
    /// 段落文本输入控件
    Paragraph {
        /// 控件展示标签名
        label: String,
        /// 控件 ID
        variable: String,
        /// 是否必填
        required: bool,
    },
    /// 数字输入空间
    Number {
        /// 控件展示标签名
        label: String,
        /// 控件 ID
        variable: String,
        /// 是否必填
        required: bool,
    },
    Select {
        /// 控件展示标签名
        label: String,
        /// 控件 ID
        variable: String,
        /// 是否必填
        required: bool,
        /// 选项值
        options: Vec<String>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
/// 文件上传配置
pub enum ParameterFileUploadItem {
    /// 当前仅支持图片类型
    Image {
        /// 是否开启
        enabled: bool,
        /// 图片数量限制，默认 3
        number_limits: u32,
        /// 传递方式
        transfer_methods: Vec<TransferMethod>,
    },
}

/// 文件传递方式
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferMethod {
    RemoteUrl,
    LocalFile,
}

/// 系统参数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemParameters {
    /// 图片文件上传大小限制（MB）
    pub image_file_size_limit: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// 获取应用Meta信息的响应
pub struct MetaResponse {
    pub tool_icons: HashMap<String, ToolIcon>,
}

/// 工具图标
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ToolIcon {
    Url(String),
    Emoji { background: String, content: String },
}

/// 语音转文字响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AudioToTextResponse {
    /// 输出文字
    pub text: String,
}

/// 上传文件响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilesUploadResponse {
    /// ID
    pub id: String,
    /// 文件名
    pub name: String,
    /// 文件大小（byte）
    pub size: u64,
    /// 文件后缀
    pub extension: String,
    /// 文件 mime-type
    pub mime_type: String,
    /// 上传人 ID
    pub created_by: String,
    /// 上传时间
    pub created_at: u64,
}

/// 执行 workflow 响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowsRunResponse {
    /// workflow 执行 ID
    pub workflow_run_id: String,
    /// 任务 ID，用于请求跟踪和下方的停止响应接口
    pub task_id: String,
    /// 详细内容
    pub data: WorkflowFinishedData,
}

/// 文本生成的响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionMessagesResponse {
    /// 消息基础信息
    #[serde(flatten)]
    pub base: MessageBase,
    /// 任务 ID，用于请求跟踪和下方的停止响应接口
    pub task_id: String,
    /// 事件
    pub event: String,
    /// App 模式
    pub mode: AppMode,
    /// 完整回复内容
    pub answer: String,
    /// 元数据
    pub metadata: HashMap<String, JsonValue>,
}

pin_project! {
    /// A Stream of SSE message events.
    pub struct SseMessageEventStream<S>
    {
        #[pin]
        stream: EventStream<S>,
        terminated: bool,
    }
}

impl<S> SseMessageEventStream<S> {
    /// Initialize the SSE message events stream with a Stream
    pub fn new(stream: EventStream<S>) -> Self {
        Self {
            stream,
            terminated: false,
        }
    }
}

impl<S, B, E> Stream for SseMessageEventStream<S>
where
    S: Stream<Item = Result<B, E>>,
    B: AsRef<[u8]>,
    E: Display,
{
    type Item = AnyResult<SseMessageEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        if *this.terminated {
            return Poll::Ready(None);
        }

        loop {
            match this.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(event))) => match event.event.as_str() {
                    "message" => match serde_json::from_str::<SseMessageEvent>(&event.data) {
                        Ok(msg_event) => return Poll::Ready(Some(Ok(msg_event))),
                        Err(e) => return Poll::Ready(Some(Err(e.into()))),
                    },
                    _ => {}
                },
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(anyhow!(e.to_string())))),
                Poll::Ready(None) => {
                    *this.terminated = true;
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// 解析响应
pub(crate) fn parse_response<T>(text: &str) -> AnyResult<T>
where
    T: serde::de::DeserializeOwned,
{
    if let Ok(data) = serde_json::from_str::<T>(text) {
        Ok(data)
    } else {
        parse_error_response(text)
    }
}

/// 解析错误响应
pub(crate) fn parse_error_response<T>(text: &str) -> AnyResult<T> {
    if let Ok(err) = serde_json::from_str::<ErrorResponse>(text) {
        bail!(err)
    } else {
        bail!(ErrorResponse::unknown(text))
    }
}
