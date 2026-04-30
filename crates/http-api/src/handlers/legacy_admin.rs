use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    auth::authenticate_request,
    handlers::legacy_access::legacy_admin_only,
    response::{ApiEnvelope, HttpApiError, success},
    state::HttpState,
};

#[derive(Debug, Serialize)]
pub struct AdminSiteStatusResponse {
    register_user_count: i64,
    online_user_count: i32,
    history_max_online: i32,
    server_up_time: i64,
}

#[derive(Debug, Serialize)]
pub struct AdminSettingsSchemaResponse {
    items: Vec<AdminSettingSchemaItem>,
}

#[derive(Debug, Serialize)]
pub struct AdminSettingsValuesResponse {
    items: Vec<AdminSettingValueItem>,
    has_pending_restart: bool,
}

#[derive(Debug, Serialize)]
pub struct AdminSettingsSaveResponse {
    items: Vec<AdminSettingValueItem>,
    updated_keys: Vec<String>,
    has_pending_restart: bool,
}

#[derive(Debug, Serialize)]
pub struct AdminSettingSchemaItem {
    key: String,
    group: String,
    section: String,
    #[serde(rename = "type")]
    value_type: String,
    label: String,
    description: String,
    apply_mode: String,
    secret: bool,
    readonly: bool,
    active: bool,
    bootstrap_value: Value,
    options: Option<Value>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AdminSettingValueItem {
    key: String,
    value: Value,
    effective_value: Value,
    source: String,
    pending_restart: bool,
    configured: bool,
    active: bool,
}

#[derive(Debug, Deserialize)]
pub struct SaveSettingsBody {
    items: Vec<SaveSettingItem>,
}

#[derive(Debug, Deserialize)]
pub struct SaveSettingItem {
    key: String,
    value: Value,
}

#[derive(Debug, Deserialize)]
pub struct AdminUserStatusBody {
    id: i64,
    status: i32,
}

#[derive(Debug, Deserialize)]
pub struct WalletRechargeBody {
    amount: i64,
}

#[derive(Debug, Deserialize)]
pub struct WalletRechargeQuery {
    id: i64,
}

#[derive(Debug, Serialize)]
pub struct WalletRechargeResponse {
    id: i64,
    pay: String,
}

#[derive(Debug, Serialize)]
pub struct WalletRechargeStatusResponse {
    id: i64,
    status: String,
}

#[derive(Debug, Serialize)]
pub struct WalletBillsResponse {
    list: Vec<WalletBillItem>,
    pager: super::legacy_users::CompatPager,
}

#[derive(Debug, Serialize)]
pub struct WalletBillItem {
    id: i64,
    reason: String,
    change_amount: i64,
    created_on: i64,
}

pub async fn admin_site_status(
    State(state): State<HttpState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<AdminSiteStatusResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let current = state.app().get_current_user(&actor).await?;
    if !current.is_admin {
        return Err(legacy_admin_only());
    }
    let register_user_count = state.app().register_user_count().await?;
    let (online_user_count, history_max_online, server_up_time) =
        state.app().admin_site_status_snapshot();
    Ok(Json(success(AdminSiteStatusResponse {
        register_user_count,
        online_user_count,
        history_max_online,
        server_up_time,
    })))
}

pub async fn admin_settings_schema(
    State(state): State<HttpState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<AdminSettingsSchemaResponse>>, HttpApiError> {
    require_admin(state.app(), &headers).await?;
    let items = state
        .app()
        .admin_settings_schema()
        .await?
        .into_iter()
        .map(|item| AdminSettingSchemaItem {
            key: item.key,
            group: item.group,
            section: item.section,
            value_type: item.value_type,
            label: item.label,
            description: item.description,
            apply_mode: item.apply_mode,
            secret: item.secret,
            readonly: item.readonly,
            active: item.active,
            bootstrap_value: item.bootstrap_value,
            options: item.options,
        })
        .collect();
    Ok(Json(success(AdminSettingsSchemaResponse { items })))
}

pub async fn admin_settings_values(
    State(state): State<HttpState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<AdminSettingsValuesResponse>>, HttpApiError> {
    require_admin(state.app(), &headers).await?;
    let items = state
        .app()
        .admin_settings_values()
        .await?
        .into_iter()
        .map(into_admin_value_item)
        .collect();
    Ok(Json(success(AdminSettingsValuesResponse {
        items,
        has_pending_restart: false,
    })))
}

pub async fn admin_settings_save(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<SaveSettingsBody>,
) -> Result<Json<ApiEnvelope<AdminSettingsSaveResponse>>, HttpApiError> {
    require_admin(state.app(), &headers).await?;
    let updated_keys = payload
        .items
        .iter()
        .map(|item| item.key.clone())
        .collect::<Vec<_>>();
    let items = payload
        .items
        .into_iter()
        .map(|item| (item.key, item.value))
        .collect::<Vec<_>>();
    let values = state.app().save_admin_settings(&items).await?;
    Ok(Json(success(AdminSettingsSaveResponse {
        items: values.into_iter().map(into_admin_value_item).collect(),
        updated_keys,
        has_pending_restart: false,
    })))
}

pub async fn admin_user_status(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<AdminUserStatusBody>,
) -> Result<Json<ApiEnvelope<serde_json::Value>>, HttpApiError> {
    require_admin(state.app(), &headers).await?;
    state
        .app()
        .update_user_status(payload.id, payload.status == 1)
        .await?;
    Ok(Json(success(serde_json::json!({}))))
}

pub async fn user_post_recharge(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(payload): Json<WalletRechargeBody>,
) -> Result<Json<ApiEnvelope<WalletRechargeResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let recharge = state
        .app()
        .create_wallet_recharge(&actor, payload.amount)
        .await?;
    Ok(Json(success(WalletRechargeResponse {
        id: recharge.id,
        pay: format!("evt://recharge/{}", recharge.id),
    })))
}

pub async fn user_get_recharge(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<WalletRechargeQuery>,
) -> Result<Json<ApiEnvelope<WalletRechargeStatusResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let recharge = state
        .app()
        .complete_wallet_recharge(&actor, query.id)
        .await?;
    Ok(Json(success(WalletRechargeStatusResponse {
        id: recharge.id,
        status: recharge.trade_status,
    })))
}

pub async fn user_wallet_bills(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Query(query): Query<super::legacy_users::CompatPageQuery>,
) -> Result<Json<ApiEnvelope<WalletBillsResponse>>, HttpApiError> {
    let actor = authenticate_request(state.app(), &headers).await?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let bills = state
        .app()
        .list_wallet_bills(&actor, page, page_size)
        .await?;
    Ok(Json(success(WalletBillsResponse {
        list: bills
            .items
            .into_iter()
            .map(|item| WalletBillItem {
                id: item.id,
                reason: item.reason,
                change_amount: item.change_amount,
                created_on: item.created_at.timestamp(),
            })
            .collect(),
        pager: super::legacy_users::CompatPager {
            page: bills.page,
            page_size: bills.page_size,
            total_rows: bills.total,
        },
    })))
}

async fn require_admin(
    app: &evt_infra::AppContext,
    headers: &HeaderMap,
) -> Result<(), HttpApiError> {
    let actor = authenticate_request(app, headers).await?;
    let current = app.get_current_user(&actor).await?;
    if !current.is_admin {
        return Err(legacy_admin_only());
    }
    Ok(())
}

fn into_admin_value_item(item: evt_domain::SiteSettingValueItem) -> AdminSettingValueItem {
    AdminSettingValueItem {
        key: item.key,
        value: item.value,
        effective_value: item.effective_value,
        source: item.source,
        pending_restart: item.pending_restart,
        configured: item.configured,
        active: item.active,
    }
}
