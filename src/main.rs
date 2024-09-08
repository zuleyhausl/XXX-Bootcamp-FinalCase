use reqwest::Client;
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct BalanceResponse {
    balances: Vec<Balance>,
}

#[derive(Deserialize)]
struct Balance {
    asset_type: String,
    balance: String,
}

#[derive(Deserialize)]
struct TransactionResponse {
    _embedded: TransactionEmbedded,
}

#[derive(Deserialize)]
struct TransactionEmbedded {
    records: Vec<Transaction>,
}

#[derive(Deserialize)]
struct Transaction {
    id: String,
    memo_type: Option<String>,
    memo: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let horizon_url = env::var("STELLAR_HORIZON_URL")?;
    let public_key = env::var("PUBLIC_KEY")?;
    let recipient_keys = vec![
        "recipient_public_key_1".to_string(),
        "recipient_public_key_2".to_string(),
    ];
    let amount = "10"; // XLM miktarı
    let memo = "Payment for services".to_string();

    let client = Client::new();

    // Bakiye sorgulama
    let balance_url = format!("{}/accounts/{}", horizon_url, public_key);
    let balance_response = client.get(&balance_url).send().await?;
    let balance_body = balance_response.text().await?;
    let balance: BalanceResponse = serde_json::from_str(&balance_body)?;

    println!("Account Balances:");
    for balance in balance.balances {
        println!("{}: {}", balance.asset_type, balance.balance);
    }

    // Çoklu alıcıya transfer
    for recipient_key in recipient_keys {
        let transfer_url = format!("{}/transactions", horizon_url);
        let response = client.post(&transfer_url)
            .json(&serde_json::json!({
                "source_account": public_key,
                "destination": recipient_key,
                "amount": amount,
                "memo": memo
            }))
            .send()
            .await?;

        if response.status().is_success() {
            println!("Transfer to {} successful.", recipient_key);
        } else {
            println!("Failed to transfer to {}.", recipient_key);
        }
    }

    // İşlem geçmişini görüntüleme
    let transactions_url = format!("{}/accounts/{}/transactions", horizon_url, public_key);
    let transaction_response = client.get(&transactions_url).send().await?;
    let transaction_body = transaction_response.text().await?;
    let transactions: TransactionResponse = serde_json::from_str(&transaction_body)?;

    println!("Transaction History:");
    for transaction in transactions._embedded.records {
        println!("Transaction ID: {}", transaction.id);
        if let Some(memo) = transaction.memo {
            println!("Memo: {}", memo);
        }
    }

    Ok(())
}
