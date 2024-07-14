use anchor_lang::prelude::*;
use anchor_spl::token::{spl_token::instruction::AuthorityType, SetAuthority, Token, TokenAccount};

declare_id!("Ha9ZBABH37ZY2sYKWUuKegRRPR1m58o8Jkz9yzdF6qro");

#[program]
pub mod token_sale_anchor {

    use super::*;

    pub fn initialize(
        ctx: Context<InitializeTokenSale>,
        per_token_price: u64,
        purchase_limit: u64,
    ) -> Result<()> {
        let seller = &ctx.accounts.seller;
        let temp_token_account = &ctx.accounts.temp_token_account;
        let token_sale_token_acct_authority = &ctx.accounts.token_sale_token_acct_authority;

        ctx.accounts.token_sale_account.set_inner(TokenSale {
            seller_pubkey: seller.key(),
            temp_token_account_pubkey: temp_token_account.key(),
            per_token_price,
            purchase_limit,
        });

        let context = ctx.accounts.token_program_context(SetAuthority {
            current_authority: seller.to_account_info(),
            account_or_mint: temp_token_account.to_account_info(),
        });
        msg!("Change temp_token_account authority: seller -> token_program");
        anchor_spl::token::set_authority(
            context,
            AuthorityType::AccountOwner,
            Some(token_sale_token_acct_authority.key()),
        )?;
        Ok(())
    }

    pub fn whitelist(ctx: Context<Whitelist>) -> Result<()> {
        let seller = &ctx.accounts.seller;
        let token_sale_account = &ctx.accounts.token_sale_account;

        if seller.key() != token_sale_account.seller_pubkey {
            return err!(ErrorCode::InvalidSellerAccount);
        }
        ctx.accounts
            .buyer_whitelist_account
            .set_inner(WhitelistData {
                is_whitelisted: true,
            });

        Ok(())
    }

    pub fn buy_token(ctx: Context<BuyToken>, number_of_tokens: u64) -> Result<()> {
        let buyer = &ctx.accounts.buyer;
        let seller = &ctx.accounts.seller;
        let buyer_token_account = &ctx.accounts.buyer_token_account;
        let temp_token_account = &ctx.accounts.temp_token_account;
        let token_sale_account = &ctx.accounts.token_sale_account;
        let token_sale_token_acct_authority = &ctx.accounts.token_sale_token_acct_authority;

        if seller.key() != token_sale_account.seller_pubkey {
            return err!(ErrorCode::InvalidSellerAccount);
        }

        if number_of_tokens > token_sale_account.purchase_limit {
            return err!(ErrorCode::PurchaseLimitExceeded);
        }

        msg!(
            "Transfer {} SOL : buyer account -> seller account",
            token_sale_account.per_token_price * number_of_tokens
        );
        let context = ctx
            .accounts
            .token_program_context(anchor_lang::system_program::Transfer {
                from: buyer.to_account_info(),
                to: seller.to_account_info(),
            });

        anchor_lang::system_program::transfer(
            context,
            token_sale_account.per_token_price * number_of_tokens,
        )?;

        msg!("Transfer tokens: temp token account -> buyer token account");
        let context = ctx
            .accounts
            .token_program_context(anchor_spl::token::Transfer {
                from: temp_token_account.to_account_info(),
                to: buyer_token_account.to_account_info(),
                authority: token_sale_token_acct_authority.to_account_info(),
            });
        anchor_spl::token::transfer(
            context,
            token_sale_account.per_token_price * number_of_tokens,
        )?;
        Ok(())
    }

    pub fn end_sale(ctx: Context<EndSale>) -> Result<()> {
        let seller = &ctx.accounts.seller;
        let seller_token_account = &ctx.accounts.seller_token_account;
        let temp_token_account = &ctx.accounts.temp_token_account;
        let token_sale_token_acct_authority = &ctx.accounts.token_sale_token_acct_authority;

        msg!("Transfer tokens: temp token account -> seller account");
        let context = ctx
            .accounts
            .token_program_context(anchor_spl::token::Transfer {
                from: temp_token_account.to_account_info(),
                to: seller_token_account.to_account_info(),
                authority: token_sale_token_acct_authority.to_account_info(),
            });
        anchor_spl::token::transfer(context, temp_token_account.amount)?;

        msg!("close account temp token account");
        let context = ctx
            .accounts
            .token_program_context(anchor_spl::token::CloseAccount {
                account: temp_token_account.to_account_info(),
                destination: seller.to_account_info(),
                authority: token_sale_token_acct_authority.to_account_info(),
            });
        anchor_spl::token::close_account(context)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeTokenSale<'info> {
    // external accounts
    #[account(mut)]
    seller: Signer<'info>,
    #[account(mut,  token::authority=seller)]
    temp_token_account: Account<'info, TokenAccount>,
    // PDAs
    #[account(
        init,
        payer = seller,
        space = TokenSale::LEN,
        seeds = [b"token_sale".as_ref(), seller.key().as_ref()], bump
    )]
    token_sale_account: Account<'info, TokenSale>,
    #[account(
        seeds = [b"authority".as_ref(), token_sale_account.key().as_ref()], bump
    )]
    token_sale_token_acct_authority: SystemAccount<'info>,
    // Programs
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> InitializeTokenSale<'info> {
    pub fn token_program_context<T: ToAccountMetas + ToAccountInfos<'info>>(
        &self,
        data: T,
    ) -> CpiContext<'_, '_, '_, 'info, T> {
        CpiContext::new(self.token_program.to_account_info(), data)
    }
}

#[account]
#[derive(Debug)]
pub struct TokenSale {
    pub seller_pubkey: Pubkey,
    pub temp_token_account_pubkey: Pubkey,
    pub per_token_price: u64,
    pub purchase_limit: u64,
}

impl TokenSale {
    pub const LEN: usize = {
        let discriminator = 8;
        let seller_pubkey = 32;
        let temp_token_account_pubkey = 32;
        let per_token_price = 8;
        let purchase_limit = 8;
        discriminator + seller_pubkey + temp_token_account_pubkey + per_token_price + purchase_limit
    };
}

#[derive(Accounts)]
pub struct BuyToken<'info> {
    // external accounts
    #[account(mut)]
    buyer: Signer<'info>,
    #[account(address = token_sale_account.seller_pubkey)]
    seller: SystemAccount<'info>,
    #[account(
        seeds = [b"buyer_whitelist".as_ref(), token_sale_account.key().as_ref(), buyer.key().as_ref()], bump
    )]
    buyer_whitelist_account: Account<'info, WhitelistData>,
    #[account(mut,  token::authority=token_sale_token_acct_authority)]
    temp_token_account: Account<'info, TokenAccount>,
    #[account(mut,  token::authority=buyer)]
    buyer_token_account: Account<'info, TokenAccount>,
    // PDAs
    #[account(
        seeds = [b"token_sale".as_ref(), seller.key().as_ref()], bump
    )]
    token_sale_account: Account<'info, TokenSale>,
    #[account(
        seeds = [b"authority".as_ref(), token_sale_account.key().as_ref()], bump
    )]
    token_sale_token_acct_authority: SystemAccount<'info>,
    // Programs
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> BuyToken<'info> {
    pub fn token_program_context<T: ToAccountMetas + ToAccountInfos<'info>>(
        &self,
        data: T,
    ) -> CpiContext<'_, '_, '_, 'info, T> {
        CpiContext::new(self.token_program.to_account_info(), data)
    }
}

#[derive(Accounts)]
pub struct EndSale<'info> {
    // external accounts
    #[account(mut)]
    seller: Signer<'info>,
    #[account(mut,  token::authority=seller)]
    seller_token_account: Account<'info, TokenAccount>,
    #[account(mut,  token::authority=token_sale_token_acct_authority)]
    temp_token_account: Account<'info, TokenAccount>,
    // PDAs
    #[account(
        seeds = [b"token_sale".as_ref(), seller.key().as_ref()], bump
    )]
    token_sale_account: Account<'info, TokenSale>,
    #[account(
        seeds = [b"authority".as_ref(), token_sale_account.key().as_ref()], bump
    )]
    token_sale_token_acct_authority: SystemAccount<'info>,
    // Programs
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> EndSale<'info> {
    pub fn token_program_context<T: ToAccountMetas + ToAccountInfos<'info>>(
        &self,
        data: T,
    ) -> CpiContext<'_, '_, '_, 'info, T> {
        CpiContext::new(self.token_program.to_account_info(), data)
    }
}

#[derive(Accounts)]
pub struct Whitelist<'info> {
    // external accounts
    #[account(mut)]
    seller: Signer<'info>,
    #[account()]
    buyer: SystemAccount<'info>,
    // PDAs
    #[account(
        seeds = [b"token_sale".as_ref(), seller.key().as_ref()], bump
    )]
    token_sale_account: Account<'info, TokenSale>,
    #[account(
        init,
        payer = seller,
        space = WhitelistData::LEN,
        seeds = [b"buyer_whitelist".as_ref(), token_sale_account.key().as_ref(), buyer.key().as_ref()], bump
    )]
    buyer_whitelist_account: Account<'info, WhitelistData>,
    // Programs
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

#[account]
#[derive(Debug)]
pub struct WhitelistData {
    is_whitelisted: bool,
}

impl WhitelistData {
    pub const LEN: usize = {
        let discriminator = 8;
        let is_whitelisted = 1;
        discriminator + is_whitelisted
    };
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid Seller")]
    InvalidSellerAccount,
    #[msg("Purchase Limit Exceeded")]
    PurchaseLimitExceeded,
}
