use paopao_domain::{
    AppError, PagedResponse, UserIdentity, WalletRechargeSummary, WalletStatementSummary,
};
use uuid::Uuid;

use crate::AppContext;

impl AppContext {
    pub async fn create_wallet_recharge(
        &self,
        actor: &UserIdentity,
        amount: i64,
    ) -> Result<WalletRechargeSummary, AppError> {
        if amount <= 0 {
            return Err(AppError::Validation("amount must be positive".into()));
        }
        self.wallet.create_recharge(actor.id, amount).await
    }

    pub async fn get_wallet_recharge(
        &self,
        actor: &UserIdentity,
        recharge_id: i64,
    ) -> Result<WalletRechargeSummary, AppError> {
        let recharge = self
            .wallet
            .find_recharge_by_id(recharge_id)
            .await?
            .ok_or_else(|| AppError::NotFound("recharge not found".into()))?;
        if recharge.user_id != actor.id {
            return Err(AppError::Unauthorized(
                "cannot read another user's recharge".into(),
            ));
        }
        Ok(recharge)
    }

    pub async fn complete_wallet_recharge(
        &self,
        actor: &UserIdentity,
        recharge_id: i64,
    ) -> Result<WalletRechargeSummary, AppError> {
        let recharge = self.get_wallet_recharge(actor, recharge_id).await?;
        if recharge.trade_status == "TRADE_SUCCESS" {
            return Ok(recharge);
        }

        let profile = self
            .profiles
            .find_by_user_id(actor.id)
            .await?
            .ok_or_else(|| AppError::NotFound("user profile not found".into()))?;
        let next_balance = profile.balance + recharge.amount;
        self.wallet
            .mark_recharge_success(recharge.id, &Uuid::new_v4().simple().to_string())
            .await?;
        self.profiles.update_balance(actor.id, next_balance).await?;
        self.wallet
            .create_statement(actor.id, recharge.amount, next_balance, "用户充值", 0)
            .await?;

        self.wallet
            .find_recharge_by_id(recharge.id)
            .await?
            .ok_or_else(|| AppError::Internal("updated recharge not found".into()))
    }

    pub async fn list_wallet_bills(
        &self,
        actor: &UserIdentity,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<WalletStatementSummary>, AppError> {
        self.wallet.list_statements(actor.id, page, page_size).await
    }
}
