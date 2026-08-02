use anyhow::Result;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    println!("[BOOT] MUN-EVE PROBE");
    println!("[FSW ] v0.1.0");

    loop {
        println!("TLM phase=BOOT status=OK");
        sleep(Duration::from_secs(1)).await;
    }
}