use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Serialize, Deserialize, Default)]
pub struct ZeroXQuoteParams {
    pub sell_token: String,
    pub buy_token: String,
    pub sell_amount: String,
    pub fee_recipient: Option<String>,
    pub buy_token_percentage_fee: Option<String>,
    pub taker_address: Option<String>,
    pub slippage_percentage: Option<String>,
    pub excluded_sources: Option<Vec<String>>,
    pub included_sources: Option<Vec<String>>,
    pub skip_validation: Option<String>,
}

#[derive(Error, Debug)]
pub enum ZeroXClientError {
    #[error("Invalid chain id: {0}")]
    InvalidChainId(u64),

    #[error("Failed to get quote: {0}")]
    ZeroXQuoteError(#[from] reqwest::Error),
}

pub struct ZeroXClient {
    base_url: String,
    api_key: String,
}

impl ZeroXClient {
    pub fn new(chain_id: u64, api_key: String) -> Result<ZeroXClient, ZeroXClientError> {
        let base_url_hashmap: HashMap<u64, String> = vec![
            (1, "https://api.0x.org".to_string()),
            (42161, "https://arbitrum.api.0x.org".to_string()),
            (43114, "https://avalanche.api.0x.org".to_string()),
            (250, "https://fantom.api.0x.org".to_string()),
            (137, "https://polygon.api.0x.org".to_string()),
            (42220, "https://celo.api.0x.org".to_string()),
            (56, "https://bsc.api.0x.org".to_string()),
            (10, "https://optimisim.api.0x.org".to_string()),
        ]
        .into_iter()
        .collect();

        let base_url = base_url_hashmap
            .get(&chain_id)
            .ok_or(ZeroXClientError::InvalidChainId(chain_id))?
            .clone();

        Ok(ZeroXClient { base_url, api_key })
    }

    pub async fn get_quote(
        &self,
        params: ZeroXQuoteParams,
    ) -> Result<ZeroXQuoteResponse, reqwest::Error> {
        let url = format!("{}/swap/v1/quote", self.base_url);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.append(
            "0x-api-key",
            HeaderValue::from_str(self.api_key.as_str()).unwrap(),
        );
        headers.append("Content-Type", HeaderValue::from_static("application/json"));

        let mut map = HashMap::new();
        map.insert("sellToken", params.sell_token);
        map.insert("buyToken", params.buy_token);
        map.insert("sellAmount", params.sell_amount);
        // Repeat the above pattern for the rest of the parameters

        let client = reqwest::Client::new();
        let resp = client.get(&url).query(&map).headers(headers).send().await?;
        let quote_response = resp.json().await?;

        Ok(quote_response)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FillData {
    pub token_address_path: Option<Vec<String>>,
    pub router: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub maker_token: Option<String>,
    pub taker_token: Option<String>,
    pub maker_amount: Option<String>,
    pub taker_amount: Option<String>,
    pub fill_data: Option<FillData>,
    pub source: Option<String>,
    pub source_path_id: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub name: Option<String>,
    pub proportion: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Fees {
    pub zero_ex_fee: Option<ZeroExFee>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ZeroExFee {
    pub billing_type: Option<String>,
    pub fee_amount: Option<String>,
    pub fee_token: Option<String>,
    pub fee_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ZeroXQuoteResponse {
    pub chain_id: Option<i32>,
    pub price: Option<String>,
    pub guaranteed_price: Option<String>,
    pub estimated_price_impact: Option<String>,
    pub to: Option<String>,
    pub data: Option<String>,
    pub value: Option<String>,
    pub gas: Option<String>,
    pub estimated_gas: Option<String>,
    pub gas_price: Option<String>,
    pub protocol_fee: Option<String>,
    pub minimum_protocol_fee: Option<String>,
    pub buy_token_address: Option<String>,
    pub sell_token_address: Option<String>,
    pub buy_amount: Option<String>,
    pub sell_amount: Option<String>,
    pub sources: Option<Vec<Source>>,
    pub orders: Option<Vec<Order>>,
    pub allowance_target: Option<String>,
    pub sell_token_to_eth_rate: Option<String>,
    pub buy_token_to_eth_rate: Option<String>,
    pub fees: Option<Fees>,
    pub gross_price: Option<String>,
    pub gross_buy_amount: Option<String>,
    pub gross_sell_amount: Option<String>,
}

#[cfg(feature = "transaction_request")]
use ethers::core::types::{Address, Bytes, TransactionRequest, U256};

#[cfg(feature = "transaction_request")]
pub trait ToTransactionRequest {
    fn to_transaction_request(&self) -> Result<TransactionRequest, Box<dyn std::error::Error>>;
}

#[cfg(feature = "transaction_request")]
impl ToTransactionRequest for ZeroXQuoteResponse {
    fn to_transaction_request(&self) -> Result<TransactionRequest, Box<dyn std::error::Error>> {
        let to = self
            .to
            .as_ref()
            .ok_or("Missing 'to' field")?
            .parse::<Address>()?;

        let data = self
            .data
            .as_ref()
            .ok_or("Missing 'data' field")?
            .parse::<Bytes>()?;

        let value = self
            .value
            .as_ref()
            .ok_or("Missing 'value' field")?
            .parse::<U256>()?;

        let gas_price = self
            .gas_price
            .as_ref()
            .ok_or("Missing 'gas_price' field")?
            .parse::<U256>()?;

        Ok(TransactionRequest {
            from: None,
            to: Some(to.into()),
            gas_price: Some(gas_price),
            gas: None,
            value: Some(value),
            data: Some(data),
            nonce: None,
            chain_id: None,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_zerox_client_init() {
        let client = ZeroXClient::new(1, String::from("test")).unwrap();
        assert_eq!(client.base_url, "https://api.0x.org");
    }

    #[test]
    fn test_zerox_client_init_invalid_chain_id() {
        let client = ZeroXClient::new(2, String::from("test"));
        assert!(client.is_err());
    }

    #[tokio::test]
    async fn test_zerox_client_get_quote() {
        let client = ZeroXClient::new(1, std::env::var("ZEROX_API_KEY").unwrap()).unwrap();
        let quote = client
            .get_quote(ZeroXQuoteParams {
                sell_amount: String::from("1000000000000000000"),
                sell_token: String::from("ETH"),
                buy_token: String::from("0x6b175474e89094c44da98b954eedeac495271d0f"), //DAI
                ..Default::default()
            })
            .await;

        assert!(quote.is_ok());
    }
}
