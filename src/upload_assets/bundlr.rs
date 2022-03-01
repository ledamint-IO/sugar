use crate::common::*;
use std::{thread, time::Duration};

pub async fn get_bundlr_solana_address(http_client: &HttpClient, node: &str) -> Result<String> {
    let url = format!("{}/info", node);
    let data = http_client.get(&url).send().await?.json::<Value>().await?;
    let addresses = data.get("addresses").unwrap();

    let solana_address = addresses
        .get("solana")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    Ok(solana_address)
}

pub async fn fund_bundlr_address(
    program: &Program,
    http_client: &HttpClient,
    bundlr_address: Pubkey,
    node: &str,
    payer: &Keypair,
    amount: u64,
) -> Result<Response> {
    let ix = system_instruction::transfer(&payer.pubkey(), &bundlr_address, amount);
    let recent_blockhash = program.rpc().get_latest_blockhash()?;
    let payer_pubkey = payer.pubkey();

    let tx =
        Transaction::new_signed_with_payer(&[ix], Some(&payer_pubkey), &[payer], recent_blockhash);

    println!("Funding address: {payer_pubkey} with {amount} lamports.");
    let sig = program
        .rpc()
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &tx,
            CommitmentConfig::confirmed(),
        )?;

    println!("Signature: {sig}");

    thread::sleep(Duration::from_millis(5000));

    let mut map = HashMap::new();
    map.insert("tx_id", sig.to_string());

    let url = format!("{}/account/balance/solana", node);

    let response = http_client.post(&url).json(&map).send().await?;

    Ok(response)
}

pub async fn get_bundlr_balance(
    http_client: &HttpClient,
    address: &str,
    node: &str,
) -> Result<u64> {
    debug!("Getting balance for address: {address}");
    let url = format!("{}/account/balance/solana/?address={}", node, address);

    let response = http_client.get(&url).send().await?.json::<Value>().await?;

    println!("response: {response:?}");
    let value = response.get("balance").unwrap();
    println!("value: {value:?}");

    // Bundlr API returns balance as a number if it's zero but as a string if it's not. :-(
    let balance = if value.is_number() {
        value.as_u64().unwrap()
    } else {
        value.as_str().unwrap().parse::<u64>().unwrap()
    };

    println!("balance: {balance:?}");

    Ok(balance)
}

pub async fn get_bundlr_fee(http_client: &HttpClient, data_size: u64) -> Result<u64> {
    let required_amount = http_client
        .get(format!(
            "https://node1.bundlr.network/price/solana/{data_size}"
        ))
        .send()
        .await?
        .text()
        .await?
        .parse::<u64>()?;

    Ok(required_amount)
}