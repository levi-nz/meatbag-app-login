mod login;
mod credentials;

extern crate core;

use std::sync::Arc;
use clap::Parser;
use futures::lock::Mutex;
use crate::credentials::parse_credentials;
use crate::login::{login, LoginError};

#[derive(Parser, Debug)]
struct Args {
    /// The file path to the list of credentials.
    #[clap(default_value="credentials.txt")]
    credentials_path: String,

    /// Number of threads to run concurrently.
    ///
    /// A thread will attempt to login with credentials and will consume
    /// a next pair of credentials once it's complete.
    #[clap(default_value_t=1)]
    thread_count: usize
}

#[tokio::main]
async fn main() {
    // Init logger
    env_logger::init();

    // Parse args and validate input
    let args = Args::parse();
    assert!(!args.credentials_path.is_empty(), "credentials_path must NOT be empty.");
    assert!(args.thread_count != 0, "thread_count must NOT be zero.");

    // Parse credentials file
    let accounts = Arc::new(Mutex::new(
        parse_credentials(args.credentials_path.as_str())
            .expect("failed to parse credentials file")
    ));
    log::info!("Parsed {} accounts from {}", accounts.lock().await.len(), args.credentials_path);

    // Start login threads
    log::info!("Starting {} login threads", args.thread_count);

    let mut tasks = Vec::with_capacity(args.thread_count);
    for i in 0..args.thread_count {
        let local = accounts.clone();
        tasks.insert(i, tokio::spawn(async move {
            while let Some(account) = local.lock().await.pop_front() {
                // Create a client for this request
                let client = reqwest::ClientBuilder::new()
                    .build()
                    .expect("failed to build client");

                // Attempt login
                let result = login(&client, account.username.clone(), account.password.clone())
                    .await;

                match result {
                    Ok(valid_credentials) => {
                        if valid_credentials {
                            log::info!(target: "valid_logins", "{} is valid", account);
                        } else {
                            log::info!(target: "invalid_logins", "{} is invalid", account);
                        }
                    },
                    Err(error) => {
                        // If an error occurs when attempting to login, we put the account
                        // in the back of the deque so it can be tried again later.
                        log::error!(target: "failed_logins", "Failed to login: {}", error);
                        local.lock().await.push_back(account);
                    }
                }
            }
        }));
    }

    // Wait for all threads to complete
    futures::future::try_join_all(tasks)
        .await
        .expect("failed to join futures");

    log::info!("Done");
}
