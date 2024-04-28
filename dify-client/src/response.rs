use super::request::{Feedback, FileType};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_with::{serde_as, EnumMap};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// 错误响应
#[derive(Debug, Serialize, Deserialize)]
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

/// 通用结果响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ResultResponse {
    pub result: String,
}

/// 发送对话消息的响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessageResponse {
    pub event: String,
    /// 消息唯一 ID
    pub message_id: String,
    /// 会话 ID
    pub conversation_id: String,
    /// App 模式
    pub mode: AppMode,
    /// 完整回复内容
    pub answer: String,
    /// 消息创建时间戳，如：1705395332
    pub created_at: u64,
    /// 元数据
    pub metadata: HashMap<String, JsonValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventBase {
    /// 消息唯一 ID
    pub message_id: String,
    /// 会话 ID
    pub conversation_id: String,
    /// 创建时间戳，如：1705395332
    pub created_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ChatMessageSteamEvent {
    /// LLM 返回文本块事件，即：完整的文本以分块的方式输出。
    Message {
        #[serde(flatten)]
        base: EventBase,
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
        #[serde(flatten)]
        base: EventBase,
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
        #[serde(flatten)]
        base: EventBase,
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
        #[serde(flatten)]
        base: EventBase,
        /// 任务 ID，用于请求跟踪和下方的停止响应接口
        task_id: String,
        /// 替换内容（直接替换 LLM 所有回复文本）
        answer: String,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },
    /// workflow 开始执行
    WorkflowStarted {
        #[serde(flatten)]
        base: EventBase,
        /// 任务 ID，用于请求跟踪和下方的停止响应接口
        task_id: String,
        /// workflow 执行 ID
        workflow_run_id: String,
        /// workflow 详细内容
        data: WorkflowData,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },

    /// node 执行开始
    NodeStarted {
        #[serde(flatten)]
        base: EventBase,
        /// 任务 ID，用于请求跟踪和下方的停止响应接口
        task_id: String,
        /// workflow 执行 ID
        workflow_run_id: String,
        /// node 详细内容
        data: NodeData,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },
    /// node 执行结束, 成功失败同一事件中不同状态
    NodeFinished {
        #[serde(flatten)]
        base: EventBase,
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
        #[serde(flatten)]
        base: EventBase,
        /// 任务 ID，用于请求跟踪和下方的停止响应接口
        task_id: String,
        /// workflow 执行 ID
        workflow_run_id: String,
        /// workflow 详细内容
        data: WorkflowFinishedData,
        #[serde(flatten)]
        extra: HashMap<String, JsonValue>,
    },
    /// 流式输出过程中出现的异常会以 stream event 形式输出，收到异常事件后即结束。
    Error {
        #[serde(flatten)]
        base: EventBase,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowData {
    /// workflow 执行 ID
    pub id: String,
    /// 关联 Workflow ID
    pub workflow_id: String,
    /// 自增序号，App 内自增，从 1 开始
    pub sequence_number: u32,
    /// 开始时间
    pub created_at: u64,
}

/// workflow 执行结束详细内容
#[derive(Debug, Serialize, Deserialize)]
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
}

/// node 详细内容
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeData {
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
#[derive(Debug, Serialize, Deserialize)]
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
}

/// 执行结束状态
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishedStatus {
    Running,
    Succeeded,
    Failed,
    Stopped,
}

/// 执行节点元数据
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// 总使用 tokens
    pub total_tokens: Option<u32>,
    /// 总费用
    pub total_price: Option<String>,
    /// 货币，如 USD / RMB
    pub currency: Option<String>,
}

/// 应用类型
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
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
#[derive(Debug, Deserialize, Serialize)]
pub struct MessagesSuggestedResponse {
    pub result: String,
    /// 建议问题列表
    pub data: Vec<String>,
}

/// 获取会话历史消息的响应
#[derive(Debug, Deserialize, Serialize)]
pub struct MessagesResponse {
    /// 返回条数，若传入超过系统限制，返回系统限制数量
    pub limit: u32,
    /// 是否存在下一页
    pub has_more: bool,
    /// 消息列表
    pub data: Vec<MessagesData>,
}

/// 历史消息数据
#[derive(Debug, Deserialize, Serialize)]
pub struct MessagesData {
    /// 消息 ID
    pub id: String,
    /// 会话 ID
    pub conversation_id: String,
    /// 用户输入参数。
    pub inputs: HashMap<String, String>,
    /// 用户输入 / 提问内容。
    pub query: String,
    /// 回答消息内容
    pub answer: String,
    /// 消息文件
    pub message_files: Vec<MessagesHistoryFile>,
    /// 反馈信息
    pub feedback: Option<MessagesHistoryFeedbacks>,
    /// 引用和归属分段列表
    pub retriever_resources: Vec<JsonValue>,
    /// 创建时间
    pub created_at: u64,
}

/// 历史消息数据中的文件信息
#[derive(Debug, Serialize, Deserialize)]
pub struct MessagesHistoryFile {
    /// ID
    id: String,
    /// 文件类型，image 图片
    #[serde(rename = "type")]
    type_: FileType,
    /// 预览图片地址
    url: String,
    /// 文件归属方，user 或 assistant
    belongs_to: BelongsTo,
}

/// 文件归属方
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BelongsTo {
    User,
    Assistant,
}

/// 历史消息数据中的反馈信息
#[derive(Debug, Deserialize, Serialize)]
pub struct MessagesHistoryFeedbacks {
    /// 点赞 like / 点踩 dislike
    pub rating: Feedback,
}

/// 获取会话列表的响应
#[derive(Debug, Deserialize, Serialize)]
pub struct ConversationsResponse {
    /// 是否存在下一页
    pub has_more: bool,
    /// 返回条数，若传入超过系统限制，返回系统限制数量
    pub limit: u32,
    /// 会话列表
    pub data: Vec<ConversationsData>,
}

/// 会话数据
#[derive(Debug, Deserialize, Serialize)]
pub struct ConversationsData {
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
#[derive(Debug, Deserialize, Serialize)]
/// 获取应用配置信息的响应
pub struct ParametersResponse {
    /// 开场白
    pub opening_statement: String,
    /// 开场推荐问题列表
    pub suggested_questions: Vec<String>,
    /// 启用回答后给出推荐问题。
    pub suggested_questions_after_answer: ParametersSuggestedQuestionsAfterAnswer,
    /// 语音转文本
    pub speech_to_text: ParametersSpeechToText,
    /// 引用和归属
    pub retriever_resource: ParametersRetrieverResource,
    /// 标记回复
    pub annotation_reply: ParametersAnnotationReply,
    /// 用户输入表单配置
    pub user_input_form: Vec<ParametersUserInputFormItem>,
    /// 文件上传配置
    #[serde_as(as = "EnumMap")]
    pub file_upload: Vec<ParametersFileUploadItem>,
    pub system_parameters: ParametersSystemParameters,
}

#[derive(Debug, Deserialize, Serialize)]
/// 启用回答后给出推荐问题。
pub struct ParametersSuggestedQuestionsAfterAnswer {
    /// 是否开启
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
/// 语音转文本
pub struct ParametersSpeechToText {
    /// 是否开启
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
/// 引用和归属
pub struct ParametersRetrieverResource {
    /// 是否开启
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
/// 标记回复
pub struct ParametersAnnotationReply {
    /// 是否开启
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
/// 用户输入表单配置
pub enum ParametersUserInputFormItem {
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
/// 文件上传配置
pub enum ParametersFileUploadItem {
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
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferMethod {
    RemoteUrl,
    LocalFile,
}

/// 系统参数
#[derive(Debug, Deserialize, Serialize)]
pub struct ParametersSystemParameters {
    /// 图片文件上传大小限制（MB）
    pub image_file_size_limit: String,
}

#[derive(Debug, Deserialize, Serialize)]
/// 获取应用Meta信息的响应
pub struct MetaResponse {
    pub tool_icons: HashMap<String, ToolIcon>,
}

/// 工具图标
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ToolIcon {
    Url(String),
    Emoji { background: String, content: String },
}

/// 语音转文字响应
#[derive(Debug, Deserialize, Serialize)]
pub struct AudioToTextResponse {
    /// 输出文字
    pub text: String,
}

/// 上传文件响应
#[derive(Debug, Deserialize, Serialize)]
pub struct FilesUploadResponse {
    /// ID
    id: String,
    /// 文件名
    name: String,
    /// 文件大小（byte）
    size: u64,
    /// 文件后缀
    extension: String,
    /// 文件 mime-type
    mime_type: String,
    /// 上传人 ID
    created_by: String,
    /// 上传时间
    created_at: u64,
}