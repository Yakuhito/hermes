use chia::{
    clvm_traits::{clvm_quote, ToClvm},
    consensus::consensus_constants::TEST_CONSTANTS,
    protocol::Bytes32,
};
use chia_wallet_sdk::{
    Conditions, Layer, P2Eip712MessageLayer, P2Eip712MessageSolution, Simulator, SpendContext,
};
use hex::encode;
use k256::ecdsa::{Signature, VerifyingKey};
use std::io::{self, Write};

fn main() {
    println!("Hello, Chia! Setting things up...");

    let ctx = &mut SpendContext::new();
    let mut sim = Simulator::new();

    let mut input = String::new();

    print!("Enter your public key: ");
    io::stdout().flush().unwrap(); // Flush to ensure prompt is displayed

    io::stdin().read_line(&mut input).unwrap();
    let hex_str = input.trim();

    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let Ok(uncompressed_pk) = hex::decode(hex_str) else {
        eprintln!("Invalid public key");
        return;
    };

    let pk = VerifyingKey::from_sec1_bytes(&uncompressed_pk).unwrap();
    println!("Public key (compressed): 0x{:}", encode(pk.to_sec1_bytes()));

    println!("Creating 31337-amount coin for you to spend...");

    let layer = P2Eip712MessageLayer::from_genesis_challenge(
        pk.to_sec1_bytes().to_vec().try_into().unwrap(),
        TEST_CONSTANTS.genesis_challenge,
    );
    let coin_puzzle_reveal = layer.construct_puzzle(ctx).unwrap();
    let coin_puzzle_hash = ctx.tree_hash(coin_puzzle_reveal);

    let coin = sim.new_coin(coin_puzzle_hash.into(), 1337);

    let delegated_puzzle_ptr = clvm_quote!(Conditions::new().reserve_fee(1337))
        .to_clvm(&mut ctx.allocator)
        .unwrap();
    let delegated_solution_ptr = ctx.allocator.nil();

    let delegated_puzzle_hash: Bytes32 = ctx.tree_hash(delegated_puzzle_ptr).into();

    println!("Done! Please input the following data and sign:");
    println!("  coin_id: 0x{:}", encode(coin.coin_id()));
    println!(
        "  delegated_puzzle_hash: 0x{:}",
        encode(delegated_puzzle_hash)
    );

    let msg_hash = layer.hash_to_sign(coin.coin_id(), delegated_puzzle_hash);
    println!("Expected message to sign: 0x{:}", encode(msg_hash));

    let mut input = String::new();
    print!("Enter your signature: ");
    io::stdout().flush().unwrap(); // Flush to ensure prompt is displayed

    io::stdin().read_line(&mut input).unwrap();
    let hex_str = input.trim();

    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let sig = hex::decode(hex_str).unwrap();
    let signature = Signature::from_slice(&sig[..64]).unwrap();
    println!("Signature: 0x{:}", encode(signature.to_vec()));

    println!("Spending the coin...");

    let coin_spend = layer
        .construct_coin_spend(
            ctx,
            coin,
            P2Eip712MessageSolution {
                my_id: coin.coin_id(),
                signed_hash: msg_hash,
                signature: signature.to_vec().try_into().unwrap(),
                delegated_puzzle: delegated_puzzle_ptr,
                delegated_solution: delegated_solution_ptr,
            },
        )
        .unwrap();

    ctx.insert(coin_spend);

    sim.spend_coins(ctx.take(), &[]).unwrap();

    println!("Spend successful! Yay!");
}
