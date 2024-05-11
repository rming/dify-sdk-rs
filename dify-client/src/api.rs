//! This module provides a client for interacting with the Dify API.
//!
//! The `Api` struct in this module allows you to send requests to the Dify API and handle the responses.
//! It provides methods for various API endpoints such as sending chat messages, uploading files, and executing workflows.
//! The module also defines several request and response structs that are used for interacting with the API.
//!
//! # Example
//!
//! ```rust
//! use dify_client::api::Api;
//! use dify_client::request::ChatMessagesRequest;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a client for interacting with the Dify API
//!     let client = dify_client::Client::new("https://api.dify.ai", "API_KEY");
//!
//!     // Create an API instance using the client
//!     let api = Api::new(&client);
//!
//!     // Create a chat message request
//!     let request = ChatMessagesRequest {
//!         query: "What is the weather today?".to_string(),
//!         user: "user123".to_string(),
//!         ..Default::default()  
//!     };
//!
//!     // Send the chat message request and get the response
//!     let response = api.chat_messages(request).await;
//!
//!     // Handle the response
//!     match response {
//!         Ok(response) => {
//!             // Do something with the response
//!             println!("Chat message sent successfully: {:?}", response);
//!         }
//!         Err(error) => {
//!             // Handle the error
//!             eprintln!("Failed to send chat message: {}", error);
//!         }
//!     }
//! }
//! ```
//! This module provides a client for interacting with the Dify API.
//!
use super::{
    client::Client,
    http::{multipart, Method, Request},
    request::{
        AudioToTextRequest, Bytes, ChatMessagesRequest, CompletionMessagesRequest,
        ConversationsDeleteRequest, ConversationsRenameRequest, ConversationsRequest,
        FilesUploadRequest, MessagesFeedbacksRequest, MessagesRequest, MessagesSuggestedRequest,
        MetaRequest, ParametersRequest, ResponseMode, StreamTaskStopRequest, TextToAudioRequest,
        WorkflowsRunRequest,
    },
    response::{
        parse_error_response, parse_response, AudioToTextResponse, ChatMessagesResponse,
        CompletionMessagesResponse, ConversationsResponse, FilesUploadResponse, MessagesResponse,
        MessagesSuggestedResponse, MetaResponse, ParametersResponse, ResultResponse,
        SseMessageEvent, WorkflowsRunResponse,
    },
};
use anyhow::{bail, Result};
use eventsource_stream::Eventsource;
use futures::stream::StreamExt;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// API 路径
#[derive(Debug)]
pub enum ApiPath {
    /// 发送对话消息, 创建会话消息。
    ChatMessages,
    /// 上传文件
    /// 上传文件（目前仅支持图片）并在发送消息时使用，可实现图文多模态理解。  
    /// 支持 png, jpg, jpeg, webp, gif 格式。  
    /// 上传的文件仅供当前终端用户使用。
    FilesUpload,
    /// 停止响应, 仅支持流式模式。
    ChatMessagesStop,
    /// 消息反馈（点赞, 消息终端用户反馈、点赞，方便应用开发者优化输出预期。
    MessagesFeedbacks,
    /// 获取下一轮建议问题列表
    MessagesSuggested,
    /// 获取会话历史消息, 滚动加载形式返回历史聊天记录，第一页返回最新 limit 条，即：倒序返回。
    Messages,
    /// 获取会话列表, 获取当前用户的会话列表，默认返回最近的 20 条。
    Conversations,
    /// 删除会话
    ConversationsDelete,
    /// 会话重命名, 对会话进行重命名，会话名称用于显示在支持多会话的客户端上。
    ConversationsRename,
    /// 语音转文字
    AudioToText,
    /// 文字转语音
    TextToAudio,
    /// 获取应用配置信息, 用于进入页面一开始，获取功能开关、输入参数名称、类型及默认值等使用。
    Parameters,
    /// 获取应用Meta信息, 用于获取工具icon
    Meta,

    /// workflow
    /// 执行 workflow
    WorkflowsRun,
    /// 停止响应, 仅支持流式模式。
    WorkflowsStop,

    /// completion 文本生成
    /// 发送请求给文本生成型应用
    CompletionMessages,
    /// 文本生成停止响应
    CompletionMessagesStop,
}

/// API 路径
impl ApiPath {
    /// 获取 API 路径
    /// # Example
    /// ```no_run
    /// use dify_client::api::ApiPath;
    /// let path = ApiPath::ChatMessages;
    /// assert_eq!(path.as_str(), "/v1/chat-messages");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiPath::ChatMessages => "/v1/chat-messages",
            ApiPath::FilesUpload => "/v1/files/upload",
            ApiPath::ChatMessagesStop => "/v1/chat-messages/{task_id}/stop",
            ApiPath::MessagesFeedbacks => "/v1/messages/{message_id}/feedbacks",
            ApiPath::MessagesSuggested => "/v1/messages/{message_id}/suggested",
            ApiPath::Messages => "/v1/messages",
            ApiPath::Conversations => "/v1/conversations",
            ApiPath::ConversationsDelete => "/v1/conversations/{conversation_id}",
            ApiPath::ConversationsRename => "/v1/conversations/{conversation_id}/name",
            ApiPath::AudioToText => "/v1/audio-to-text",
            ApiPath::TextToAudio => "/v1/text-to-audio",
            ApiPath::Parameters => "/v1/parameters",
            ApiPath::Meta => "/v1/meta",
            ApiPath::WorkflowsRun => "/v1/workflows/run",
            ApiPath::WorkflowsStop => "/v1/workflows/{task_id}/stop",
            ApiPath::CompletionMessages => "/v1/completion-messages",
            ApiPath::CompletionMessagesStop => "/v1/completion-messages/{task_id}/stop",
        }
    }
}

impl Display for ApiPath {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.as_str())
    }
}

/// 发送请求前的钩子函数
type BeforeSend = Option<Box<dyn Fn(Request) -> Request + Send + Sync>>;

/// Dify API
pub struct Api<'a> {
    before_send_hook: BeforeSend,
    pub(crate) client: &'a Client,
}

/// Dify API
impl<'a> Api<'a> {
    /// Creates a new `Api` instance with the specified client.
    ///
    /// # Arguments
    /// * `client` - The client for interacting with the Dify API.
    pub fn new(client: &'a Client) -> Self {
        Self {
            before_send_hook: None,
            client,
        }
    }

    /// Sets a hook function to be called before sending a request.
    /// The hook function is called with the request before it is sent.
    /// The hook function can be used to modify the request before it is sent.
    /// The hook function should return the modified request.
    /// The hook function can be used to add headers, query parameters, etc.
    ///
    /// # Arguments
    /// * `hook` - The hook function to be called before sending a request.
    pub fn before_send<F>(&mut self, hook: F)
    where
        F: Fn(Request) -> Request + Send + Sync + 'static,
    {
        self.before_send_hook = Some(Box::new(hook));
    }

    /// Sends a request to the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req` - The request to send.
    ///
    /// # Returns
    /// A `Result` containing the response or an error.
    async fn send(&self, mut req: Request) -> Result<reqwest::Response> {
        if let Some(hook) = self.before_send_hook.as_ref() {
            req = hook(req);
        }
        self.client.execute(req).await
    }

    /// Builds the API request URL.
    ///
    /// # Arguments
    /// * `api_path` - The API path.
    ///
    /// # Returns
    /// The request URL.
    fn build_request_api(&self, api_path: ApiPath) -> String {
        self.client.config.base_url.clone() + api_path.as_str()
    }

    /// Creates a chat message request.
    ///
    /// # Arguments
    /// * `req` - The chat message request data.
    ///
    /// # Returns
    /// A `Result` containing the request or an error.
    ///
    /// # Errors
    /// Returns an error if the request cannot be created.
    fn create_chat_messages_request(&self, req: ChatMessagesRequest) -> Result<Request> {
        let url = self.build_request_api(ApiPath::ChatMessages);
        self.client.create_request(url, Method::POST, req)
    }

    /// Sends a chat message request to the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The chat message request data.
    ///
    /// # Returns
    /// A `Result` containing the chat message response or an error.
    pub async fn chat_messages(
        &self,
        mut req_data: ChatMessagesRequest,
    ) -> Result<ChatMessagesResponse> {
        req_data.response_mode = ResponseMode::Blocking;

        let req = self.create_chat_messages_request(req_data)?;
        let resp = self.send(req).await?;
        let text = resp.text().await?;
        parse_response::<ChatMessagesResponse>(&text)
    }

    /// Sends a chat message request to the Dify API and returns the response as a stream.
    /// The callback function is called for each event in the stream.
    /// The callback function should return `Some(T)` if the event is processed successfully, otherwise `None`.
    /// The function returns a vector of the processed events.
    /// The stream is stopped when the callback function returns an error or the stream ends.
    ///
    /// # Arguments
    ///
    /// * `req_data` - The chat message request data.
    /// * `callback` - The callback function to process the stream events.
    ///
    /// # Returns
    /// A `Result` containing the processed events or an error.
    ///
    /// # Errors
    /// Returns an error if the request cannot be created or the stream fails.
    pub async fn chat_messages_stream<F, T>(
        &self,
        mut req_data: ChatMessagesRequest,
        callback: F,
    ) -> Result<Vec<T>>
    where
        F: Fn(SseMessageEvent) -> Result<Option<T>> + Send + Sync,
    {
        req_data.response_mode = ResponseMode::Streaming;

        let req = self.create_chat_messages_request(req_data)?;
        let resp = self.send(req).await?;
        let mut stream = resp.bytes_stream().eventsource();

        let mut ret: Vec<T> = Vec::new();
        while let Some(event) = stream.next().await {
            let event = event?;
            if event.event == "message" {
                match serde_json::from_str::<SseMessageEvent>(&event.data) {
                    Ok(msg_event) => {
                        if let Some(answer) = callback(msg_event)? {
                            ret.push(answer);
                        }
                    }
                    Err(e) => bail!("data: {}, error: {}", event.data, e),
                };
            }
        }
        Ok(ret)
    }

    /// Sends a request to upload files to the Dify API and returns the response.  
    /// 上传文件（目前仅支持图片）并在发送消息时使用，可实现图文多模态理解。  
    /// 支持 png, jpg, jpeg, webp, gif 格式。  
    /// 上传的文件仅供当前终端用户使用。  
    ///
    /// # Arguments
    /// * `req_data` - The files upload request data.
    ///
    /// # Returns
    /// A `Result` containing the files upload response or an error.
    pub async fn files_upload(&self, req_data: FilesUploadRequest) -> Result<FilesUploadResponse> {
        if !infer::is_image(&req_data.file) {
            bail!("FilesUploadRequest.File Illegal");
        }
        let kind = infer::get(&req_data.file).expect("Failed to get file type");
        let file_part = multipart::Part::stream(req_data.file)
            .file_name(format!("image_file.{}", kind.extension()))
            .mime_str(kind.mime_type())?;
        let form = multipart::Form::new()
            .text("user", req_data.user)
            .part("file", file_part);

        let url = self.build_request_api(ApiPath::FilesUpload);
        let req = self.client.create_multipart_request(url, form)?;
        let resp = self.send(req).await?;
        let text = resp.text().await?;
        parse_response::<FilesUploadResponse>(&text)
    }

    /// Sends a request to stop stream task from the Dify API and returns the response.
    /// 仅支持流式模式。
    ///
    /// # Arguments
    /// * `req_data` - The stream task stop request data.
    /// * `api_path` - The API path.
    ///
    /// # Returns
    /// A `Result` containing the stream task stop response or an error.
    async fn stream_task_stop(
        &self,
        mut req_data: StreamTaskStopRequest,
        api_path: ApiPath,
    ) -> Result<ResultResponse> {
        if req_data.task_id.is_empty() {
            bail!("StreamTaskStopRequest.TaskId Illegal");
        }

        let url = self.build_request_api(api_path);
        let url = url.replace("{task_id}", &req_data.task_id);

        req_data.task_id = String::new();
        let req = self.client.create_request(url, Method::POST, req_data)?;
        let resp = self.send(req).await?;
        let text = resp.text().await?;
        parse_response::<ResultResponse>(&text)
    }

    /// Sends a request to stop stream chat messages to the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The chat message stop request data.
    ///
    /// # Returns
    /// A `Result` containing the chat message stop response or an error.
    pub async fn chat_messages_stop(
        &self,
        req_data: StreamTaskStopRequest,
    ) -> Result<ResultResponse> {
        self.stream_task_stop(req_data, ApiPath::ChatMessagesStop)
            .await
    }

    /// Sends a request to retrieve suggested messages from the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The messages suggested request data.
    ///
    /// # Returns
    /// A `Result` containing the messages suggested response or an error.
    pub async fn messages_suggested(
        &self,
        mut req_data: MessagesSuggestedRequest,
    ) -> Result<MessagesSuggestedResponse> {
        if req_data.message_id.is_empty() {
            bail!("MessagesSuggestedRequest.MessageID Illegal");
        }

        let url = self.build_request_api(ApiPath::MessagesSuggested);
        let url = url.replace("{message_id}", &req_data.message_id);

        req_data.message_id = String::new();
        let req = self.client.create_request(url, Method::GET, req_data)?;
        let resp = self.send(req).await?;
        let text = resp.text().await?;
        parse_response::<MessagesSuggestedResponse>(&text)
    }

    /// Sends a request to retrieve messages feedbacks from the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The messages feedbacks request data.
    ///
    /// # Returns
    /// A `Result` containing the messages feedbacks response or an error.
    pub async fn messages_feedbacks(
        &self,
        mut req_data: MessagesFeedbacksRequest,
    ) -> Result<ResultResponse> {
        if req_data.message_id.is_empty() {
            bail!("MessagesFeedbacksRequest.MessageID Illegal");
        }

        let url = self.build_request_api(ApiPath::MessagesFeedbacks);
        let url = url.replace("{message_id}", &req_data.message_id);

        req_data.message_id = String::new();
        let req = self.client.create_request(url, Method::POST, req_data)?;
        let resp = self.send(req).await?;
        let text = resp.text().await?;
        parse_response::<ResultResponse>(&text)
    }

    /// Sends a request to retrieve conversations from the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The conversations request data.
    ///
    /// # Returns
    /// A `Result` containing the conversations response or an error.
    pub async fn conversations(
        &self,
        req_data: ConversationsRequest,
    ) -> Result<ConversationsResponse> {
        if req_data.user.is_empty() {
            bail!("ConversationsRequest.User Illegal");
        }

        let url = self.build_request_api(ApiPath::Conversations);
        let req = self.client.create_request(url, Method::GET, req_data)?;
        let resp = self.send(req).await?;
        let text = resp.text().await?;
        parse_response::<ConversationsResponse>(&text)
    }

    /// Sends a request to retrieve history messages from the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The messages request data.
    ///
    /// # Returns
    /// A `Result` containing the messages response or an error.
    pub async fn messages(&self, req_data: MessagesRequest) -> Result<MessagesResponse> {
        if req_data.conversation_id.is_empty() {
            bail!("MessagesRequest.ConversationID Illegal");
        }

        let url = self.build_request_api(ApiPath::Messages);
        let req = self.client.create_request(url, Method::GET, req_data)?;
        let resp = self.send(req).await?;
        let text = resp.text().await?;
        parse_response::<MessagesResponse>(&text)
    }

    /// Sends a request to rename a conversation in the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The conversations rename request data.
    ///
    /// # Returns
    /// A `Result` containing the conversations rename response or an error.
    pub async fn conversations_renaming(
        &self,
        mut req_data: ConversationsRenameRequest,
    ) -> Result<ResultResponse> {
        if req_data.conversation_id.is_empty() {
            bail!("ConversationsRenameRequest.ConversationID Illegal");
        }
        if req_data.auto_generate && req_data.name.is_none() {
            bail!("ConversationsRenameRequest.Name Illegal");
        }

        let url = self.build_request_api(ApiPath::ConversationsRename);
        let url = url.replace("{conversation_id}", &req_data.conversation_id);

        req_data.conversation_id = String::new();
        let req = self.client.create_request(url, Method::POST, req_data)?;
        let resp = self.send(req).await?;
        let text = resp.text().await?;
        parse_response::<ResultResponse>(&text)
    }

    /// Sends a request to delete a conversation in the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The conversations delete request data.
    ///
    /// # Returns
    /// A `Result` containing the conversations delete response or an error.
    pub async fn conversations_delete(
        &self,
        mut req_data: ConversationsDeleteRequest,
    ) -> Result<()> {
        if req_data.conversation_id.is_empty() {
            bail!("ConversationsDeleteRequest.ConversationID Illegal");
        }

        let url = self.build_request_api(ApiPath::ConversationsDelete);
        let url = url.replace("{conversation_id}", &req_data.conversation_id);

        req_data.conversation_id = String::new();
        let req = self.client.create_request(url, Method::DELETE, req_data)?;
        let resp = self.send(req).await?;
        // http 204 means success ?
        if resp.status().as_u16() == 204 {
            Ok(())
        } else {
            // parse message type
            let text = resp.text().await?;
            parse_error_response(&text)
        }
    }

    /// Sends a request to convert audio to text in the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The audio to text request data.
    ///
    /// # Returns
    /// A `Result` containing the audio to text response or an error.
    pub async fn text_to_audio(&self, req_data: TextToAudioRequest) -> Result<Bytes> {
        if req_data.text.is_empty() {
            bail!("TextToAudioRequest.Text Illegal");
        }

        let url = self.build_request_api(ApiPath::TextToAudio);
        let req = self.client.create_request(url, Method::POST, req_data)?;
        let resp = self.send(req).await?;
        // check if header is audio
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE);
        let content_type = content_type
            .ok_or(anyhow::anyhow!("Content-Type is missing"))?
            .to_str()?;
        // check if content_type is audio
        if content_type.starts_with("audio/") {
            let bytes = resp.bytes().await?;
            return Ok(bytes);
        }
        let text = resp.text().await?;
        parse_error_response(&text)
    }

    /// Sends a request to convert audio to text in the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The audio to text request data.
    ///
    /// # Returns
    /// A `Result` containing the audio to text response or an error.
    pub async fn audio_to_text(&self, req_data: AudioToTextRequest) -> Result<AudioToTextResponse> {
        if !infer::is_audio(&req_data.file) {
            bail!("AudioToTextRequest.File Illegal");
        }
        let kind = infer::get(&req_data.file).expect("Failed to get file type");
        let file_part = multipart::Part::stream(req_data.file)
            .file_name(format!("audio_file.{}", kind.extension()))
            .mime_str(kind.mime_type())?;
        let form = multipart::Form::new()
            .text("user", req_data.user)
            .part("file", file_part);

        let url = self.build_request_api(ApiPath::AudioToText);
        let req = self.client.create_multipart_request(url, form)?;
        let resp = self.send(req).await?;
        let text = resp.text().await?;
        parse_response::<AudioToTextResponse>(&text)
    }

    /// Sends a request to retrieve parameters from the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The parameters request data.
    ///
    /// # Returns
    /// A `Result` containing the parameters response or an error.
    pub async fn parameters(&self, req_data: ParametersRequest) -> Result<ParametersResponse> {
        if req_data.user.is_empty() {
            bail!("ParametersRequest.User Illegal");
        }

        let url = self.build_request_api(ApiPath::Parameters);
        let req = self.client.create_request(url, Method::GET, req_data)?;
        let resp = self.send(req).await?;
        let text = resp.text().await?;
        parse_response::<ParametersResponse>(&text)
    }

    /// Sends a request to retrieve meta information from the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The meta request data.
    ///
    /// # Returns
    /// A `Result` containing the meta response or an error.
    pub async fn meta(&self, req_data: MetaRequest) -> Result<MetaResponse> {
        if req_data.user.is_empty() {
            bail!("MetaRequest.User Illegal");
        }

        let url = self.build_request_api(ApiPath::Meta);
        let req = self.client.create_request(url, Method::GET, req_data)?;
        let resp = self.send(req).await?;
        let text = resp.text().await?;
        parse_response::<MetaResponse>(&text)
    }

    /// Creates a request to run workflows from the Dify API.
    ///
    /// # Arguments
    /// * `req` - The workflows run request data.
    ///     
    /// # Returns
    /// A `Result` containing the request or an error.
    fn create_workflows_run_request(&self, req: WorkflowsRunRequest) -> Result<Request> {
        let url = self.build_request_api(ApiPath::WorkflowsRun);
        self.client.create_request(url, Method::POST, req)
    }

    /// Sends a request to run workflows from the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The workflows run request data.
    ///
    /// # Returns
    /// A `Result` containing the workflows run response or an error.
    pub async fn workflows_run(
        &self,
        mut req_data: WorkflowsRunRequest,
    ) -> Result<WorkflowsRunResponse> {
        req_data.response_mode = ResponseMode::Blocking;

        let req = self.create_workflows_run_request(req_data)?;
        let resp = self.send(req).await?;
        let text = resp.text().await?;
        parse_response::<WorkflowsRunResponse>(&text)
    }

    /// Sends a request to run workflows from the Dify API and returns the response as a stream.
    /// The callback function is called for each event in the stream.
    /// The callback function should return `Some(T)` if the event is processed successfully, otherwise `None`.
    /// The function returns a vector of the processed events.
    /// The stream is stopped when the callback function returns an error or the stream ends.
    ///
    /// # Arguments
    /// * `req_data` - The workflows run request data.
    /// * `callback` - The callback function to process the stream events.
    ///
    /// # Returns
    /// A `Result` containing the processed events or an error.
    ///
    /// # Errors
    /// Returns an error if the request cannot be created or the stream fails.
    pub async fn workflows_run_stream<F, T>(
        &self,
        mut req_data: WorkflowsRunRequest,
        callback: F,
    ) -> Result<Vec<T>>
    where
        F: Fn(SseMessageEvent) -> Result<Option<T>> + Send + Sync,
    {
        req_data.response_mode = ResponseMode::Streaming;

        let req = self.create_workflows_run_request(req_data)?;
        let resp = self.send(req).await?;
        let mut stream = resp.bytes_stream().eventsource();

        let mut ret: Vec<T> = Vec::new();
        while let Some(event) = stream.next().await {
            let event = event?;
            if event.event == "message" {
                match serde_json::from_str::<SseMessageEvent>(&event.data) {
                    Ok(msg_event) => {
                        if let Some(answer) = callback(msg_event)? {
                            ret.push(answer);
                        }
                    }
                    Err(e) => bail!("data: {}, error: {}", event.data, e),
                };
            }
        }
        Ok(ret)
    }

    /// Sends a request to stop stream workflows from the Dify API and returns the response.
    ///
    /// # Arguments
    /// * `req_data` - The stream task stop request data.
    ///
    /// # Returns
    /// A `Result` containing the stream task stop response or an error.
    pub async fn workflows_stop(&self, req_data: StreamTaskStopRequest) -> Result<ResultResponse> {
        self.stream_task_stop(req_data, ApiPath::WorkflowsStop)
            .await
    }

    /// Creates a request to create completion messages from the Dify API.
    ///
    /// # Arguments
    /// * `req` - The completion messages request data.
    ///
    /// # Returns
    /// A `Result` containing the request or an error.
    fn create_completion_messages_request(
        &self,
        req: CompletionMessagesRequest,
    ) -> Result<Request> {
        let url = self.build_request_api(ApiPath::CompletionMessages);
        self.client.create_request(url, Method::POST, req)
    }

    /// Sends a request to create completion messages from the Dify API and returns the response.
    /// 发送请求给文本生成型应用
    ///
    /// # Arguments
    /// * `req_data` - The completion messages request data.
    ///
    /// # Returns
    /// A `Result` containing the completion messages response or an error.
    pub async fn completion_messages(
        &self,
        mut req_data: CompletionMessagesRequest,
    ) -> Result<CompletionMessagesResponse> {
        req_data.response_mode = ResponseMode::Blocking;

        let req = self.create_completion_messages_request(req_data)?;
        let resp = self.send(req).await?;
        let text = resp.text().await?;
        parse_response::<CompletionMessagesResponse>(&text)
    }

    /// Sends a request to create completion messages from the Dify API and returns the response as a stream.
    /// The callback function is called for each event in the stream.
    /// The callback function should return `Some(T)` if the event is processed successfully, otherwise `None`.
    /// The function returns a vector of the processed events.
    /// The stream is stopped when the callback function returns an error or the stream ends.
    ///
    /// # Arguments
    /// * `req_data` - The completion messages request data.
    /// * `callback` - The callback function to process the stream events.
    ///
    /// # Returns
    /// A `Result` containing the processed events or an error.
    ///
    /// # Errors
    /// Returns an error if the request cannot be created or the stream fails.
    pub async fn completion_messages_stream<F, T>(
        &self,
        mut req_data: CompletionMessagesRequest,
        callback: F,
    ) -> Result<Vec<T>>
    where
        F: Fn(SseMessageEvent) -> Result<Option<T>> + Send + Sync,
    {
        req_data.response_mode = ResponseMode::Streaming;

        let req = self.create_completion_messages_request(req_data)?;
        let resp = self.send(req).await?;
        let mut stream = resp.bytes_stream().eventsource();

        let mut ret: Vec<T> = Vec::new();
        while let Some(event) = stream.next().await {
            let event = event?;
            if event.event == "message" {
                match serde_json::from_str::<SseMessageEvent>(&event.data) {
                    Ok(msg_event) => {
                        if let Some(answer) = callback(msg_event)? {
                            ret.push(answer);
                        }
                    }
                    Err(e) => bail!("data: {}, error: {}", event.data, e),
                };
            }
        }
        Ok(ret)
    }

    /// Sends a request to stop stream completion messages from the Dify API and returns the response.
    /// 文本生成停止响应
    ///
    /// # Arguments
    /// * `req_data` - The stream task stop request data.
    ///
    /// # Returns
    /// A `Result` containing the stream task stop response or an error.
    pub async fn completion_messages_stop(
        &self,
        req_data: StreamTaskStopRequest,
    ) -> Result<ResultResponse> {
        self.stream_task_stop(req_data, ApiPath::CompletionMessagesStop)
            .await
    }
}
