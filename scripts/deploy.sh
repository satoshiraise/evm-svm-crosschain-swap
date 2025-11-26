#!/bin/bash

# SuperSwap Solana Program Deployment Script
# 
# Usage:
#   ./scripts/deploy.sh <network>
#
# Networks: localnet, devnet, mainnet

set -e

NETWORK=${1:-localnet}

echo "================================================"
echo "SuperSwap Solana Program Deployment"
echo "Network: $NETWORK"
echo "================================================"

# Validate network
if [[ ! "$NETWORK" =~ ^(localnet|devnet|mainnet)$ ]]; then
    echo "Error: Invalid network. Use: localnet, devnet, or mainnet"
    exit 1
fi

# Set cluster
if [ "$NETWORK" = "localnet" ]; then
    solana config set --url localhost
elif [ "$NETWORK" = "devnet" ]; then
    solana config set --url devnet
elif [ "$NETWORK" = "mainnet" ]; then
    solana config set --url mainnet-beta
fi

echo ""
echo "Current configuration:"
solana config get

echo ""
echo "Wallet balance:"
solana balance

# Check balance for mainnet
if [ "$NETWORK" = "mainnet" ]; then
    BALANCE=$(solana balance | awk '{print $1}')
    if (( $(echo "$BALANCE < 5.0" | bc -l) )); then
        echo "⚠️  Warning: Low balance. You need at least 5 SOL for deployment."
        read -p "Continue anyway? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
fi

echo ""
echo "Step 1: Building program..."
anchor build

echo ""
echo "Step 2: Getting program ID..."
PROGRAM_ID=$(solana-keygen pubkey target/deploy/superswap_sol-keypair.json)
echo "Program ID: $PROGRAM_ID"

echo ""
echo "Step 3: Checking if program ID matches declared ID..."
DECLARED_ID=$(grep "declare_id" programs/superswap-sol/src/lib.rs | grep -oP '(?<=").*(?=")')
if [ "$PROGRAM_ID" != "$DECLARED_ID" ]; then
    echo "⚠️  Warning: Program ID mismatch!"
    echo "  Keypair ID:  $PROGRAM_ID"
    echo "  Declared ID: $DECLARED_ID"
    echo ""
    echo "Updating Anchor.toml and lib.rs..."
    
    # Update Anchor.toml
    sed -i "s/superswap_sol = \".*\"/superswap_sol = \"$PROGRAM_ID\"/" Anchor.toml
    
    # Update lib.rs
    sed -i "s/declare_id!(\".*\")/declare_id!(\"$PROGRAM_ID\")/" programs/superswap-sol/src/lib.rs
    
    echo "Rebuilding..."
    anchor build
else
    echo "✅ Program ID matches!"
fi

echo ""
echo "Step 4: Deploying program to $NETWORK..."

if [ "$NETWORK" = "mainnet" ]; then
    echo "⚠️  MAINNET DEPLOYMENT"
    echo "This will deploy to MAINNET. This action cannot be undone."
    read -p "Are you absolutely sure? (yes/no) " -r
    echo
    if [[ ! $REPLY = "yes" ]]; then
        echo "Deployment cancelled."
        exit 1
    fi
fi

anchor deploy --provider.cluster $NETWORK

echo ""
echo "Step 5: Verifying deployment..."
solana program show $PROGRAM_ID

echo ""
echo "================================================"
echo "✅ Deployment Complete!"
echo "================================================"
echo "Program ID: $PROGRAM_ID"
echo "Network: $NETWORK"
echo ""
echo "Next steps:"
echo "1. Initialize the program:"
echo "   ts-node scripts/initialize.ts"
echo ""
echo "2. Update frontend with program ID:"
echo "   PROGRAM_ID=$PROGRAM_ID"
echo ""
echo "3. Test the deployment:"
echo "   anchor test --skip-deploy --provider.cluster $NETWORK"
echo "================================================"

