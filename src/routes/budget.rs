use axum::{
    routing::{get},
    Router,
};

use crate::handlers::budget::{
    get_budgets, get_budget_by_id, create_budget, update_budget, delete_budget,
    get_budget_summary, get_budget_performance, get_budget_categories, get_budget_suggestions
};
use crate::middleware::auth_middleware;
use crate::services::BudgetService;
use crate::repositories::PostgresBudgetRepository;

pub fn budget_routes() -> Router<BudgetService<PostgresBudgetRepository>> {
    Router::new()
        .route("/budgets", get(get_budgets).post(create_budget))
        .route("/budgets/{id}", get(get_budget_by_id).put(update_budget).delete(delete_budget))
        .route("/budgets/summary", get(get_budget_summary))
        .route("/budgets/performance", get(get_budget_performance))
        .route("/budgets/categories", get(get_budget_categories))
        .route("/budgets/suggestions", get(get_budget_suggestions))
        .layer(axum::middleware::from_fn(auth_middleware))
}