use chrono::{DateTime, Utc};
use paopao_domain::{AppError, PagedResponse, WalletRechargeSummary, WalletStatementSummary};
use sqlx::{FromRow, MySqlPool};

use super::map_db_error;

#[derive(Clone)]
pub struct WalletRepository {
    pool: MySqlPool,
}

impl WalletRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create_recharge(
        &self,
        user_id: i64,
        amount: i64,
    ) -> Result<WalletRechargeSummary, AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO wallet_recharges (user_id, amount)
            VALUES (?, ?)
            "#,
        )
        .bind(user_id)
        .bind(amount)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        self.find_recharge_by_id(result.last_insert_id() as i64)
            .await?
            .ok_or_else(|| AppError::Internal("recharge cannot be loaded".into()))
    }

    pub async fn find_recharge_by_id(
        &self,
        recharge_id: i64,
    ) -> Result<Option<WalletRechargeSummary>, AppError> {
        sqlx::query_as::<_, WalletRechargeRow>(
            r#"
            SELECT id, user_id, amount, trade_no, trade_status, created_at
            FROM wallet_recharges
            WHERE id = ?
            LIMIT 1
            "#,
        )
        .bind(recharge_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    pub async fn mark_recharge_success(
        &self,
        recharge_id: i64,
        trade_no: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE wallet_recharges
            SET trade_no = ?, trade_status = 'TRADE_SUCCESS', updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(trade_no)
        .bind(recharge_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn create_statement(
        &self,
        user_id: i64,
        change_amount: i64,
        balance_snapshot: i64,
        reason: &str,
        post_id: i64,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO wallet_statements (user_id, change_amount, balance_snapshot, reason, post_id)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(change_amount)
        .bind(balance_snapshot)
        .bind(reason)
        .bind(post_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn list_statements(
        &self,
        user_id: i64,
        page: u64,
        page_size: u64,
    ) -> Result<PagedResponse<WalletStatementSummary>, AppError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM wallet_statements WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        let items = sqlx::query_as::<_, WalletStatementRow>(
            r#"
            SELECT id, user_id, change_amount, balance_snapshot, reason, post_id, created_at
            FROM wallet_statements
            WHERE user_id = ?
            ORDER BY id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?
        .into_iter()
        .map(Into::into)
        .collect();

        Ok(PagedResponse {
            items,
            total,
            page,
            page_size,
        })
    }

    pub async fn has_attachment_purchase(
        &self,
        user_id: i64,
        post_id: i64,
    ) -> Result<bool, AppError> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM attachment_purchase_records
            WHERE user_id = ? AND post_id = ?
            "#,
        )
        .bind(user_id)
        .bind(post_id)
        .fetch_one(&self.pool)
        .await
        .map(|count| count > 0)
        .map_err(map_db_error)
    }

    pub async fn purchase_attachment(
        &self,
        user_id: i64,
        author_user_id: i64,
        post_id: i64,
        amount: i64,
        author_income: i64,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;

        let existing = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id
            FROM attachment_purchase_records
            WHERE user_id = ? AND post_id = ?
            LIMIT 1
            FOR UPDATE
            "#,
        )
        .bind(user_id)
        .bind(post_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;
        if existing.is_some() {
            tx.commit().await.map_err(map_db_error)?;
            return Ok(());
        }

        let balance = sqlx::query_scalar::<_, i64>(
            "SELECT balance FROM user_profiles WHERE user_id = ? LIMIT 1 FOR UPDATE",
        )
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| AppError::NotFound("buyer wallet not found".into()))?;

        if balance < amount {
            return Err(AppError::Validation(
                "attachment download balance insufficient".into(),
            ));
        }

        let next_balance = balance - amount;
        sqlx::query("UPDATE user_profiles SET balance = ? WHERE user_id = ?")
            .bind(next_balance)
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;

        sqlx::query(
            r#"
            INSERT INTO wallet_statements (user_id, change_amount, balance_snapshot, reason, post_id)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(-amount)
        .bind(next_balance)
        .bind("购买附件支出")
        .bind(post_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

        if author_user_id > 0 && author_user_id != user_id && author_income > 0 {
            let author_balance = sqlx::query_scalar::<_, i64>(
                "SELECT balance FROM user_profiles WHERE user_id = ? LIMIT 1 FOR UPDATE",
            )
            .bind(author_user_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?
            .ok_or_else(|| AppError::NotFound("author wallet not found".into()))?;
            let next_author_balance = author_balance + author_income;

            sqlx::query("UPDATE user_profiles SET balance = ? WHERE user_id = ?")
                .bind(next_author_balance)
                .bind(author_user_id)
                .execute(&mut *tx)
                .await
                .map_err(map_db_error)?;

            sqlx::query(
                r#"
                INSERT INTO wallet_statements (user_id, change_amount, balance_snapshot, reason, post_id)
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(author_user_id)
            .bind(author_income)
            .bind(next_author_balance)
            .bind("附件收入")
            .bind(post_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;
        }

        sqlx::query(
            r#"
            INSERT INTO attachment_purchase_records (user_id, post_id, paid_amount)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(post_id)
        .bind(amount)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

        tx.commit().await.map_err(map_db_error)
    }
}

#[derive(Debug, FromRow)]
struct WalletRechargeRow {
    id: i64,
    user_id: i64,
    amount: i64,
    trade_no: String,
    trade_status: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct WalletStatementRow {
    id: i64,
    user_id: i64,
    change_amount: i64,
    balance_snapshot: i64,
    reason: String,
    post_id: i64,
    created_at: DateTime<Utc>,
}

impl From<WalletRechargeRow> for WalletRechargeSummary {
    fn from(row: WalletRechargeRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            amount: row.amount,
            trade_no: row.trade_no,
            trade_status: row.trade_status,
            created_at: row.created_at,
        }
    }
}

impl From<WalletStatementRow> for WalletStatementSummary {
    fn from(row: WalletStatementRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            change_amount: row.change_amount,
            balance_snapshot: row.balance_snapshot,
            reason: row.reason,
            post_id: row.post_id,
            created_at: row.created_at,
        }
    }
}
