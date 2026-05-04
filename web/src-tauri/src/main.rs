#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use evt_grpc_api::proto::message_service_client::MessageServiceClient;
use evt_grpc_api::proto::{
    ListLegacyMessagesRequest, MarkReadRequest, MessageActor, SendLegacyWhisperRequest,
    UnreadCountRequest,
};
#[cfg(target_os = "macos")]
use tauri::{api::shell, CustomMenuItem, Manager, Menu, MenuEntry, MenuItem, Submenu};

#[derive(serde::Serialize)]
struct CompatPager {
    page: u64,
    page_size: u64,
    total_rows: i64,
}

#[derive(serde::Serialize)]
struct CompatUserInfo {
    id: i64,
    nickname: String,
    username: String,
    avatar: String,
    phone: String,
    activation: String,
    is_admin: bool,
    is_friend: bool,
    is_following: bool,
    created_on: i64,
    follows: i64,
    followings: i64,
    tweets_count: i64,
    balance: i64,
    status: i32,
}

#[derive(serde::Serialize)]
struct CompatMessageItem {
    id: i64,
    #[serde(rename = "type")]
    message_type: i32,
    brief: String,
    content: String,
    is_read: i32,
    sender_user_id: i64,
    sender_user: CompatUserInfo,
    receiver_user_id: i64,
    receiver_user: CompatUserInfo,
    post_id: i64,
    post: serde_json::Value,
    comment_id: i64,
    comment: serde_json::Value,
    reply_id: i64,
    replay: serde_json::Value,
    created_on: i64,
}

#[derive(serde::Serialize)]
struct CompatMessageList {
    list: Vec<CompatMessageItem>,
    pager: CompatPager,
}

#[derive(serde::Serialize)]
struct CompatUnreadCount {
    count: i64,
}

fn grpc_endpoint() -> String {
    if let Ok(endpoint) = std::env::var("EVT_TAURI_GRPC_ENDPOINT") {
        let trimmed = endpoint.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let host = std::env::var("EVT_RS__SERVER__GRPC__HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("EVT_RS__SERVER__GRPC__PORT").unwrap_or_else(|_| "18020".into());
    format!("http://{host}:{port}")
}

async fn grpc_client() -> Result<MessageServiceClient<tonic::transport::Channel>, String> {
    MessageServiceClient::connect(grpc_endpoint())
        .await
        .map_err(|error| format!("connect grpc message service failed: {error}"))
}

fn grpc_error(status_code: i32, error_message: String) -> String {
    if error_message.trim().is_empty() {
        format!("grpc message service returned status {status_code}")
    } else {
        error_message
    }
}

fn map_user(user: Option<evt_grpc_api::proto::LegacyUser>) -> CompatUserInfo {
    match user {
        Some(user) => CompatUserInfo {
            id: user.id,
            nickname: user.nickname,
            username: user.username,
            avatar: user.avatar,
            phone: String::new(),
            activation: String::new(),
            is_admin: false,
            is_friend: false,
            is_following: user.is_following,
            created_on: user.created_on,
            follows: 0,
            followings: 0,
            tweets_count: 0,
            balance: 0,
            status: 1,
        },
        None => CompatUserInfo {
            id: 0,
            nickname: String::new(),
            username: String::new(),
            avatar: String::new(),
            phone: String::new(),
            activation: String::new(),
            is_admin: false,
            is_friend: false,
            is_following: false,
            created_on: 0,
            follows: 0,
            followings: 0,
            tweets_count: 0,
            balance: 0,
            status: 1,
        },
    }
}

#[tauri::command]
async fn grpc_list_legacy_messages(
    bearer_token: String,
    style: String,
    page: u64,
    page_size: u64,
) -> Result<CompatMessageList, String> {
    let mut client = grpc_client().await?;
    let response = client
        .list_legacy_messages(ListLegacyMessagesRequest {
            bearer_token,
            style,
            page,
            page_size,
        })
        .await
        .map_err(|error| format!("list legacy messages grpc request failed: {error}"))?
        .into_inner();

    if response.status_code != 0 {
        return Err(grpc_error(response.status_code, response.error_message));
    }

    Ok(CompatMessageList {
        list: response
            .items
            .into_iter()
            .map(|item| CompatMessageItem {
                id: item.id,
                message_type: item.r#type,
                brief: item.brief,
                content: item.content,
                is_read: item.is_read,
                sender_user_id: item.sender_user_id,
                sender_user: map_user(item.sender_user),
                receiver_user_id: item.receiver_user_id,
                receiver_user: map_user(item.receiver_user),
                post_id: item.post_id,
                post: if item.post_id > 0 {
                    serde_json::json!({ "id": item.post_id })
                } else {
                    serde_json::json!({})
                },
                comment_id: item.comment_id,
                comment: if item.comment_id > 0 {
                    serde_json::json!({ "id": item.comment_id })
                } else {
                    serde_json::json!({})
                },
                reply_id: item.reply_id,
                replay: serde_json::json!({}),
                created_on: item.created_on,
            })
            .collect(),
        pager: CompatPager {
            page: response.page,
            page_size: response.page_size,
            total_rows: response.total,
        },
    })
}

#[tauri::command]
async fn grpc_legacy_unread_count(bearer_token: String) -> Result<CompatUnreadCount, String> {
    let mut client = grpc_client().await?;
    let response = client
        .legacy_unread_count(UnreadCountRequest { bearer_token })
        .await
        .map_err(|error| format!("legacy unread count grpc request failed: {error}"))?
        .into_inner();

    if response.status_code != 0 {
        return Err(grpc_error(response.status_code, response.error_message));
    }

    Ok(CompatUnreadCount {
        count: response.unread_count,
    })
}

#[tauri::command]
async fn grpc_mark_message_read(bearer_token: String, message_id: i64) -> Result<(), String> {
    let mut client = grpc_client().await?;
    let response = client
        .mark_read(MarkReadRequest {
            bearer_token,
            message_id,
        })
        .await
        .map_err(|error| format!("mark read grpc request failed: {error}"))?
        .into_inner();

    if response.status_code != 0 {
        return Err(grpc_error(response.status_code, response.error_message));
    }

    Ok(())
}

#[tauri::command]
async fn grpc_mark_all_messages_read(bearer_token: String) -> Result<(), String> {
    let mut client = grpc_client().await?;
    let response = client
        .mark_all_read(MessageActor { bearer_token })
        .await
        .map_err(|error| format!("mark all read grpc request failed: {error}"))?
        .into_inner();

    if response.status_code != 0 {
        return Err(grpc_error(response.status_code, response.error_message));
    }

    Ok(())
}

#[tauri::command]
async fn grpc_send_legacy_whisper(
    bearer_token: String,
    user_id: i64,
    content: String,
) -> Result<(), String> {
    let mut client = grpc_client().await?;
    let response = client
        .send_legacy_whisper(SendLegacyWhisperRequest {
            bearer_token,
            user_id,
            content,
        })
        .await
        .map_err(|error| format!("send legacy whisper grpc request failed: {error}"))?
        .into_inner();

    if response.status_code != 0 {
        return Err(grpc_error(response.status_code, response.error_message));
    }

    Ok(())
}

fn main() {
    let _ctx = tauri::generate_context!();

    #[cfg(target_os = "macos")]
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            grpc_list_legacy_messages,
            grpc_legacy_unread_count,
            grpc_mark_message_read,
            grpc_mark_all_messages_read,
            grpc_send_legacy_whisper,
        ])
        .menu(Menu::with_items([
            MenuEntry::Submenu(Submenu::new(
                &_ctx.package_info().name,
                Menu::with_items([
                    MenuItem::Separator.into(),
                    MenuItem::Services.into(),
                    MenuItem::Separator.into(),
                    MenuItem::Hide.into(),
                    MenuItem::HideOthers.into(),
                    MenuItem::ShowAll.into(),
                    MenuItem::Separator.into(),
                    MenuItem::Quit.into(),
                ]),
            )),
            MenuEntry::Submenu(Submenu::new(
                "Window",
                Menu::with_items([MenuItem::Minimize.into(), MenuItem::Zoom.into()]),
            )),
            MenuEntry::Submenu(Submenu::new(
                "Help",
                Menu::with_items([CustomMenuItem::new("Learn More", "Learn More").into()]),
            )),
        ]))
        .on_menu_event(|event| {
            let event_name = event.menu_item_id();
            event.window().emit("menu", event_name).unwrap();
            match event_name {
                "Learn More" => {
                    let link = "https://github.com/Cricle/Evt".to_string();
                    shell::open(&event.window().shell_scope(), link, None).unwrap();
                }
                _ => {}
            }
        });

    #[cfg(not(target_os = "macos"))]
    let app = tauri::Builder::default().invoke_handler(tauri::generate_handler![
        grpc_list_legacy_messages,
        grpc_legacy_unread_count,
        grpc_mark_message_read,
        grpc_mark_all_messages_read,
        grpc_send_legacy_whisper,
    ]);

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
