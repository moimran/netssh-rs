use std::sync::Arc;

use axum::{response::Html, routing::get, Router};
use tracing::info;

use crate::config::BoardConfig;
use crate::storage::traits::Storage;

pub struct BoardService {
    _storage: Arc<dyn Storage>,
    config: BoardConfig,
}

impl BoardService {
    pub fn new(storage: Arc<dyn Storage>, config: BoardConfig) -> Self {
        Self {
            _storage: storage,
            config,
        }
    }

    pub fn routes<S>(&self) -> Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        if !self.config.enabled {
            info!("Board UI is disabled, returning empty router");
            return Router::new();
        }

        info!(
            ui_path = %self.config.ui_path,
            api_prefix = %self.config.api_prefix,
            auth_enabled = self.config.auth_enabled,
            "Creating board UI routes"
        );

        Router::new()
            .route("/", get(board_ui))
            .route("/ui", get(board_ui))
            .route("/ui/*path", get(board_ui))
    }
}

/// Serve the board UI
async fn board_ui() -> Html<&'static str> {
    info!("Serving board UI page");
    Html(include_str!("../../static/board.html"))
}
