use rust_decimal::Decimal;
use uuid::Uuid;

use crate::models::{AccountSummaryResponse, AccountInfo};
use crate::repositories::{PocketRepository, TransactionRepository};
use crate::utils::AppError;

#[derive(Clone)]
pub struct AccountSummaryService<P: PocketRepository, T: TransactionRepository> {
    pocket_repository: P,
    transaction_repository: T,
}

impl<P: PocketRepository, T: TransactionRepository> AccountSummaryService<P, T> {
    pub fn new(pocket_repository: P, transaction_repository: T) -> Self {
        Self {
            pocket_repository,
            transaction_repository,
        }
    }

    pub async fn get_account_summary(&self, user_id: Uuid) -> Result<AccountSummaryResponse, AppError> {
        // Get all pockets (accounts) for the user
        let pockets = self.pocket_repository.find_by_user_id(user_id).await?;
        
        let mut total_balance = Decimal::new(0, 0);
        let mut accounts = Vec::new();

        for pocket in pockets {
            total_balance += pocket.balance;
            accounts.push(AccountInfo {
                id: pocket.id.to_string(),
                name: pocket.name,
                balance: pocket.balance.to_string(),
                account_type: "pocket".to_string(),
            });
        }

        // Calculate total income and expenses from transactions
        let (total_income, total_expenses) = self.calculate_income_expenses(user_id).await?;
        
        // Net worth is total balance (since we're tracking current balances in pockets)
        let net_worth = total_balance;

        Ok(AccountSummaryResponse {
            total_balance: total_balance.to_string(),
            accounts,
            total_income: total_income.to_string(),
            total_expenses: total_expenses.to_string(),
            net_worth: net_worth.to_string(),
        })
    }

    async fn calculate_income_expenses(&self, user_id: Uuid) -> Result<(Decimal, Decimal), AppError> {
        // Use a simple query to get all transactions for the user
        let query = crate::models::ListTransactionsQuery {
            page: None,
            limit: None,
            category: None,
            from_date: None,
            to_date: None,
            transaction_type: None,
        };

        let transactions = self.transaction_repository.find_by_user_id(user_id, &query).await?;
        
        let mut total_income = Decimal::new(0, 0);
        let mut total_expenses = Decimal::new(0, 0);

        for transaction in transactions {
            match transaction.transaction_type.as_str() {
                "income" => total_income += transaction.amount,
                "expense" => total_expenses += transaction.amount,
                _ => {} // Handle other types if needed
            }
        }

        Ok((total_income, total_expenses))
    }
}