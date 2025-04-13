use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod call_external {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let signer = &ctx.accounts.signer;
        let pubkey = signer.signer_key().unwrap();
        msg!("Signer's pubkey: {}", pubkey);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    pub signer: Signer<'info>,
}
