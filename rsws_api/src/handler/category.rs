//! Category handler

use salvo::prelude::*;
use salvo_oapi::endpoint;
use crate::state::get_state;
use rsws_common::{ResponseExt, RswsError};
use rsws_db::category::CategoryRepository;

/// Get category list
#[endpoint(
    responses(
        (status_code = 200, description = "Category list"),
    )
)]
pub async fn list_categories(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let repo = CategoryRepository::new(state.pool());

    match repo.find_all().await {
        Ok(categories) => res.success(serde_json::json!({
            "categories": categories
        })),
        Err(e) => res.error(RswsError::Database(e)),
    }
}