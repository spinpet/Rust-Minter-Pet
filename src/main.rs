use anyhow::Result;
use bs58;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use solana_sdk::signer::{keypair::Keypair, Signer};
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::Duration;

#[derive(Parser)]
#[command(name = "solana-mint-generator")]
#[command(about = "Generate Solana mint addresses with specific suffixes")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate addresses ending with 'pump' for pump.fun tokens
    Pump {
        /// Number of addresses to generate
        #[arg(short, long, default_value = "1")]
        count: u32,
        /// Output format (json or txt)
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Output filename (optional, will auto-generate if not provided)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Generate addresses ending with 'bonk' for lets.bonk tokens
    Bonk {
        /// Number of addresses to generate
        #[arg(short, long, default_value = "1")]
        count: u32,
        /// Output format (json or txt)
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Output filename (optional, will auto-generate if not provided)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Generate addresses ending with 'pet' for pet tokens
    Pet {
        /// Number of addresses to generate
        #[arg(short, long, default_value = "1")]
        count: u32,
        /// Output format (json or txt)
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Output filename (optional, will auto-generate if not provided)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Generate both pump and bonk addresses
    Both {
        /// Number of addresses to generate for each type
        #[arg(short, long, default_value = "1")]
        count: u32,
        /// Output format (json or txt)
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Output filename prefix (optional, will auto-generate if not provided)
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Serialize, Deserialize)]
struct AddressRecord {
    pub_key: String,
    private_key: String,
    suffix_type: String,
    created_at: String,
}

struct AddressGenerator {
    attempts: Arc<AtomicU64>,
}

impl AddressGenerator {
    fn new() -> Self {
        Self {
            attempts: Arc::new(AtomicU64::new(0)),
        }
    }

    async fn generate_addresses(&self, suffix: &str, count: u32, format: &str, output: Option<&str>) -> Result<Vec<(Keypair, String)>> {
        let mut results = Vec::new();
        let start_time = Instant::now();
        
        // Get number of CPU cores
        let num_cores = thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1);
        
        println!("🔍 Generating {} addresses ending with '{}' using {} CPU cores...", count, suffix, num_cores);
        println!("🚀 Running at 100% CPU utilization...");
        
        // Generate all addresses first
        let batch_start_time = Instant::now();
        for i in 0..count {
            let keypair = self.find_address_with_suffix(suffix, num_cores, batch_start_time).await?;
            let pub_key = keypair.pubkey().to_string();
            
            println!("✨ Found address {}/{}: {}", i + 1, count, pub_key);
            results.push((keypair, pub_key));
        }
        
        // Save to file
        self.save_addresses_to_file(&results, suffix, format, output).await?;
        
        let elapsed = batch_start_time.elapsed();
        let total_attempts = self.attempts.load(Ordering::Relaxed);
        
        println!("\n📊 Generation complete!");
        println!("⏱️  Total time: {:?}", elapsed);
        println!("🎯 Total attempts: {}", total_attempts);
        println!("📈 Average attempts per address: {:.2}", total_attempts as f64 / count as f64);
        println!("⚡ Performance: {:.2} attempts/second", total_attempts as f64 / elapsed.as_secs_f64());
        
        Ok(results)
    }

    async fn generate_both_addresses(&self, count: u32, format: &str, output: Option<&str>) -> Result<(Vec<(Keypair, String)>, Vec<(Keypair, String)>)> {
        let mut pump_results = Vec::new();
        let mut bonk_results = Vec::new();
        let batch_start_time = Instant::now();
        
        // Get number of CPU cores
        let num_cores = thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1);
        
        println!("🔍 Generating {} addresses for both 'pump' and 'bonk' using {} CPU cores...", count, num_cores);
        println!("🚀 Running at 100% CPU utilization...");
        println!("🎯 Target: {} pump + {} bonk = {} total addresses", count, count, count * 2);
        
        // Generate addresses for both types simultaneously
        let count_usize = count as usize;
        while pump_results.len() < count_usize || bonk_results.len() < count_usize {
            let keypair = self.find_address_with_both_suffixes(num_cores, batch_start_time).await?;
            let pub_key = keypair.pubkey().to_string();
            
            if pub_key.ends_with("pump") && pump_results.len() < count_usize {
                println!("✨ Found PUMP address {}/{}: {}", pump_results.len() + 1, count, pub_key);
                pump_results.push((keypair, pub_key));
            } else if pub_key.ends_with("bonk") && bonk_results.len() < count_usize {
                println!("✨ Found BONK address {}/{}: {}", bonk_results.len() + 1, count, pub_key);
                bonk_results.push((keypair, pub_key));
            }
            
            // Show progress
            let total_found = pump_results.len() + bonk_results.len();
            let total_target = count_usize * 2;
            if total_found % 10 == 0 || total_found == total_target {
                println!("📊 Progress: {}/{} addresses found ({} pump, {} bonk)", 
                        total_found, total_target, pump_results.len(), bonk_results.len());
            }
        }
        
        // Save to files
        let pump_output = output.as_ref().map(|s| format!("{}_pump", s));
        let bonk_output = output.as_ref().map(|s| format!("{}_bonk", s));
        
        self.save_addresses_to_file(&pump_results, "pump", format, pump_output.as_deref()).await?;
        self.save_addresses_to_file(&bonk_results, "bonk", format, bonk_output.as_deref()).await?;
        
        let elapsed = batch_start_time.elapsed();
        let total_attempts = self.attempts.load(Ordering::Relaxed);
        
        println!("\n📊 Generation complete!");
        println!("⏱️  Total time: {:?}", elapsed);
        println!("🎯 Total attempts: {}", total_attempts);
        println!("📈 Average attempts per address: {:.2}", total_attempts as f64 / (count * 2) as f64);
        println!("⚡ Performance: {:.2} attempts/second", total_attempts as f64 / elapsed.as_secs_f64());
        println!("✅ Generated {} pump and {} bonk addresses", pump_results.len(), bonk_results.len());
        
        Ok((pump_results, bonk_results))
    }

    async fn save_addresses_to_file(&self, results: &[(Keypair, String)], suffix: &str, format: &str, output: Option<&str>) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = output.map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}_addresses_{}.{}", suffix, timestamp, format));
        
        println!("💾 Saving {} addresses to: {}", results.len(), filename);
        
        match format.to_lowercase().as_str() {
            "json" => self.save_as_json(results, suffix, &filename).await?,
            "txt" => self.save_as_txt(results, suffix, &filename).await?,
            _ => return Err(anyhow::anyhow!("Unsupported format: {}. Use 'json' or 'txt'", format)),
        }
        
        println!("✅ Successfully saved addresses to: {}", filename);
        Ok(())
    }

    async fn save_as_json(&self, results: &[(Keypair, String)], suffix: &str, filename: &str) -> Result<()> {
        let records: Vec<AddressRecord> = results
            .iter()
            .map(|(keypair, pub_key)| AddressRecord {
                pub_key: pub_key.clone(),
                private_key: bs58::encode(keypair.to_bytes()).into_string(),
                suffix_type: suffix.to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            })
            .collect();
        
        let json_content = serde_json::to_string_pretty(&records)?;
        let mut file = File::create(filename)?;
        writeln!(file, "{}", json_content)?;
        Ok(())
    }

    async fn save_as_txt(&self, results: &[(Keypair, String)], suffix: &str, filename: &str) -> Result<()> {
        let mut file = File::create(filename)?;
        
        // Write header
        writeln!(file, "# Solana Mint Addresses - Generated with suffix '{}'", suffix)?;
        writeln!(file, "# Generated at: {}", chrono::Utc::now().to_rfc3339())?;
        writeln!(file, "# Format: public_key,private_key,suffix_type")?;
        writeln!(file, "")?;
        
        // Write addresses
        for (keypair, pub_key) in results {
            writeln!(file, "{},{},{}", 
                pub_key, 
                bs58::encode(keypair.to_bytes()).into_string(), 
                suffix
            )?;
        }
        
        Ok(())
    }

    async fn find_address_with_suffix(&self, suffix: &str, num_cores: usize, batch_start_time: Instant) -> Result<Keypair> {
        let (tx, mut rx) = mpsc::channel::<Keypair>(1);
        let found = Arc::new(AtomicBool::new(false));
        let attempts = self.attempts.clone();
        let suffix_owned = suffix.to_string();
        
        // Spawn worker threads on all CPU cores
        let mut handles = Vec::new();
        for thread_id in 0..num_cores {
            let tx = tx.clone();
            let found = found.clone();
            let attempts = attempts.clone();
            let suffix = suffix_owned.clone();
            let batch_start_time = batch_start_time.clone();
            
            let handle = tokio::task::spawn_blocking(move || {
                let mut local_attempts = 0u64;
                let mut last_report = Instant::now();
                
                loop {
                    // Check if another thread found the address
                    if found.load(Ordering::Relaxed) {
                        break;
                    }
                    
                    let keypair = Keypair::new();
                    let pubkey = keypair.pubkey();
                    let address = pubkey.to_string();
                    
                    local_attempts += 1;
                    attempts.fetch_add(1, Ordering::Relaxed);
                    
                    // Report progress from thread 0 only every 5 seconds
                    if thread_id == 0 && last_report.elapsed() >= Duration::from_secs(5) {
                        let total_attempts = attempts.load(Ordering::Relaxed);
                        let total_elapsed = batch_start_time.elapsed().as_secs_f64();
                        let keys_per_second = if total_elapsed > 0.0 { total_attempts as f64 / total_elapsed } else { 0.0 };
                        println!("🔄 Total attempts: {} ({:.0} keys/s) (searching for '{}' on {} cores)", 
                                total_attempts, keys_per_second, suffix, num_cores);
                        last_report = Instant::now();
                    }
                    
                    if address.ends_with(&suffix) {
                        // Signal other threads to stop
                        found.store(true, Ordering::Relaxed);
                        
                        println!("🎉 Found matching address after {} local attempts on thread {}!", 
                                local_attempts, thread_id);
                        
                        // Send the result
                        if tx.blocking_send(keypair).is_err() {
                            // Channel was closed, another thread might have found it first
                            break;
                        }
                        break;
                    }
                    
                    // No delay - run at 100% CPU
                }
            });
            
            handles.push(handle);
        }
        
        // Drop the original sender so the channel can close when all workers are done
        drop(tx);
        
        // Wait for the first result
        let result = rx.recv().await.ok_or_else(|| {
            anyhow::anyhow!("All worker threads finished without finding a matching address")
        })?;
        
        // Signal all threads to stop
        found.store(true, Ordering::Relaxed);
        
        // Wait for all threads to complete
        for handle in handles {
            let _ = handle.await;
        }
        
        Ok(result)
    }

    async fn find_address_with_both_suffixes(&self, num_cores: usize, batch_start_time: Instant) -> Result<Keypair> {
        let (tx, mut rx) = mpsc::channel::<Keypair>(1);
        let found = Arc::new(AtomicBool::new(false));
        let attempts = self.attempts.clone();
        
        // Spawn worker threads on all CPU cores
        let mut handles = Vec::new();
        for thread_id in 0..num_cores {
            let tx = tx.clone();
            let found = found.clone();
            let attempts = attempts.clone();
            let batch_start_time = batch_start_time.clone();
            
            let handle = tokio::task::spawn_blocking(move || {
                let mut local_attempts = 0u64;
                let mut last_report = Instant::now();
                
                loop {
                    // Check if another thread found the address
                    if found.load(Ordering::Relaxed) {
                        break;
                    }
                    
                    let keypair = Keypair::new();
                    let pubkey = keypair.pubkey();
                    let address = pubkey.to_string();
                    
                    local_attempts += 1;
                    attempts.fetch_add(1, Ordering::Relaxed);
                    
                    // Report progress from thread 0 only every 5 seconds
                    if thread_id == 0 && last_report.elapsed() >= Duration::from_secs(5) {
                        let total_attempts = attempts.load(Ordering::Relaxed);
                        let total_elapsed = batch_start_time.elapsed().as_secs_f64();
                        let keys_per_second = if total_elapsed > 0.0 { total_attempts as f64 / total_elapsed } else { 0.0 };
                        println!("🔄 Total attempts: {} ({:.0} keys/s) (searching for 'pump' or 'bonk' on {} cores)", 
                                total_attempts, keys_per_second, num_cores);
                        last_report = Instant::now();
                    }
                    
                    // Check for either suffix
                    if address.ends_with("pump") || address.ends_with("bonk") {
                        // Signal other threads to stop
                        found.store(true, Ordering::Relaxed);
                        
                        println!("🎉 Found matching address after {} local attempts on thread {}! (ends with: {})", 
                                local_attempts, thread_id, 
                                if address.ends_with("pump") { "pump" } else { "bonk" });
                        
                        // Send the result
                        if tx.blocking_send(keypair).is_err() {
                            // Channel was closed, another thread might have found it first
                            break;
                        }
                        break;
                    }
                    
                    // No delay - run at 100% CPU
                }
            });
            
            handles.push(handle);
        }
        
        // Drop the original sender so the channel can close when all workers are done
        drop(tx);
        
        // Wait for the first result
        let result = rx.recv().await.ok_or_else(|| {
            anyhow::anyhow!("All worker threads finished without finding a matching address")
        })?;
        
        // Signal all threads to stop
        found.store(true, Ordering::Relaxed);
        
        // Wait for all threads to complete
        for handle in handles {
            let _ = handle.await;
        }
        
        Ok(result)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let generator = AddressGenerator::new();
    
    match cli.command {
        Commands::Pump { count, format, output } => {
            generator.generate_addresses("pump", count, &format, output.as_deref()).await?;
        }
        Commands::Bonk { count, format, output } => {
            generator.generate_addresses("bonk", count, &format, output.as_deref()).await?;
        }
        Commands::Pet { count, format, output } => {
            generator.generate_addresses("Pet", count, &format, output.as_deref()).await?;
        }
        Commands::Both { count, format, output } => {
            generator.generate_both_addresses(count, &format, output.as_deref()).await?;
        }
    }
    
    Ok(())
} 