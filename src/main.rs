use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::env;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    let rpc_url = "https://api.devnet.solana.com".to_string();
    let client = Arc::new(RpcClient::new(rpc_url));

    let sender_keypair = Arc::new(Keypair::from_base58_string(&env::var("PRIVATE_KEY")?));
    let recipient_keypair = Arc::new(Keypair::from_base58_string(&env::var("RECIPIENT_PRIVATE_KEY")?));

    let sequence_lock = Arc::new(Mutex::new(()));
    let mut handles = vec![];

    // SENDER -> RECIPIENT (2 transactions)
    for i in 0..2 {
        let client = Arc::clone(&client);
        let sender = Arc::clone(&sender_keypair);
        let recipient_pubkey = recipient_keypair.pubkey(); 
        let lock = Arc::clone(&sequence_lock);

        handles.push(thread::spawn(move || -> Result<(), anyhow::Error> {
            let _guard = lock.lock().unwrap(); 
            
            let transfer_ix = system_instruction::transfer(
                &sender.pubkey(),
                &recipient_pubkey,
                (0.1 * 1_000_000_000.0) as u64,
            );

            let recent_blockhash = client.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(
                &[transfer_ix],
                Some(&sender.pubkey()),
                &[&*sender],
                recent_blockhash,
            );

            thread::sleep(Duration::from_millis(500 * (i + 1) as u64));

            let signature = client.send_and_confirm_transaction(&tx)?;
            println!("{}. Sent 0.1 SOL from {} to {}. Signature: {}", 
                i+1, sender.pubkey(), recipient_pubkey, signature);
            Ok(())
        }));
    }

    // RECIPIENT -> SENDER (2 transactions)
    for i in 0..2 {
        let client = Arc::clone(&client);
        let recipient = Arc::clone(&recipient_keypair);
        let sender_pubkey = sender_keypair.pubkey(); 
        let lock = Arc::clone(&sequence_lock);

        handles.push(thread::spawn(move || -> Result<(), anyhow::Error> {
            let _guard = lock.lock().unwrap(); 
            
            let transfer_ix = system_instruction::transfer(
                &recipient.pubkey(),
                &sender_pubkey,
                (0.2 * 1_000_000_000.0) as u64,
            );

            let recent_blockhash = client.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(
                &[transfer_ix],
                Some(&recipient.pubkey()),
                &[&*recipient],
                recent_blockhash,
            );

            thread::sleep(Duration::from_millis(500 * (i + 1) as u64));

            let signature = client.send_and_confirm_transaction(&tx)?;
            println!("{}. Sent 0.2 SOL from {} to {}. Signature: {}", 
                i+1, recipient.pubkey(), sender_pubkey, signature);
            Ok(())
        }));
    }

    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}