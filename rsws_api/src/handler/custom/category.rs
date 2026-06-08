//! 鐢ㄦ埛绔垎绫诲鐞嗗櫒

use crate::state::get_state;
use rsws_common::{ResponseExt, RswsError};
use rsws_db::category::CategoryRepository;
use salvo::prelude::*;
use salvo_oapi::endpoint;

/// Get category list (浠呮椿璺冨垎绫?
#[endpoint(
    responses(
        (status_code = 200, description = "Category list"),
    )
)]
pub async fn list_categories(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = get_state(depot);
    let repo = CategoryRepository::new(state.pool());

    match repo.find_all().await {
        Ok(categories) => {
            let tree = CategoryRepository::build_tree(&categories);
            res.success(serde_json::json!({
                "categories": categories,
                "tree": tree
            }))
        }
        Err(e) => res.error(RswsError::Database(e)),
    }
}
