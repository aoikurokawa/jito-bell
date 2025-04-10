use std::str::FromStr;

use ::borsh::BorshDeserialize;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_stake_pool::instruction::StakePoolInstruction;
use yellowstone_grpc_proto::prelude::CompiledInstruction;

#[derive(Debug)]
pub enum JitoStakePool {
    DepositStakeWithSlippage {
        ix: Instruction,
        minimum_pool_tokens_out: u64,
    },
    WithdrawStakeWithSlippage {
        ix: Instruction,
        minimum_lamports_out: u64,
    },
    DepositSol {
        ix: Instruction,
        amount: u64,
    },
    WithdrawSol {
        ix: Instruction,
        amount: u64,
    },
}

impl JitoStakePool {
    /// Retrieve Program ID of SPL Stake Pool Program
    pub fn program_id() -> Pubkey {
        Pubkey::from_str("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy").unwrap()
    }

    pub fn parse_jito_stake_pool_ix(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
    ) -> Option<JitoStakePool> {
        let stake_pool_ix = StakePoolInstruction::try_from_slice(&instruction.data).unwrap();

        match stake_pool_ix {
            StakePoolInstruction::DepositStakeWithSlippage {
                minimum_pool_tokens_out,
            } => Some(Self::parse_deposit_stake_with_slippage_ix(
                instruction,
                account_keys,
                minimum_pool_tokens_out,
            )),
            StakePoolInstruction::WithdrawStakeWithSlippage {
                pool_tokens_in: _,
                minimum_lamports_out,
            } => Some(Self::parse_withdraw_stake_with_slippage_ix(
                instruction,
                account_keys,
                minimum_lamports_out,
            )),
            StakePoolInstruction::DepositSol(amount) => Some(Self::parse_deposit_sol_ix(
                instruction,
                account_keys,
                amount,
            )),
            StakePoolInstruction::WithdrawSol(amount) => Some(Self::parse_withdraw_sol_ix(
                instruction,
                account_keys,
                amount,
            )),
            _ => None,
        }
    }
    /// Parse Deposit Stake With Slippage Instruction
    /// https://github.com/solana-program/stake-pool/blob/4ad88c05c567d47cbf4f3ea7e6cb765e15b336b9/program/src/instruction.rs#L1834C1-L1845C64
    ///
    ///   0. `[w]` Stake pool
    ///   1. `[w]` Validator stake list storage account
    ///   2. `[s]/[]` Stake pool deposit authority
    ///   3. `[]` Stake pool withdraw authority
    ///   4. `[w]` Stake account to join the pool (withdraw authority for the
    ///      stake account should be first set to the stake pool deposit
    ///      authority)
    ///   5. `[w]` Validator stake account for the stake account to be merged
    ///      with
    ///   6. `[w]` Reserve stake account, to withdraw rent exempt reserve
    ///   7. `[w]` User account to receive pool tokens
    ///   8. `[w]` Account to receive pool fee tokens
    ///   9. `[w]` Account to receive a portion of pool fee tokens as referral
    ///      fees
    ///   10. `[w]` Pool token mint account
    ///   11. '[]' Sysvar clock account
    ///   12. '[]' Sysvar stake history account
    ///   13. `[]` Pool token program id,
    ///   14. `[]` Stake program id,
    fn parse_deposit_stake_with_slippage_ix(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
        minimum_pool_tokens_out: u64,
    ) -> JitoStakePool {
        let mut account_metas = [
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
        ];

        for (index, account) in instruction.accounts.iter().enumerate() {
            account_metas[index].pubkey = account_keys[*account as usize];
        }

        let ix = Instruction {
            program_id: Pubkey::new_unique(),
            accounts: account_metas.to_vec(),
            data: instruction.data.clone(),
        };

        JitoStakePool::DepositStakeWithSlippage {
            ix,
            minimum_pool_tokens_out,
        }
    }

    /// Parse Withdraw Stake With Slippage Instruction
    /// https://github.com/solana-program/stake-pool/blob/4ad88c05c567d47cbf4f3ea7e6cb765e15b336b9/program/src/instruction.rs#L671-L683
    ///
    ///   0. `[w]` Stake pool
    ///   1. `[w]` Validator stake list storage account
    ///   2. `[]` Stake pool withdraw authority
    ///   3. `[w]` Validator or reserve stake account to split
    ///   4. `[w]` Uninitialized stake account to receive withdrawal
    ///   5. `[]` User account to set as a new withdraw authority
    ///   6. `[s]` User transfer authority, for pool token account
    ///   7. `[w]` User account with pool tokens to burn from
    ///   8. `[w]` Account to receive pool fee tokens
    ///   9. `[w]` Pool token mint account
    ///  10. `[]` Sysvar clock account (required)
    ///  11. `[]` Pool token program id
    ///  12. `[]` Stake program id,
    fn parse_withdraw_stake_with_slippage_ix(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
        minimum_lamports_out: u64,
    ) -> JitoStakePool {
        let mut account_metas = [
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
        ];

        for (index, account) in instruction.accounts.iter().enumerate() {
            account_metas[index].pubkey = account_keys[*account as usize];
        }

        let ix = Instruction {
            program_id: Pubkey::new_unique(),
            accounts: account_metas.to_vec(),
            data: instruction.data.clone(),
        };

        JitoStakePool::WithdrawStakeWithSlippage {
            ix,
            minimum_lamports_out,
        }
    }

    /// Parse Deposit SOL Instruction
    /// https://github.com/solana-program/stake-pool/blob/4ad88c05c567d47cbf4f3ea7e6cb765e15b336b9/program/src/instruction.rs#L367C1-L377C64
    ///
    ///   0. `[w]` Stake pool
    ///   1. `[]` Stake pool withdraw authority
    ///   2. `[w]` Reserve stake account, to deposit SOL
    ///   3. `[s]` Account providing the lamports to be deposited into the pool
    ///   4. `[w]` User account to receive pool tokens
    ///   5. `[w]` Account to receive fee tokens
    ///   6. `[w]` Account to receive a portion of fee as referral fees
    ///   7. `[w]` Pool token mint account
    ///   8. `[]` System program account
    ///   9. `[]` Token program id
    ///  10. `[s]` (Optional) Stake pool sol deposit authority.
    fn parse_deposit_sol_ix(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
        amount: u64,
    ) -> JitoStakePool {
        let mut account_metas = [
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
        ];

        for (index, account) in instruction.accounts.iter().enumerate() {
            account_metas[index].pubkey = account_keys[*account as usize];
        }

        let ix = Instruction {
            program_id: Pubkey::new_unique(),
            accounts: account_metas.to_vec(),
            data: instruction.data.clone(),
        };

        JitoStakePool::DepositSol { ix, amount }
    }

    /// Parse Withdraw SOL Instruction
    /// https://github.com/solana-program/stake-pool/blob/4ad88c05c567d47cbf4f3ea7e6cb765e15b336b9/program/src/instruction.rs#L391-L404
    ///
    ///   0. `[w]` Stake pool
    ///   1. `[]` Stake pool withdraw authority
    ///   2. `[s]` User transfer authority, for pool token account
    ///   3. `[w]` User account to burn pool tokens
    ///   4. `[w]` Reserve stake account, to withdraw SOL
    ///   5. `[w]` Account receiving the lamports from the reserve, must be a
    ///      system account
    ///   6. `[w]` Account to receive pool fee tokens
    ///   7. `[w]` Pool token mint account
    ///   8. '[]' Clock sysvar
    ///   9. '[]' Stake history sysvar
    ///  10. `[]` Stake program account
    ///  11. `[]` Token program id
    ///  12. `[s]` (Optional) Stake pool sol withdraw authority
    fn parse_withdraw_sol_ix(
        instruction: &CompiledInstruction,
        account_keys: &[Pubkey],
        amount: u64,
    ) -> JitoStakePool {
        let mut account_metas = [
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
        ];

        for (index, account) in instruction.accounts.iter().enumerate() {
            account_metas[index].pubkey = account_keys[*account as usize];
        }

        let ix = Instruction {
            program_id: Pubkey::new_unique(),
            accounts: account_metas.to_vec(),
            data: instruction.data.clone(),
        };

        JitoStakePool::WithdrawSol { ix, amount }
    }
}
