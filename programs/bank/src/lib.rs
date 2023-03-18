use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};

declare_id!("3Fnff21ctkFSJvpAzrTeGMs1sscYKcHytB3f873NPAXY");

#[program]
pub mod bank {
    use super::*;

    pub fn create(ctx: Context<Create>, name: String) -> Result<()> {
        require!(name.len() != 0, BankError::EmptyName);
        require!(name.len() < 20, BankError::NameTooLong);
        ctx.accounts.bank.set_inner(Bank {
            name,
            balance: 0,
            owner: ctx.accounts.user.key(),
        });
        Ok(())
    }
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let user = &mut ctx.accounts.user;
        let bank = &mut ctx.accounts.bank;
        require!(amount > 0, BankError::ZeroAmount);
        require!(user.lamports() >= amount, BankError::InsufficientFunds);
        invoke(
            &system_instruction::transfer(user.key, &bank.key(), amount),
            &[user.to_account_info(), bank.to_account_info()],
        )?;
        bank.balance = bank.to_account_info().lamports() - Rent::get()?.minimum_balance(BANK_SIZE);
        Ok(())
    }
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let user = &ctx.accounts.user;
        let bank = &mut ctx.accounts.bank;
        require!(amount > 0, BankError::ZeroAmount);
        require_keys_eq!(bank.owner, user.key(), BankError::Unauthorized);
        bank.balance = bank.to_account_info().lamports() - Rent::get()?.minimum_balance(BANK_SIZE);
        require!(bank.balance >= amount, BankError::InsufficientBankBalance);
        **bank.to_account_info().try_borrow_mut_lamports()? -= amount;
        **user.try_borrow_mut_lamports()? += amount;
        bank.balance -= amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Create<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init, payer=user, space=BANK_SIZE, seeds=[b"user_bank", user.key().as_ref()], bump)]
    pub bank: Account<'info, Bank>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub bank: Account<'info, Bank>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub bank: Account<'info, Bank>,
}

pub const BANK_SIZE: usize = 8 + 4 + 20 + 8 + 32;
#[account]
pub struct Bank {
    pub name: String, // max bytes: 20
    pub balance: u64,
    pub owner: Pubkey,
}

#[error_code]
pub enum BankError {
    #[msg("Bank name can not be empty string")]
    EmptyName,
    #[msg("Bank name can be atmost 20 bytes long")]
    NameTooLong,
    #[msg("The deposit/withdraw amount can not be zero")]
    ZeroAmount,
    #[msg("User does not have enough funds to deposit")]
    InsufficientFunds,
    #[msg("Only bank's owner can withdraw funds")]
    Unauthorized,
    #[msg("Bank balance is lower than withdraw amount requested")]
    InsufficientBankBalance,
}
