//! Fhenix Nitrogen testnet client.
//! Uses raw JSON-RPC over HTTP — no heavy EVM crate dependency.
//! Handles: tx submission, receipt polling, contract read calls.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const FHENIX_RPC: &str = "https://api.nitrogen.fhenix.zone";
const CHAIN_ID: u64 = 8008135;

#[derive(Clone)]
pub struct FhenixClient {
    http: reqwest::Client,
    rpc_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub transaction_hash: String,
    pub block_number: String,
    pub status: String, // "0x1" = success, "0x0" = revert
    pub contract_address: Option<String>,
    pub gas_used: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FhenixNetworkInfo {
    pub chain_id: u64,
    pub rpc_url: String,
    pub explorer_url: String,
    pub latest_block: u64,
}

impl FhenixClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("HTTP client build failed"),
            rpc_url: FHENIX_RPC.to_string(),
        }
    }

    /// Send a JSON-RPC request and return the `result` field.
    async fn rpc_call(&self, method: &str, params: Value) -> Result<Value> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let resp = self
            .http
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await
            .context("Fhenix RPC request failed")?;

        let body: Value = resp.json().await.context("Fhenix RPC response parse failed")?;

        if let Some(err) = body.get("error") {
            bail!("Fhenix RPC error: {}", err);
        }

        Ok(body["result"].clone())
    }

    /// Fetch basic network info — used for health checks and UI display.
    pub async fn network_info(&self) -> Result<FhenixNetworkInfo> {
        let block_hex = self.rpc_call("eth_blockNumber", json!([])).await?;
        let block_str = block_hex.as_str().unwrap_or("0x0");
        let latest_block = u64::from_str_radix(block_str.trim_start_matches("0x"), 16)
            .unwrap_or(0);

        Ok(FhenixNetworkInfo {
            chain_id: CHAIN_ID,
            rpc_url: FHENIX_RPC.to_string(),
            explorer_url: "https://explorer.nitrogen.fhenix.zone".to_string(),
            latest_block,
        })
    }

    /// Fetch the tFHE balance of a wallet address.
    pub async fn get_balance(&self, address: &str) -> Result<String> {
        let result = self
            .rpc_call("eth_getBalance", json!([address, "latest"]))
            .await?;

        let hex = result.as_str().unwrap_or("0x0");
        // Convert hex wei to human-readable tFHE (18 decimals)
        let wei = u128::from_str_radix(hex.trim_start_matches("0x"), 16).unwrap_or(0);
        let tfhe = wei as f64 / 1e18;
        Ok(format!("{:.6} tFHE", tfhe))
    }

    /// Fetch transaction receipt by hash — used after frontend submits a tx.
    pub async fn get_receipt(&self, tx_hash: &str) -> Result<Option<TransactionReceipt>> {
        let result = self
            .rpc_call("eth_getTransactionReceipt", json!([tx_hash]))
            .await?;

        if result.is_null() {
            return Ok(None); // Not yet mined
        }

        let receipt = TransactionReceipt {
            transaction_hash: result["transactionHash"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            block_number: result["blockNumber"].as_str().unwrap_or("0x0").to_string(),
            status: result["status"].as_str().unwrap_or("0x0").to_string(),
            contract_address: result["contractAddress"]
                .as_str()
                .map(|s| s.to_string()),
            gas_used: result["gasUsed"].as_str().unwrap_or("0x0").to_string(),
        };

        Ok(Some(receipt))
    }

    /// Poll for receipt until mined or timeout (30 attempts × 2s = 60s).
    pub async fn await_receipt(&self, tx_hash: &str) -> Result<TransactionReceipt> {
        for attempt in 0..30 {
            if let Some(receipt) = self.get_receipt(tx_hash).await? {
                if receipt.status == "0x1" {
                    return Ok(receipt);
                }
                bail!("Transaction reverted: {}", tx_hash);
            }
            tracing::debug!("Receipt not yet available (attempt {}), waiting...", attempt + 1);
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
        bail!("Receipt timeout for tx: {}", tx_hash)
    }

    /// Call a read-only contract function (eth_call).
    /// `data` is ABI-encoded calldata as a hex string.
    pub async fn call_contract(&self, contract: &str, data: &str) -> Result<String> {
        let result = self
            .rpc_call(
                "eth_call",
                json!([{"to": contract, "data": data}, "latest"]),
            )
            .await?;

        Ok(result.as_str().unwrap_or("0x").to_string())
    }

    /// Verify a transaction exists and succeeded on-chain.
    /// Called by the backend after the frontend submits a payroll or vote tx.
    pub async fn verify_tx_success(&self, tx_hash: &str) -> Result<bool> {
        match self.get_receipt(tx_hash).await? {
            Some(r) => Ok(r.status == "0x1"),
            None => Ok(false),
        }
    }
}

impl Default for FhenixClient {
    fn default() -> Self {
        Self::new()
    }
}