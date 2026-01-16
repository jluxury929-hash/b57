use alloy::{
    providers::{Provider, ProviderBuilder, WsConnect},
    primitives::{address, Address, U256, B256},
    rpc::types::eth::TransactionRequest,
    sol,
};
// FIX: Direct imports from revm_primitives to avoid "not in root" errors
use revm_primitives::{AccountInfo, TxKind, Address as RevmAddress, U256 as RevmU256};
use revm::{
    database::{CacheDB, EmptyDB},
    evm::Evm, // FIX: Explicit path to Evm struct
};
use std::{sync::Arc, net::TcpListener, io::Write, thread, time::Instant};
use dashmap::DashMap;
use colored::Colorize;
use futures_util::StreamExt;
use url::Url;

// --- ELITE CONSTANTS ---
const EXECUTOR: Address = address!("0x458f94e935f829DCAD18Ae0A18CA5C3E223B71DE");
const WETH: Address = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");

sol! {
    #[sol(rpc)]
    interface IApexOmega {
        function startFlashStrike(address token, uint256 amount, bytes calldata userData) external;
    }
}

pub struct ArbRequest {
    pub tx: TransactionRequest,
    pub estimated_profit: U256,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    // 1. HEALTH GUARD
    thread::spawn(|| {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        if let Ok(listener) = TcpListener::bind(format!("0.0.0.0:{}", port)) {
            for stream in listener.incoming() {
                if let Ok(mut s) = stream {
                    let _ = s.write_all(b"HTTP/1.1 200 OK\r\n\r\nOK");
                }
            }
        }
    });

    // 2. PROVIDER SETUP
    let rpc_url = std::env::var("ETH_RPC_WSS").expect("Missing ETH_RPC_WSS");
    let url_obj = Url::parse(&rpc_url).expect("Invalid WebSocket URL");
    
    // Connect using Alloy 1.4 syntax
    let ws_connect = WsConnect::new(url_obj);
    let provider = ProviderBuilder::new().connect_ws(ws_connect).await?;
    let provider = Arc::new(provider);
    
    let shared_db = CacheDB::new(EmptyDB::default());

    println!("{}", "╔════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║    ⚡ APEX SINGULARITY | SYSTEMS ONLINE                ║".cyan().bold());
    println!("{}", "╚════════════════════════════════════════════════════════╝".cyan());

    match provider.subscribe_pending_transactions().await {
        Ok(sub) => {
            let mut stream = sub.into_stream();
            while let Some(tx_hash) = stream.next().await {
                let prov = Arc::clone(&provider);
                let mut local_db = shared_db.clone(); 

                tokio::spawn(async move {
                    if let Some(strike_req) = simulate_flash_locally(&mut local_db, tx_hash).await {
                        if strike_req.estimated_profit > U256::from(100000000000000u128) { 
                            let _ = prov.send_transaction(strike_req.tx).await;
                            println!("🚀 STRIKE DISPATCHED");
                        }
                    }
                });
            }
        }
        Err(e) => eprintln!("Subscription failed: {:?}", e),
    }
    
    Ok(())
}

async fn simulate_flash_locally(db: &mut CacheDB<EmptyDB>, _tx_hash: B256) -> Option<ArbRequest> {
    // FIX: Builder pattern for REVM v33
    let mut evm = Evm::builder()
        .with_db(db)
        .build();

    let executor_revm = RevmAddress::from_slice(EXECUTOR.as_slice());
    let weth_revm = RevmAddress::from_slice(WETH.as_slice());

    // Inject Mock Liquidity
    let mock_info = AccountInfo {
        balance: RevmU256::from(1000000000000000000000u128),
        ..Default::default()
    };
    
    // Use the context to access DB in v33
    // evm.context.evm.db.insert_account_info(executor_revm, mock_info);

    let tx_env = evm.tx_mut();
    tx_env.caller = executor_revm;
    tx_env.transact_to = TxKind::Call(weth_revm);
    
    None
}
