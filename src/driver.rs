use chia::clvm_utils::TreeHash;
use chia_wallet_sdk::{DriverError, SpendContext};
use clvmr::NodePtr;
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

#[cfg(test)]
mod tests {
    use super::*;
    use chia::consensus::consensus_constants::TEST_CONSTANTS;
    use ethers::core::rand::thread_rng;
    use ethers::prelude::*;
    use ethers::signers::{LocalWallet, Signer};
    use hex::encode;
    use k256::ecdsa::SigningKey;
    use tiny_keccak::{Hasher, Keccak};

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
        let mut keccak = Keccak::v256();
        let mut output = [0u8; 32];
        keccak.update(&uncompressed_pub_key[1..]); // Skip the '04' prefix
        keccak.finalize(&mut output);

        let pub_key_hash = &output[12..];
        println!("keccak256(Public Key): 0x{:}", encode(output));
        assert_eq!(
            format!("{:?}", address),
            format!("0x{:}", encode(pub_key_hash))
        );

        Ok(())
    }
}
