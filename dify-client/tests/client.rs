use anyhow::Result;
use dify_client::{request, response, Client, Config};
use std::{collections::HashMap, env, time::Duration};

#[test]
fn test_config() {
    let config = Config {
        base_url: "https://api.dify.ai".into(),
        api_key: "API_KEY".into(),
        timeout: Duration::from_secs(30),
    };
    assert_eq!(config.base_url, "https://api.dify.ai");
    assert_eq!(config.api_key, "API_KEY");
    assert_eq!(config.timeout, Duration::from_secs(30));
}

#[test]
fn test_new_client() {
    let client = Client::new("https://api.dify.ai".into(), "API_KEY".into());
    assert_eq!(client.config.base_url, "https://api.dify.ai");
    assert_eq!(client.config.api_key, "API_KEY");
    assert_eq!(client.config.timeout, Duration::from_secs(30));
}

#[test]
fn test_new_client_with_config() {
    let config = Config {
        base_url: "https://api.dify.ai".into(),
        api_key: "API_KEY".into(),
        timeout: Duration::from_secs(60),
    };
    let client = Client::new_with_config(config);
    assert_eq!(client.config.base_url, "https://api.dify.ai");
    assert_eq!(client.config.api_key, "API_KEY");
    assert_eq!(client.config.timeout, Duration::from_secs(60));
}

fn get_client(api_key: Option<&str>) -> Client {
    let dify_base_url = env::var("DIFY_BASE_URL").expect("DIFY_BASE_URL is not set");
    let dify_api_key = env::var("DIFY_API_KEY").expect("DIFY_API_KEY is not set");
    let dify_api_key = api_key.unwrap_or(dify_api_key.as_str());
    Client::new_with_config(Config {
        base_url: dify_base_url,
        api_key: dify_api_key.to_owned(),
        timeout: Duration::from_secs(60),
    })
}

fn get_client_default() -> Client {
    get_client(None)
}

#[tokio::test]
async fn test_chat_message_complex() {
    let client = get_client_default();
    let msg = request::ChatMessageRequest {
        inputs: HashMap::from([("name".into(), "iPhone 13 Pro Max".into())]),
        query: "What are the specs of the iPhone 13 Pro Max?".into(),
        response_mode: request::ResponseMode::Blocking,
        conversation_id: "".into(),
        user: "afa".into(),
        files: vec![request::ChatMessageFile::RemoteUrl {
            url: "https://www.baidu.com/img/PCfb_5bf082d29588c07f842ccde3f97243ea.png".into(),
            type_: request::FileType::Image,
        }],
        auto_generate_name: true,
    };
    let result = client.chat_messages(msg).await;
    assert_chat_message_result(result);
}

#[tokio::test]
async fn test_chat_message_simple() {
    let client = get_client_default();
    let msg = request::ChatMessageRequest {
        query: "how are you?".into(),
        user: "afa".into(),
        ..Default::default()
    };
    let result = client.chat_messages(msg).await;
    assert_chat_message_result(result);
}

#[tokio::test]
async fn test_chat_message_stream() {
    let client = get_client_default();
    let msg = request::ChatMessageRequest {
        query: "写一段关于人性的500字小作文".into(),
        user: "afa".into(),
        ..Default::default()
    };

    let result = client
        .chat_messages_stream(msg, |e| {
            println!("{:?}", e);
            match e {
                response::SteamMessageEvent::Message { answer, .. } => {
                    return Ok(Some(answer));
                }
                _ => Ok(None),
            }
        })
        .await;
    // println!("{:?}", result);
    assert!(result.is_ok());
    let answers = result.unwrap();
    let answer = answers.concat();
    println!("{:?}", answer);
}

fn assert_chat_message_result(result: Result<response::ChatMessageResponse>) {
    if let Err(e) = result {
        match e.downcast::<response::ErrorResponse>() {
            Ok(err_resp) => {
                assert!(!err_resp.message.is_empty());
            }
            Err(e_self) => {
                assert!(!e_self.to_string().is_empty());
            }
        };
    } else {
        let response = result.unwrap();
        println!("{:}", serde_json::to_string_pretty(&response).unwrap());
        assert_eq!(response.event, "message");
        assert_eq!(response.mode, response::AppMode::AdvancedChat);
        assert!(!response.base.message_id.is_empty());
        assert!(!response.base.conversation_id.is_empty());
    }
}

#[tokio::test]
async fn test_feedback_message() {
    let client = get_client_default();
    let msg = request::MessagesFeedbacksRequest {
        message_id: "e754aaf1-d2a3-426a-a9cc-39c508ccfe86".into(),
        rating: Some(request::Feedback::Like),
        user: "afa".into(),
    };
    let result = client.messages_feedbacks(msg).await;
    assert_feedback_result(result);

    let msg1 = request::MessagesFeedbacksRequest {
        message_id: "e754aaf1-d2a3-426a-a9cc-39c508ccfe86".into(),
        rating: None,
        user: "afa".into(),
    };
    let result1 = client.messages_feedbacks(msg1).await;
    assert_feedback_result(result1);

    let msg2 = request::MessagesFeedbacksRequest {
        message_id: "e754aaf1-d2a3-426a-a9cc-39c508ccfe86".into(),
        rating: Some(request::Feedback::Dislike),
        user: "afa".into(),
    };
    let result2 = client.messages_feedbacks(msg2).await;
    assert_feedback_result(result2);
}

fn assert_feedback_result(result: Result<response::ResultResponse>) {
    if let Err(e) = result {
        match e.downcast::<response::ErrorResponse>() {
            Ok(err_resp) => {
                assert!(!err_resp.message.is_empty());
            }
            Err(e_self) => {
                assert!(!e_self.to_string().is_empty());
            }
        };
    } else {
        let response = result.unwrap();
        assert_eq!(response.result, "success");
    }
}

#[tokio::test]
async fn test_conversations_get() {
    let client = get_client_default();
    let msg = request::ConversationsRequest {
        user: "afa".into(),
        ..Default::default()
    };
    let result = client.conversations(msg).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    println!("{:}", serde_json::to_string_pretty(&response).unwrap());
    assert!(response.data.len() > 0);
}

#[tokio::test]
async fn test_messages_get() {
    let client = get_client_default();
    let msg = request::MessagesRequest {
        conversation_id: "45000310-eb4a-480b-ba90-3f658e87bc6a".into(),
        user: "afa".into(),
        ..Default::default()
    };
    let result = client.messages(msg).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    println!("{:}", serde_json::to_string_pretty(&response).unwrap());
    assert!(response.data.len() > 0);
}

#[tokio::test]
async fn test_parameters() {
    let client = get_client_default();
    let msg = request::ParametersRequest { user: "afa".into() };
    let result = client.parameters(msg).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    println!("{:}", serde_json::to_string_pretty(&response).unwrap());
    assert!(response.system_parameters.image_file_size_limit.len() > 0);
}

#[tokio::test]
async fn test_chat_messages_stop() {
    let client = get_client_default();
    let msg = request::StreamTaskStopRequest {
        task_id: "task_id".into(),
        user: "afa".into(),
    };
    let result = client.chat_messages_stop(msg).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    println!("{:}", serde_json::to_string_pretty(&response).unwrap());
}

#[tokio::test]
async fn test_messages_suggested() {
    let client = get_client_default();
    let msg = request::MessagesSuggestedRequest {
        message_id: "6f52aa16-e102-405c-a6a8-058e8c900058".into(),
    };
    let result = client.messages_suggested(msg).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    println!("{:}", serde_json::to_string_pretty(&response).unwrap());
}

#[tokio::test]
async fn test_conversations_delete() {
    let client = get_client_default();
    let msg = request::ConversationsDeleteRequest {
        conversation_id: "40d530ea-f743-4c7a-9639-bbdae4ef6e6d".into(),
        user: "afa".into(),
    };
    let result = client.conversations_delete(msg).await;
    println!("{:?}", result);
    // assert!(result.is_ok());
    // let response = result.unwrap();
    // println!("{:}", serde_json::to_string_pretty(&response).unwrap());
}

#[tokio::test]
async fn test_meta() {
    let client = get_client(Some("app-iTiQkNf5LUbMq0mG0QdxXTob"));
    let msg = request::MetaRequest { user: "afa".into() };
    let result = client.meta(msg).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    println!("{:}", serde_json::to_string_pretty(&response).unwrap());
}

#[tokio::test]
async fn test_text_to_audio() {
    let client = get_client_default();
    let msg = request::TextToAudioRequest {
        text: "Hello, dify client!".into(),
        user: "afa".into(),
        ..Default::default()
    };
    let result = client.text_to_audio(msg).await;
    assert!(result.is_ok());
    let bytes = result.unwrap();
    // write bytes to /tmp/test.mp3
    std::fs::write("/tmp/test.mp3", bytes).expect("write file failed");
}

#[tokio::test]
async fn test_audio_to_text() {
    let client = get_client_default();
    let vec_u8 = std::fs::read("/tmp/test.mp3").expect("read file failed");

    let msg = request::AudioToTextRequest {
        file: vec_u8.into(),
        user: "afa".into(),
    };
    let result = client.audio_to_text(msg).await;
    println!("{:?}", result);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_files_upload() {
    let client = get_client_default();
    let vec_u8 = std::fs::read("/tmp/test.png").expect("read file failed");

    let msg = request::FilesUploadRequest {
        file: vec_u8.into(),
        user: "afa".into(),
    };
    let result = client.files_upload(msg).await;
    println!("{:?}", result);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_workflows_run() {
    let client = get_client(Some("app-hxBGNNbzVsl46o20NPvSYOxB"));
    let s = r#"4月29日，外交部发言人林剑主持例行记者会。
有记者提问两个问题，第一个，马斯克访华后，国内会否允许特斯拉的完全自动驾驶科技（FSD）在国内运行？第二，请问您能否确认，中国气候变化事务特使刘振民将于下周访问美国华盛顿？
林剑回应，关于你提到的第一个问题，中国国务院总理李强会见了马斯克，中国有关机构也同马斯克进行了相关的会谈，中方已经发布了消息稿，你可以查阅。作为原则，我愿再次重申，中国致力于高质量发展，坚定奉行合作、共赢的开放战略，持续建设市场化、法治化、国际化的一流营商环境，积极促进外商投资，高度重视外资企业的相关诉求，切实保护外资企业的合法权益。我们对内外资企业一视同仁、平等相待，欢迎外资企业继续深化对华合作，共享中国经济发展成果，也希望外资企业遵守中国的法律法规，履行安全承诺。
他表示，关于你提到的第二个问题，中国气候变化事务特使与美国总统国际气候政策高级顾问一直保持着密切沟通。至于你所说到的访问，目前我没有更多的信息可以提供。"#;
    let msg = request::WorkflowsRunRequest {
        inputs: HashMap::from([
            ("input".into(), s.into()),
            ("summaryStyle".into(), "General Overview".into()),
        ]),
        user: "afa".into(),
        ..Default::default()
    };
    let result = client.workflows_run(msg).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    println!("{:}", serde_json::to_string_pretty(&response).unwrap());
}

#[tokio::test]
async fn test_workflows_run_stream() {
    let client = get_client(Some("app-hxBGNNbzVsl46o20NPvSYOxB"));
    let s = r#"Hi, Gu from Dify here. I couldn't be more excited to share with you our latest feature: Workflow.
We've all seen the huge potential of LLMs in the past year. But as many of you have experienced firsthand, harnessing that potential for robust, production-ready solutions comes with its own set of challenges. Workflow is our answer to that challenge -- it is designed to bridge the gap where single-prompt LLMs falter: generating predictable outputs with multi-step logic. 
Workflow is currently accessible as a standalone app type. It can also be activated in 'Chatbot' apps for building complex conversation flows (Chatflow). We can't wait for you to start experimenting with it now.
Chatflow is set to overtake "expert mode" in current Chatbot apps. You may choose to keep editing your existing apps in "expert mode", or transform them into workflows. "#;
    let msg = request::WorkflowsRunRequest {
        inputs: HashMap::from([
            ("input".into(), s.into()),
            ("summaryStyle".into(), "General Overview".into()),
        ]),
        user: "afa".into(),
        ..Default::default()
    };

    let result = client
        .workflows_run_stream(msg, |e| {
            println!("{:?}", e);
            match e {
                response::SteamMessageEvent::WorkflowFinished { data, .. } => {
                    let output = data
                        .outputs
                        .map(|o| o["output"].as_str().map(|s| s.to_owned()))
                        .flatten();
                    Ok(output)
                }
                _ => Ok(None),
            }
        })
        .await;
    println!("{:?}", result);
    assert!(result.is_ok());
    let answers = result.unwrap();
    let answer = answers.concat();
    println!("{:?}", answer);
}

#[tokio::test]
async fn test_workflows_stop() {
    let client = get_client(Some("app-hxBGNNbzVsl46o20NPvSYOxB"));
    let msg = request::StreamTaskStopRequest {
        task_id: "4ad31d44-7845-4dc3-893d-42211e800378".into(),
        user: "afa".into(),
    };
    let result = client.workflows_stop(msg).await;
    println!("{:?}", result);
    assert!(result.is_ok());
    let response = result.unwrap();
    println!("{:}", serde_json::to_string_pretty(&response).unwrap());
}
