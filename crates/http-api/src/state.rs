use std::sync::Arc;

use paopao_infra::AppContext;

#[derive(Clone)]
pub struct HttpState {
    app: Arc<AppContext>,
}

impl HttpState {
    pub fn new(app: AppContext) -> Self {
        Self { app: Arc::new(app) }
    }

    pub fn app(&self) -> &AppContext {
        self.app.as_ref()
    }
}
