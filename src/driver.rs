use chia::clvm_utils::TreeHash;
use chia_wallet_sdk::{DriverError, SpendContext};
use clvmr::NodePtr;
use hex_literal::hex;

pub const P2_EIP712_MESSAGE_PUZZLE: [u8; 196] = hex!("ff02ffff01ff02ffff03ffff8413d61f00ffff0eff17ff5f80ffff3eff05ffff3eff0bff2fffff02ff06ffff04ff02ffff04ff82017fff808080808080ff81bf80ffff01ff04ffff04ff04ffff04ff2fff808080ffff02ff82017fff8202ff8080ffff01ff088080ff0180ffff04ffff01ff46ff02ffff03ffff07ff0580ffff01ff0bffff0102ffff02ff06ffff04ff02ffff04ff09ff80808080ffff02ff06ffff04ff02ffff04ff0dff8080808080ffff01ff0bffff0101ff058080ff0180ff018080");
pub const P2_EIP712_MESSAGE_PUZZLE_HASH: TreeHash = TreeHash::new(hex!(
    "
    e1470661c64fc02b4a580fedbaa75db534219fb6afb264fe575f5856a0252c17
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
    use ethers::core::rand::thread_rng;
    use ethers::prelude::*;
    use ethers::signers::{LocalWallet, Signer};
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
    use tiny_keccak::{Hasher, Keccak};

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
