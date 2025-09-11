use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod multiple_test_configs {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn increment_x(ctx: Context<IncrementX>) -> Result<()> {
        ctx.accounts.storage.x += 1;
        Ok(())
    }

    pub fn increment_y(ctx: Context<IncrementY>) -> Result<()> {
        ctx.accounts.storage.y += 1;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + Storage::INIT_SPACE, seeds = [], bump)]
    pub storage: Account<'info, Storage>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct IncrementX<'info> {
    #[account(mut, seeds = [], bump)]
    pub storage: Account<'info, Storage>,
}

#[derive(Accounts)]
pub struct IncrementY<'info> {
    #[account(mut, seeds = [], bump)]
    pub storage: Account<'info, Storage>,
}

#[account]
#[derive(InitSpace)]
pub struct Storage {
    x: u64,
    y: u64,
}
