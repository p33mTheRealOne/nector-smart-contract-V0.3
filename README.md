# Nector Smart Contract V0.2

## Overview

Nector Smart Contract V0.2 is the latest version of Nector's deterministic, non-custodial escrow protocol built on Solana.

## What's New?

### Penalties Are Burned

Penalty funds are no longer sent to the Nector platform. Instead, they are permanently burned.

### Revenue Comes From Fees

Nector generates protocol revenue through predefined escrow fees rather than user penalties.

### No Incentive to Create Penalties

The new economic model removes the platform's financial incentive to benefit from disputes, timeouts, or other penalty-triggering outcomes.

## Prerequites
Install Rust, the Solana CLI, and Anchor Framework on Windows (WSL), Linux, or Mac.

https://www.anchor-lang.com/docs/installation

```bash
# Verify that the installation was successful, check the Rust version:
rustc --version

# Verify that the installation was successful, check the Solana CLI version:
solana --version

# Verify that the installation was successful, check the Anchor CLI version:
anchor --version

# See your current config:
solana config get

# Set to devnet:
solana config set --url devnet

# Create new wallet:
solana-keygen new

# Request an airdrop of devnet SOL:
solana airdrop 5

# Check your wallet's SOL balance:
solana balance # If it show 5 you good to go!
```

Prepare the project
```bash
# Clone repo:
git clone https://github.com/p33mTheRealOne/nector-smart-contract-V0.2

# Install node modules
yarn
```

## Start

### Build Program

```bash
# Build the program
# This will generate new keypair for the program if doesn't exist
anchor build

# Sync all keys:
anchor keys sync

# If the program_id in lib.rs changed. Build the program again:
anchor build
```

### Deploy program

```bash
# Set to devnet:
solana config set --url devnet

# Make sure you have some SOL in your wallet beacuse this will cost you some SOL
solana balance

# Deploy program to devnet 
solana program deploy target/deploy/nector_smart_contract_V0_2.so
```

### Test

https://github.com/p33mTheRealOne/nector-smart-contract-V0.2/tree/main/tests/how_to_use

# Learn more:
https://nector.chat/docs
