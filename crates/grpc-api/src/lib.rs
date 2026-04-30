use std::sync::Arc;

use evt_infra::AppContext;
use tonic::{Request, Response, Status};

pub mod proto {
    tonic::include_proto!("core.v1");
}

use proto::authenticate_service_server::{AuthenticateService, AuthenticateServiceServer};
use proto::{ActionReply, LoginReply, User};

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

pub struct AuthenticateGrpc {
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
