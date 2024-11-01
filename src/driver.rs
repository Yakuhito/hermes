use chia::clvm_utils::TreeHash;
use chia_wallet_sdk::{DriverError, SpendContext};
use clvmr::NodePtr;
use ethers::contract::{Eip712, EthAbiType};
use ethers::types::H256;
use hex_literal::hex;

pub const P2_EIP712_MESSAGE_PUZZLE: [u8; 234] = hex!("ff02ffff01ff02ffff03ffff22ffff09ff17ffff0cffff3eff81bf80ffff010cffff01208080ffff8413d61f00ffff0eff17ff5f80ffff3eff05ffff3eff0bff2fffff02ff06ffff04ff02ffff04ff8202ffff808080808080ff82017f8080ffff01ff04ffff04ff04ffff04ff2fff808080ffff02ff8202ffff8205ff8080ffff01ff08ffff01846e6f70658080ff0180ffff04ffff01ff46ff02ffff03ffff07ff0580ffff01ff0bffff0102ffff02ff06ffff04ff02ffff04ff09ff80808080ffff02ff06ffff04ff02ffff04ff0dff8080808080ffff01ff0bffff0101ff058080ff0180ff018080");
pub const P2_EIP712_MESSAGE_PUZZLE_HASH: TreeHash = TreeHash::new(hex!(
    "
    76b33566c2f473e69e6eecbacc9138e1d10f46c1607545aeab8f9a30b6c394e2
    "
));

pub trait SpendContextExt {
    fn p2_eip712_message_puzzle(&mut self) -> Result<NodePtr, DriverError>;
}

impl SpendContextExt for SpendContext {
    fn p2_eip712_message_puzzle(&mut self) -> Result<NodePtr, DriverError> {
        self.puzzle(P2_EIP712_MESSAGE_PUZZLE_HASH, &P2_EIP712_MESSAGE_PUZZLE)
    }
}

#[derive(Clone, Default, EthAbiType, Eip712)]
#[eip712(
    name = "ChiaCoinSpend",
    // TEST_CONSTANTS.agg_sig_data
    raw_salt = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
)]
pub struct ChiaCoinSpend {
    pub coin_id: H256,
    pub delegated_puzzle_hash: H256,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::core::rand::thread_rng;
    use ethers::prelude::*;
    use ethers::signers::{LocalWallet, Signer};
    use ethers::types::transaction::eip712::Eip712;
    use ethers::utils::keccak256;
    use hex::encode;
    use k256::ecdsa::SigningKey;

    // we really have to expose this in chia-sdk-test
    macro_rules! assert_puzzle_hash {
        ($puzzle:ident => $puzzle_hash:ident) => {
            let ctx = &mut SpendContext::new();
            let ptr = ctx.p2_eip712_message_puzzle().unwrap();
            let hash = ctx.tree_hash(ptr);
            assert_eq!($puzzle_hash, hash);
        };
    }

    #[test]
    fn test_puzzle_hashes() -> anyhow::Result<()> {
        assert_puzzle_hash!(P2_EIP712_MESSAGE_PUZZLE => P2_EIP712_MESSAGE_PUZZLE_HASH);

        Ok(())
    }

    #[test]
    fn test_thing() -> anyhow::Result<()> {
        let signing_key = SigningKey::random(&mut thread_rng());
        let wallet: LocalWallet = signing_key.into();

        let address = wallet.address();
        let public_key = wallet.signer().verifying_key();
        println!("Address: {:?}", address);

        // compute keccak256 of pub key (sanity check)
        let uncompressed_pub_key = public_key.clone().to_encoded_point(false);
        let uncompressed_pub_key = uncompressed_pub_key.as_bytes();
        println!("Public Key: 0x{:}", encode(public_key.to_sec1_bytes()));
        let output = keccak256(&uncompressed_pub_key[1..]);

        let pub_key_hash = &output[12..];
        println!("keccak256(Public Key): 0x{:}", encode(output));
        assert_eq!(
            format!("{:?}", address),
            format!("0x{:}", encode(pub_key_hash))
        );

        // test EIP-712 knowledge
        println!("--");

        let coin_id = keccak256(b"coin_id");
        let delegated_puzzle_hash = keccak256(b"delegated_puzzle_hash");
        println!("coin_id: 0x{:}", encode(coin_id));
        println!(
            "delegated_puzzle_hash: 0x{:}",
            encode(delegated_puzzle_hash)
        );

        let msg = ChiaCoinSpend {
            coin_id: H256::from(coin_id),
            delegated_puzzle_hash: H256::from(delegated_puzzle_hash),
        };
        println!(
            "Ethers struct hash: 0x{:}",
            encode(msg.struct_hash().unwrap())
        );
        let msg_hash = msg.encode_eip712().unwrap();
        println!("Hash To Sign (ethers): 0x{:}", encode(msg_hash));

        /*
        bytes32 messageHash = keccak256(abi.encode(
            typeHash,
            coin_id,
            delegated_puzzle_hash
        ));
        */
        let type_hash = keccak256(b"ChiaCoinSpend(bytes32 coin_id,bytes32 delegated_puzzle_hash)");
        let mut to_hash = Vec::new();
        to_hash.extend_from_slice(&type_hash);
        to_hash.extend_from_slice(&coin_id);
        to_hash.extend_from_slice(&delegated_puzzle_hash);

        let message_hash = keccak256(&to_hash);
        println!("Message Hash: 0x{:}", encode(message_hash));

        let mut to_hash = Vec::new();
        to_hash.extend_from_slice(&[0x19, 0x01]); // "\x19\x01",
        to_hash.extend_from_slice(&msg.domain_separator().unwrap());
        to_hash.extend_from_slice(&message_hash);

        let hash_to_sign = keccak256(&to_hash);
        println!(
            "Hash To Sign (hand-calculated): 0x{:}",
            encode(hash_to_sign)
        );

        Ok(())
    }
}
