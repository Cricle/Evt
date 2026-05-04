use std::collections::HashMap;
use std::sync::Arc;

use evt_domain::{AppError, LegacyMessageSummary, MessageSummary, UserPreview};
use evt_infra::AppContext;
use tonic::{Request, Response, Status};

pub mod proto {
    tonic::include_proto!("core.v1");
}

use proto::authenticate_service_server::{AuthenticateService, AuthenticateServiceServer};
use proto::message_service_server::{MessageService, MessageServiceServer};
use proto::{
    ActionReply, ActionStatusReply, LegacyMessageItem, LegacyUser, ListLegacyMessagesReply,
    ListLegacyMessagesRequest, ListMessagesReply, ListMessagesRequest, LoginReply, MarkReadRequest,
    MessageActor, MessageItem, MessageReply, SendLegacyWhisperRequest, SendMessageRequest,
    UnreadCountReply, UnreadCountRequest, User,
};

#[derive(Clone)]
pub struct GrpcState {
    app: Arc<AppContext>,
}

impl GrpcState {
    pub fn new(app: AppContext) -> Self {
        Self { app: Arc::new(app) }
    }
}

pub fn authenticate_service(app: AppContext) -> AuthenticateServiceServer<AuthenticateGrpc> {
    AuthenticateServiceServer::new(AuthenticateGrpc {
        state: GrpcState::new(app),
    })
}

pub fn message_service(app: AppContext) -> MessageServiceServer<MessageGrpc> {
    MessageServiceServer::new(MessageGrpc {
        state: GrpcState::new(app),
    })
}

pub struct AuthenticateGrpc {
    state: GrpcState,
}

pub struct MessageGrpc {
    state: GrpcState,
}

#[tonic::async_trait]
impl AuthenticateService for AuthenticateGrpc {
    async fn pre_login(&self, request: Request<User>) -> Result<Response<ActionReply>, Status> {
        let payload = request.into_inner();
        let code = match self.state.app.pre_login_by_phone(&payload.phone_num).await {
            Ok(_) => 0,
            Err(err) => err.code(),
        };
        Ok(Response::new(ActionReply { status_code: code }))
    }

    async fn login(&self, request: Request<User>) -> Result<Response<LoginReply>, Status> {
        let payload = request.into_inner();
        let reply = match self.state.app.login_by_phone(&payload.phone_num).await {
            Ok(result) => LoginReply {
                status_code: 0,
                token: result.token,
            },
            Err(err) => LoginReply {
                status_code: err.code(),
                token: String::new(),
            },
        };
        Ok(Response::new(reply))
    }

    async fn logout(&self, _request: Request<User>) -> Result<Response<ActionReply>, Status> {
        Ok(Response::new(ActionReply { status_code: 0 }))
    }
}

#[tonic::async_trait]
impl MessageService for MessageGrpc {
    async fn list_messages(
        &self,
        request: Request<ListMessagesRequest>,
    ) -> Result<Response<ListMessagesReply>, Status> {
        let payload = request.into_inner();
        let actor = match authenticate_grpc_request(self.state.app.as_ref(), &payload.bearer_token).await {
            Ok(actor) => actor,
            Err(err) => return Ok(Response::new(message_list_error(err))),
        };

        match self
            .state
            .app
            .list_messages(&actor, payload.page.max(1), payload.page_size.clamp(1, 100))
            .await
        {
            Ok(messages) => Ok(Response::new(ListMessagesReply {
                status_code: 0,
                items: messages.items.into_iter().map(map_message_item).collect(),
                page: messages.page,
                page_size: messages.page_size,
                total: messages.total,
                error_message: String::new(),
            })),
            Err(err) => Ok(Response::new(message_list_error(err))),
        }
    }

    async fn unread_count(
        &self,
        request: Request<UnreadCountRequest>,
    ) -> Result<Response<UnreadCountReply>, Status> {
        let payload = request.into_inner();
        let actor = match authenticate_grpc_request(self.state.app.as_ref(), &payload.bearer_token).await {
            Ok(actor) => actor,
            Err(err) => return Ok(Response::new(unread_count_error(err))),
        };

        match self.state.app.unread_message_count(&actor).await {
            Ok(unread) => Ok(Response::new(UnreadCountReply {
                status_code: 0,
                unread_count: unread.unread_count,
                error_message: String::new(),
            })),
            Err(err) => Ok(Response::new(unread_count_error(err))),
        }
    }

    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<MessageReply>, Status> {
        let payload = request.into_inner();
        let actor = match authenticate_grpc_request(self.state.app.as_ref(), &payload.bearer_token).await {
            Ok(actor) => actor,
            Err(err) => return Ok(Response::new(message_error(err))),
        };

        match self
            .state
            .app
            .send_message(&actor, &payload.receiver_username, &payload.content)
            .await
        {
            Ok(message) => Ok(Response::new(MessageReply {
                status_code: 0,
                item: Some(map_message_item(message)),
                error_message: String::new(),
            })),
            Err(err) => Ok(Response::new(message_error(err))),
        }
    }

    async fn mark_read(
        &self,
        request: Request<MarkReadRequest>,
    ) -> Result<Response<ActionStatusReply>, Status> {
        let payload = request.into_inner();
        let actor = match authenticate_grpc_request(self.state.app.as_ref(), &payload.bearer_token).await {
            Ok(actor) => actor,
            Err(err) => return Ok(Response::new(action_error(err))),
        };

        match self.state.app.mark_message_read(&actor, payload.message_id).await {
            Ok(_) => Ok(Response::new(ActionStatusReply {
                status_code: 0,
                error_message: String::new(),
            })),
            Err(err) => Ok(Response::new(action_error(err))),
        }
    }

    async fn mark_all_read(
        &self,
        request: Request<MessageActor>,
    ) -> Result<Response<ActionStatusReply>, Status> {
        let payload = request.into_inner();
        let actor = match authenticate_grpc_request(self.state.app.as_ref(), &payload.bearer_token).await {
            Ok(actor) => actor,
            Err(err) => return Ok(Response::new(action_error(err))),
        };

        match self.state.app.mark_all_messages_read(&actor).await {
            Ok(_) => Ok(Response::new(ActionStatusReply {
                status_code: 0,
                error_message: String::new(),
            })),
            Err(err) => Ok(Response::new(action_error(err))),
        }
    }

    async fn list_legacy_messages(
        &self,
        request: Request<ListLegacyMessagesRequest>,
    ) -> Result<Response<ListLegacyMessagesReply>, Status> {
        let payload = request.into_inner();
        let actor = match authenticate_grpc_request(self.state.app.as_ref(), &payload.bearer_token).await {
            Ok(actor) => actor,
            Err(err) => return Ok(Response::new(legacy_message_list_error(err))),
        };

        match self
            .state
            .app
            .list_legacy_messages(
                &actor,
                payload.style.as_str(),
                payload.page.max(1),
                payload.page_size.clamp(1, 100),
            )
            .await
        {
            Ok(messages) => {
                let user_ids = messages
                    .items
                    .iter()
                    .flat_map(|item| [item.sender_user_id, item.receiver_user_id])
                    .collect::<Vec<_>>();
                let users = self.state.app.batch_user_previews_by_ids(&user_ids).await.map_err(to_status)?;
                let following_status = self
                    .state
                    .app
                    .batch_following_status(actor.id, &user_ids)
                    .await
                    .map_err(to_status)?;

                Ok(Response::new(ListLegacyMessagesReply {
                    status_code: 0,
                    items: messages
                        .items
                        .into_iter()
                        .map(|item| map_legacy_message_item(item, &users, &following_status))
                        .collect(),
                    page: messages.page,
                    page_size: messages.page_size,
                    total: messages.total,
                    error_message: String::new(),
                }))
            }
            Err(err) => Ok(Response::new(legacy_message_list_error(err))),
        }
    }

    async fn legacy_unread_count(
        &self,
        request: Request<UnreadCountRequest>,
    ) -> Result<Response<UnreadCountReply>, Status> {
        let payload = request.into_inner();
        let actor = match authenticate_grpc_request(self.state.app.as_ref(), &payload.bearer_token).await {
            Ok(actor) => actor,
            Err(err) => return Ok(Response::new(unread_count_error(err))),
        };

        match self.state.app.unread_legacy_message_count(&actor).await {
            Ok(unread_count) => Ok(Response::new(UnreadCountReply {
                status_code: 0,
                unread_count,
                error_message: String::new(),
            })),
            Err(err) => Ok(Response::new(unread_count_error(err))),
        }
    }

    async fn send_legacy_whisper(
        &self,
        request: Request<SendLegacyWhisperRequest>,
    ) -> Result<Response<ActionStatusReply>, Status> {
        let payload = request.into_inner();
        let actor = match authenticate_grpc_request(self.state.app.as_ref(), &payload.bearer_token).await {
            Ok(actor) => actor,
            Err(err) => return Ok(Response::new(action_error(err))),
        };

        let receiver = match self.state.app.get_user_preview_by_id(payload.user_id).await {
            Ok(receiver) => receiver,
            Err(err) => return Ok(Response::new(action_error(err))),
        };
        if actor.id == receiver.id {
            return Ok(Response::new(action_error(AppError::Validation(
                "不允许给自己发送私信".into(),
            ))));
        }
        if payload.content.trim().is_empty() {
            return Ok(Response::new(action_error(AppError::Validation(
                "私信发送失败".into(),
            ))));
        }

        match self
            .state
            .app
            .send_legacy_message(
                actor.id,
                receiver.id,
                4,
                "给你发送新私信了",
                payload.content.trim(),
                0,
                0,
                0,
            )
            .await
        {
            Ok(_) => Ok(Response::new(ActionStatusReply {
                status_code: 0,
                error_message: String::new(),
            })),
            Err(err) => Ok(Response::new(action_error(err))),
        }
    }
}

async fn authenticate_grpc_request(
    app: &AppContext,
    bearer_token: &str,
) -> Result<evt_domain::UserIdentity, AppError> {
    let token = bearer_token.trim();
    if token.is_empty() {
        return Err(AppError::Unauthorized("missing bearer token".into()));
    }

    let identity = app.authenticate_token(token).await?;
    app.mark_online(identity.id);
    Ok(identity)
}

fn map_message_item(message: MessageSummary) -> MessageItem {
    MessageItem {
        id: message.id,
        sender_user_id: message.sender_user_id,
        sender_username: message.sender_username,
        receiver_user_id: message.receiver_user_id,
        receiver_username: message.receiver_username,
        content: message.content,
        is_read: message.is_read,
        created_at_unix: message.created_at.timestamp(),
    }
}

fn map_legacy_message_item(
    message: LegacyMessageSummary,
    users: &HashMap<i64, UserPreview>,
    following_status: &HashMap<i64, bool>,
) -> LegacyMessageItem {
    LegacyMessageItem {
        id: message.id,
        r#type: message.message_type,
        brief: message.brief,
        content: message.content,
        is_read: if message.is_read { 1 } else { 0 },
        sender_user_id: message.sender_user_id,
        sender_user: Some(map_legacy_user(
            users.get(&message.sender_user_id),
            message.sender_user_id,
            following_status.get(&message.sender_user_id).copied().unwrap_or(false),
        )),
        receiver_user_id: message.receiver_user_id,
        receiver_user: Some(map_legacy_user(
            users.get(&message.receiver_user_id),
            message.receiver_user_id,
            following_status
                .get(&message.receiver_user_id)
                .copied()
                .unwrap_or(false),
        )),
        post_id: message.post_id,
        comment_id: message.comment_id,
        reply_id: message.reply_id,
        created_on: message.created_at.timestamp(),
    }
}

fn map_legacy_user(user: Option<&UserPreview>, user_id: i64, is_following: bool) -> LegacyUser {
    match user {
        Some(user) => LegacyUser {
            id: user.id,
            username: user.username.clone(),
            nickname: user.nickname.clone(),
            avatar: user.avatar.clone(),
            is_following,
            created_on: user.created_at.timestamp(),
        },
        None => LegacyUser {
            id: user_id,
            username: String::new(),
            nickname: String::new(),
            avatar: String::new(),
            is_following,
            created_on: 0,
        },
    }
}

fn action_error(error: AppError) -> ActionStatusReply {
    ActionStatusReply {
        status_code: error.code(),
        error_message: error.to_string(),
    }
}

fn message_error(error: AppError) -> MessageReply {
    MessageReply {
        status_code: error.code(),
        item: None,
        error_message: error.to_string(),
    }
}

fn message_list_error(error: AppError) -> ListMessagesReply {
    ListMessagesReply {
        status_code: error.code(),
        items: Vec::new(),
        page: 0,
        page_size: 0,
        total: 0,
        error_message: error.to_string(),
    }
}

fn unread_count_error(error: AppError) -> UnreadCountReply {
    UnreadCountReply {
        status_code: error.code(),
        unread_count: 0,
        error_message: error.to_string(),
    }
}

fn legacy_message_list_error(error: AppError) -> ListLegacyMessagesReply {
    ListLegacyMessagesReply {
        status_code: error.code(),
        items: Vec::new(),
        page: 0,
        page_size: 0,
        total: 0,
        error_message: error.to_string(),
    }
}

fn to_status(error: AppError) -> Status {
    Status::internal(error.to_string())
}
