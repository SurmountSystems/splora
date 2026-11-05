pub fn loads_peg_in_addresses_bitcoin_tx_url(txid: &str, vout: u32) -> String {
    format!("https://mempool.space/tx/{txid}#vout={vout}")
}
