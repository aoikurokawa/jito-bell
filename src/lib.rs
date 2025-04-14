use std::{collections::HashMap, path::PathBuf};

use error::JitoBellError;
use futures::{sink::SinkExt, stream::StreamExt};
use instruction::Instruction;
use log::{error, info};
use maplit::hashmap;
use parser::{
    stake_pool::SplStakePoolProgram, token_2022::SplToken2022Program, JitoBellProgram,
    JitoTransactionParser,
};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use subscribe_option::SubscribeOption;
use tonic::transport::channel::ClientTlsConfig;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestFilterTransactions,
};

use crate::config::JitoBellConfig;

pub mod config;
mod error;
pub mod instruction;
pub mod notification_config;
pub mod notification_info;
pub mod parser;
pub mod program;
pub mod subscribe_option;

pub struct JitoBellHandler {
    /// Configuration for Notification
    config: JitoBellConfig,

    /// RPC Client
    rpc_client: RpcClient,
}

impl JitoBellHandler {
    /// Initialize Jito Bell Handler
    pub fn new(
        endpoint: String,
        commitment: CommitmentConfig,
        config_path: PathBuf,
    ) -> Result<Self, JitoBellError> {
        let config_str = std::fs::read_to_string(&config_path).map_err(JitoBellError::Io)?;

        let config: JitoBellConfig = serde_yaml::from_str(&config_str)?;
        let rpc_client = RpcClient::new_with_commitment(endpoint.to_string(), commitment);

        Ok(Self { config, rpc_client })
    }

    /// Start heart beating
    pub async fn heart_beat(
        &self,
        subscribe_option: &SubscribeOption,
    ) -> Result<(), JitoBellError> {
        let mut client = GeyserGrpcClient::build_from_shared(subscribe_option.endpoint.clone())?
            .x_token(subscribe_option.x_token.clone())?
            .tls_config(ClientTlsConfig::new())?
            .connect()
            .await?;
        let (mut subscribe_tx, mut stream) = client.subscribe().await?;

        let subscribe_request = SubscribeRequest {
            slots: HashMap::new(),
            accounts: HashMap::new(),
            transactions: hashmap! { "".to_owned() => SubscribeRequestFilterTransactions {
                vote: subscribe_option.vote,
                failed: subscribe_option.failed,
                signature: subscribe_option.signature.clone(),
                account_include: subscribe_option.account_include.clone(),
                account_exclude: subscribe_option.account_exclude.clone(),
                account_required: subscribe_option.account_required.clone(),
            } },
            transactions_status: HashMap::new(),
            entry: HashMap::new(),
            blocks: HashMap::new(),
            blocks_meta: HashMap::new(),
            commitment: Some(subscribe_option.commitment as i32),
            accounts_data_slice: vec![],
            ping: None,
        };
        if let Err(e) = subscribe_tx.send(subscribe_request).await {
            return Err(JitoBellError::Subscription(format!(
                "Failed to send subscription request: {}",
                e
            )));
        }

        while let Some(message) = stream.next().await {
            match message {
                Ok(msg) => {
                    if let Some(UpdateOneof::Transaction(transaction)) = msg.update_oneof {
                        let parser = JitoTransactionParser::new(transaction);

                        info!("Instruction: {:?}", parser.programs);

                        self.send_notification(&parser).await?;
                    }
                }
                Err(error) => {
                    error!("Stream error: {error:?}");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Send notification
    pub async fn send_notification(
        &self,
        parser: &JitoTransactionParser,
    ) -> Result<(), JitoBellError> {
        info!("Before Send notification");
        for program in &parser.programs {
            match program {
                JitoBellProgram::SplStakePool(spl_stake_program) => {
                    info!("SPL Stake Pool");
                    if let Some(program_config) = self.config.programs.get(&program.to_string()) {
                        info!("Found Program Config");
                        if let Some(instruction) = program_config
                            .instructions
                            .get(&spl_stake_program.to_string())
                        {
                            self.handle_spl_stake_pool_program(
                                parser,
                                spl_stake_program,
                                instruction,
                            )
                            .await?;
                        }
                    }
                }
                JitoBellProgram::SplToken2022(_) => {
                    info!("Token 2022");
                }
            }
        }

        Ok(())
    }

    /// Handle SPL Stake Pool Program
    async fn handle_spl_stake_pool_program(
        &self,
        parser: &JitoTransactionParser,
        spl_stake_program: &SplStakePoolProgram,
        instruction: &Instruction,
    ) -> Result<(), JitoBellError> {
        info!("SPL Stake Program: {}", spl_stake_program);

        match spl_stake_program {
            SplStakePoolProgram::DepositStake { ix } => {
                let _stake_pool_info = &ix.accounts[0];
                let _validator_list_info = &ix.accounts[1];
                let _stake_deposit_authority_info = &ix.accounts[2];
                let withdraw_authority_info = &ix.accounts[3];
                let _stake_info = &ix.accounts[4];
                let _validator_stake_account_info = &ix.accounts[5];
                let _reserve_stake_account_info = &ix.accounts[6];
                let dest_user_pool_info = &ix.accounts[7];
                let _manager_fee_info = &ix.accounts[8];
                let _referrer_fee_info = &ix.accounts[9];
                let pool_mint_info = &ix.accounts[10];

                for program in &parser.programs {
                    if let JitoBellProgram::SplToken2022(program) = program {
                        match program {
                            SplToken2022Program::MintTo { ix, amount } => {
                                let mint_info = &ix.accounts[0];
                                let destination_account_info = &ix.accounts[1];
                                let owner_info = &ix.accounts[2];

                                if mint_info.pubkey.eq(&pool_mint_info.pubkey)
                                    && destination_account_info
                                        .pubkey
                                        .eq(&dest_user_pool_info.pubkey)
                                    && owner_info.pubkey.eq(&withdraw_authority_info.pubkey)
                                {
                                    self.dispatch_platform_notifications(
                                        &instruction.notification.destinations,
                                        &instruction.notification.description,
                                        *amount as f64,
                                        &parser.transaction_signature,
                                    )
                                    .await?;

                                    break;
                                }
                            }
                        }
                    }
                }
            }
            SplStakePoolProgram::WithdrawStake {
                ix: _,
                minimum_lamports_out,
            } => {
                self.dispatch_platform_notifications(
                    &instruction.notification.destinations,
                    &instruction.notification.description,
                    *minimum_lamports_out,
                    &parser.transaction_signature,
                )
                .await?;
            }
            SplStakePoolProgram::DepositSol { ix: _, amount } => {
                if *amount >= instruction.threshold {
                    self.dispatch_platform_notifications(
                        &instruction.notification.destinations,
                        &instruction.notification.description,
                        100.0,
                        &parser.transaction_signature,
                    )
                    .await?;
                }
            }
            SplStakePoolProgram::WithdrawSol { ix: _, amount } => {
                if *amount >= instruction.threshold {
                    self.dispatch_platform_notifications(
                        &instruction.notification.destinations,
                        &instruction.notification.description,
                        100.0,
                        &parser.transaction_signature,
                    )
                    .await?;
                }
            }
            SplStakePoolProgram::Initialize
            | SplStakePoolProgram::AddValidatorToPool
            | SplStakePoolProgram::RemoveValidatorFromPool
            | SplStakePoolProgram::DecreaseValidatorStake
            | SplStakePoolProgram::IncreaseValidatorStake
            | SplStakePoolProgram::SetPreferredValidator
            | SplStakePoolProgram::UpdateValidatorListBalance
            | SplStakePoolProgram::UpdateStakePoolBalance
            | SplStakePoolProgram::CleanupRemovedValidatorEntries
            | SplStakePoolProgram::SetManager
            | SplStakePoolProgram::SetFee
            | SplStakePoolProgram::SetStaker
            | SplStakePoolProgram::SetFundingAuthority
            | SplStakePoolProgram::CreateTokenMetadata
            | SplStakePoolProgram::UpdateTokenMetadata
            | SplStakePoolProgram::IncreaseAdditionalValidatorStake
            | SplStakePoolProgram::DecreaseAdditionalValidatorStake
            | SplStakePoolProgram::DecreaseValidatorStakeWithReserve
            | SplStakePoolProgram::Redelegate
            | SplStakePoolProgram::DepositStakeWithSlippage
            | SplStakePoolProgram::WithdrawStakeWithSlippage
            | SplStakePoolProgram::DepositSolWithSlippage
            | SplStakePoolProgram::WithdrawSolWithSlippage => {
                unreachable!()
            }
        }

        Ok(())
    }

    /// Dispatch platform notifications
    async fn dispatch_platform_notifications(
        &self,
        destinations: &[String],
        description: &str,
        amount: f64,
        transaction_signature: &str,
    ) -> Result<(), JitoBellError> {
        for destination in destinations {
            match destination.as_str() {
                "telegram" => {
                    info!("Will Send Telegram Notification");
                    self.send_telegram_message(description, amount, transaction_signature)
                        .await
                }
                "slack" => {
                    info!("Will Send Slack Notification");
                    self.send_slack_message(description, amount, transaction_signature)
                        .await?
                }
                "discord" => {
                    info!("Will Send Discord Notification");
                    self.send_discord_message(description, amount, transaction_signature)
                        .await?
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Send message to Telegram
    async fn send_telegram_message(&self, description: &str, amount: f64, sig: &str) {
        if let Some(telegram_config) = &self.config.notifications.telegram {
            let template = self
                .config
                .message_templates
                .get("telegram")
                .unwrap_or(self.config.message_templates.get("default").unwrap());
            let message = template
                .replace("{{description}}", description)
                .replace("{{amount}}", &format!("{:.2}", amount))
                .replace("{{tx_hash}}", sig);

            let bot_token = &telegram_config.bot_token;
            let chat_id = &telegram_config.chat_id;

            let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

            let client = reqwest::Client::new();
            let response = client
                .post(&url)
                .form(&[("chat_id", chat_id), ("text", &message)])
                .send()
                .await
                .unwrap();

            if !response.status().is_success() {
                println!("Failed to send Telegram message: {:?}", response.status());
            }
        }
    }

    /// Send message to Discord
    async fn send_discord_message(
        &self,
        description: &str,
        amount: f64,
        sig: &str,
    ) -> Result<(), JitoBellError> {
        if let Some(discord_config) = &self.config.notifications.discord {
            let webhook_url = &discord_config.webhook_url;

            let payload = serde_json::json!({
                "embeds": [{
                    "title": "New Transaction Detected",
                    "description": description,
                    "color": 3447003, // Blue color
                    "fields": [
                        {
                            "name": "Amount",
                            "value": format!("{:.2} SOL", amount),
                            "inline": true
                        },
                        {
                            "name": "Transaction",
                            "value": format!("[View on Explorer](https://explorer.solana.com/tx/{})", sig),
                            "inline": true
                        }
                    ],
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }]
            });

            let client = reqwest::Client::new();
            let response = client
                .post(webhook_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            match response {
                Ok(res) => {
                    if res.status().is_success() {
                        return Ok(());
                    } else {
                        return Err(JitoBellError::Notification(format!(
                            "Failed to send Discord message: {:?}",
                            res.status(),
                        )));
                    }
                }
                Err(e) => {
                    return Err(JitoBellError::Notification(format!(
                        "Error sending Discord message: {:?}",
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Send message to Slack
    async fn send_slack_message(
        &self,
        description: &str,
        amount: f64,
        sig: &str,
    ) -> Result<(), JitoBellError> {
        if let Some(slack_config) = &self.config.notifications.slack {
            let webhook_url = &slack_config.webhook_url;

            // Build a Slack message with blocks for better formatting
            let payload = serde_json::json!({
                "blocks": [
                    {
                        "type": "header",
                        "text": {
                            "type": "plain_text",
                            "text": "New Transaction Detected"
                        }
                    },
                    {
                        "type": "section",
                        "text": {
                            "type": "mrkdwn",
                            "text": format!("*Description:* {}", description)
                        }
                    },
                    {
                        "type": "section",
                        "fields": [
                            {
                                "type": "mrkdwn",
                                "text": format!("*Amount:* {:.2} SOL", amount)
                            },
                            {
                                "type": "mrkdwn",
                                "text": format!("*Transaction:* <https://explorer.solana.com/tx/{}|View on Explorer>", sig)
                            }
                        ]
                    }
                ]
            });

            let client = reqwest::Client::new();
            let response = client
                .post(webhook_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
                .map_err(|e| JitoBellError::Notification(format!("Slack request error: {}", e)))?;

            if !response.status().is_success() {
                return Err(JitoBellError::Notification(format!(
                    "Failed to send Slack message: Status {}",
                    response.status()
                )));
            }
        }

        Ok(())
    }
}
