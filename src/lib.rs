use reqwest::{header::HeaderValue, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;
use tracing::debug;

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

    #[error("Invalid response status code from 0x API: {0}")]
    ZeroXInvalidResponseStatusCode(StatusCode),

    #[error("Failed to parse response from 0x API: {0}")]
    ZeroXInvalidResponse(#[from] serde_json::Error),
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
    ) -> Result<ZeroXQuoteResponse, ZeroXClientError> {
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

        if let Some(taker_address) = params.taker_address {
            map.insert("takerAddress", taker_address);
        }

        if let Some(fee_recipient) = params.fee_recipient {
            map.insert("feeRecipient", fee_recipient);
        }

        if let Some(buy_token_percentage_fee) = params.buy_token_percentage_fee {
            map.insert("buyTokenPercentageFee", buy_token_percentage_fee);
        }

        if let Some(slippage_percentage) = params.slippage_percentage {
            map.insert("slippagePercentage", slippage_percentage);
        }

        if let Some(excluded_sources) = params.excluded_sources {
            map.insert("excludedSources", excluded_sources.join(","));
        }

        if let Some(included_sources) = params.included_sources {
            map.insert("includedSources", included_sources.join(","));
        }

        if let Some(skip_validation) = params.skip_validation {
            map.insert("skipValidation", skip_validation);
        }

        let client = reqwest::Client::new();

        let resp = client.get(&url).query(&map).headers(headers).send().await?;

        debug!("{:#?}", resp);

        if resp.status().as_u16() != 200 {
            return Err(ZeroXClientError::ZeroXInvalidResponseStatusCode(
                resp.status(),
            ));
        }

        let quote_response: Value = resp.json().await?;

        debug!("{:#?}", quote_response);

        let quote_response = serde_json::from_value::<ZeroXQuoteResponse>(quote_response)?;

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

// #[cfg(feature = "transaction_request")]
use ethers::core::types::{Address, Bytes, TransactionRequest, U256};

// #[cfg(feature = "transaction_request")]
pub trait ToTransactionRequest {
    fn to_transaction_request(&self) -> Result<TransactionRequest, Box<dyn std::error::Error>>;
}

// #[cfg(feature = "transaction_request")]
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

    use ethers::utils::parse_ether;

    use super::*;

    static VITALIK: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";

    #[test]
    fn test_init() {
        dotenv::dotenv().ok();

        let client = ZeroXClient::new(1, String::from("test")).unwrap();
        assert_eq!(client.base_url, "https://api.0x.org");
    }

    #[test]
    fn test_init_invalid_chain_id() {
        dotenv::dotenv().ok();

        let client = ZeroXClient::new(2, String::from("test"));
        assert!(client.is_err());
    }

    #[tokio::test]
    async fn test_get_quote() {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        dotenv::dotenv().ok();

        let client = ZeroXClient::new(1, std::env::var("ZEROX_API_KEY").unwrap()).unwrap();
        let quote = client
            .get_quote(ZeroXQuoteParams {
                sell_amount: String::from("1000000000000000000"),
                sell_token: String::from("ETH"),
                buy_token: String::from("0x6b175474e89094c44da98b954eedeac495271d0f"), //DAI
                ..Default::default()
            })
            .await;

        println!("{:#?}", quote);
        assert!(quote.is_ok());
    }

    #[tokio::test]
    async fn test_get_quote_with_slippage() {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        dotenv::dotenv().ok();

        let client = ZeroXClient::new(1, std::env::var("ZEROX_API_KEY").unwrap()).unwrap();
        let quote = client
            .get_quote(ZeroXQuoteParams {
                sell_amount: String::from("1000000000000000000"),
                sell_token: String::from("ETH"),
                buy_token: String::from("0x6b175474e89094c44da98b954eedeac495271d0f"), //DAI
                slippage_percentage: Some(String::from("0.1")),
                ..Default::default()
            })
            .await;

        assert!(quote.is_ok());
    }

    #[tokio::test]
    async fn test_get_quote_with_excluded_sources() {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        dotenv::dotenv().ok();

        let client = ZeroXClient::new(1, std::env::var("ZEROX_API_KEY").unwrap()).unwrap();
        let quote = client
            .get_quote(ZeroXQuoteParams {
                sell_amount: String::from("1000000000000000000"),
                sell_token: String::from("ETH"),
                buy_token: String::from("0x6b175474e89094c44da98b954eedeac495271d0f"), //DAI
                excluded_sources: Some(vec![String::from("Uniswap")]),
                ..Default::default()
            })
            .await;

        assert!(quote.is_ok());
    }

    #[tokio::test]
    async fn test_get_quote_with_included_sources() {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        dotenv::dotenv().ok();

        let client = ZeroXClient::new(1, std::env::var("ZEROX_API_KEY").unwrap()).unwrap();
        let quote = client
            .get_quote(ZeroXQuoteParams {
                sell_amount: String::from("1000000000000000000"),
                sell_token: String::from("ETH"),
                buy_token: String::from("0x6b175474e89094c44da98b954eedeac495271d0f"), //DAI
                included_sources: Some(vec![String::from("Uniswap")]),
                ..Default::default()
            })
            .await;

        assert!(quote.is_ok());
    }

    #[tokio::test]
    async fn test_get_quote_with_fee_recipient() {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        dotenv::dotenv().ok();

        let client = ZeroXClient::new(1, std::env::var("ZEROX_API_KEY").unwrap()).unwrap();
        let quote = client
            .get_quote(ZeroXQuoteParams {
                sell_amount: String::from("1000000000000000000"),
                sell_token: String::from("ETH"),
                buy_token: String::from("0x6b175474e89094c44da98b954eedeac495271d0f"), //DAI
                fee_recipient: Some(String::from(VITALIK)),
                buy_token_percentage_fee: Some(String::from("0.1")),
                ..Default::default()
            })
            .await;

        assert!(quote.is_ok());
    }

    #[tokio::test]
    async fn test_get_quote_with_taker_address() {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        dotenv::dotenv().ok();

        let client = ZeroXClient::new(1, std::env::var("ZEROX_API_KEY").unwrap()).unwrap();
        let quote = client
            .get_quote(ZeroXQuoteParams {
                sell_amount: String::from("1000000000000000000"),
                sell_token: String::from("ETH"),
                buy_token: String::from("0x6b175474e89094c44da98b954eedeac495271d0f"), //DAI
                taker_address: Some(String::from(VITALIK)),
                ..Default::default()
            })
            .await;

        assert!(quote.is_ok());

        let quote = quote.unwrap();

        assert!(quote.to.is_some());
        assert!(quote.data.is_some());
        assert!(quote.value.is_some());
        assert!(quote.gas_price.is_some());
    }

    #[tokio::test]
    async fn test_get_quote_with_taker_address_fails() {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        dotenv::dotenv().ok();

        let client = ZeroXClient::new(1, std::env::var("ZEROX_API_KEY").unwrap()).unwrap();
        let quote = client
            .get_quote(ZeroXQuoteParams {
                sell_amount: String::from("1000000000000000000"),
                sell_token: String::from("ETH"),
                buy_token: String::from("0x6b175474e89094c44da98b954eedeac495271d0f"), //DAI
                taker_address: Some(String::from("0x49AAf12E4367966B46e840371Ad0E91E0191e8B4")),
                ..Default::default()
            })
            .await;

        println!("{:#?}", quote);
        assert!(quote.is_err());
    }

    // #[cfg(feature = "transaction_request")]
    #[tokio::test]
    async fn test_get_quote_to_transaction_request() {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        dotenv::dotenv().ok();

        let client = ZeroXClient::new(1, std::env::var("ZEROX_API_KEY").unwrap()).unwrap();

        let sell_amount = parse_ether("1").unwrap().to_string();

        let quote = client
            .get_quote(ZeroXQuoteParams {
                sell_amount: sell_amount.clone(),
                sell_token: String::from("ETH"),
                buy_token: String::from("0x6b175474e89094c44da98b954eedeac495271d0f"), //DAI
                taker_address: Some(String::from(VITALIK)),
                ..Default::default()
            })
            .await;

        assert!(quote.is_ok());

        let quote = quote.unwrap();

        println!("{:#?}", quote);

        assert!(quote.to.is_some());
        assert!(quote.data.is_some());
        assert!(quote.value.is_some());
        assert!(quote.gas_price.is_some());

        let transaction_request = quote.to_transaction_request();

        assert!(transaction_request.is_ok());

        let transaction_request = transaction_request.unwrap();

        assert_eq!(
            transaction_request.value,
            Some(sell_amount.parse::<U256>().unwrap())
        );
    }
}
