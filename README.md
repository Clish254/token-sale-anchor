# Token Sale Program

This repository contains a token sale program implemented using the Anchor framework on Solana.
This program allows a seller to initialize a token sale, whitelist buyers, manage token purchases,
and end the sale.
This is my submission for the [Rust track](https://earn.superteam.fun/listings/hackathon/whitelist-gated-token-sale-st-talent-olympics/) in
the [superteam talent olympics hackathon](https://earn.superteam.fun/talent-olympics/)

## Table of Contents

- [Overview](#overview)

- [Prerequisites](#prerequisites)

- [Getting Started](#getting-started)

- [Usage](#usage)

  - [Initialize Token Sale](#initialize-token-sale)
  - [Whitelist Buyer](#whitelist-buyer)
  - [Buy Token](#buy-token)
  - [End Sale](#end-sale)

- [Error Codes](#error-codes)

- [License](#license)

## Overview

This smart contract provides functionalities for:

- Initializing a token sale with a specified price and purchase limit.
- Whitelisting buyers for the token sale.
- Facilitating token purchases by transferring tokens from a temporary account to the buyer's account.
- Ending the token sale by transferring remaining tokens back to the seller.

## Prerequisites

To run this smart contract, you will need the following:

- Rust and Cargo installed
- Solana CLI installed and configured
- Anchor framework installed

## Getting Started

Clone the repository and navigate to the project directory:

```sh
git clone https://github.com/yourusername/token-sale-anchor.git
cd token-sale-anchor
```

Build the project:

```sh
anchor build
```

Deploy the program:

```sh
anchor deploy
```

## Usage

### Initialize Token Sale

Initialize the token sale by providing the price per token and the purchase limit.

```rust
pub fn initialize(
    ctx: Context<InitializeTokenSale>,
    per_token_price: u64,
    purchase_limit: u64,
) -> Result<()>
```

- `per_token_price`: The price of each token in SOL.
- `purchase_limit`: The maximum number of tokens that can be purchased in a single transaction.

### Whitelist Buyer

Whitelist a buyer to allow them to participate in the token sale.

```rust
pub fn whitelist(ctx: Context<Whitelist>) -> Result<()>
```

### Buy Token

Allow a whitelisted buyer to purchase tokens.

```rust
pub fn buy_token(ctx: Context<BuyToken>, number_of_tokens: u64) -> Result<()>
```

- `number_of_tokens`: The number of tokens the buyer wants to purchase.

### End Sale

End the token sale, transferring any remaining tokens back to the seller and closing the temporary token account.

```rust
pub fn end_sale(ctx: Context<EndSale>) -> Result<()>
```

## Error Codes

The contract includes the following error codes:

- `InvalidSellerAccount`: The provided seller account is invalid.
- `PurchaseLimitExceeded`: The number of tokens requested exceeds the purchase limit.

