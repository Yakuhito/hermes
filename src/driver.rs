use chia::{
    clvm_traits::{self, FromClvm, ToClvm},
    clvm_utils::{CurriedProgram, TreeHash},
    protocol::{Bytes, Bytes32},
};
use chia_wallet_sdk::{DriverError, Layer, Puzzle, Spend, SpendContext};
use clvmr::{Allocator, NodePtr};
use ethers::utils::keccak256;
use hex_literal::hex;

pub const P2_EIP712_MESSAGE_PUZZLE: [u8; 191] = hex!("ff02ffff01ff02ffff03ffff20ffff8413d61f00ff17ffff3eff05ffff3eff0bff2fffff02ff06ffff04ff02ffff04ff81bfff808080808080ff5f8080ffff01ff04ffff04ff04ffff04ff2fff808080ffff02ff81bfff82017f8080ffff01ff088080ff0180ffff04ffff01ff46ff02ffff03ffff07ff0580ffff01ff0bffff0102ffff02ff06ffff04ff02ffff04ff09ff80808080ffff02ff06ffff04ff02ffff04ff0dff8080808080ffff01ff0bffff0101ff058080ff0180ff018080");
pub const P2_EIP712_MESSAGE_PUZZLE_HASH: TreeHash = TreeHash::new(hex!(
    "
    7ffc39fd252c800c7208999bb1f284e2148464bf83a1d719d5b6da3db3a32c29
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

type EthPubkeyBytes = [u8; 33];
type EthSignatureBytes = [u8; 64];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct P2Eip712MessageLayer {
    pub genesis_challenge: Bytes32,
    pub pubkey: EthPubkeyBytes,
}

#[derive(Debug, Clone, PartialEq, Eq, ToClvm, FromClvm)]
#[clvm(curry)]
pub struct P2Eip712MessageArgs {
    pub prefix_and_domain_separator: Bytes,
    pub type_hash: Bytes32,
    pub pubkey: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, ToClvm, FromClvm)]
#[clvm(solution)]
pub struct P2Eip712MessageSolution<P, S> {
    pub my_id: Bytes32,
    pub signature: Bytes,
    pub delegated_puzzle: P,
    pub delegated_solution: S,
}

impl P2Eip712MessageLayer {
    pub fn new(genesis_challenge: Bytes32, pubkey: EthPubkeyBytes) -> Self {
        Self {
            genesis_challenge,
            pubkey,
        }
    }

    pub fn spend(
        &self,
        ctx: &mut SpendContext,
        my_id: Bytes32,
        signature: EthSignatureBytes,
        delegated_spend: Spend,
    ) -> Result<Spend, DriverError> {
        self.construct_spend(
            ctx,
            P2Eip712MessageSolution {
                my_id,
                signature: Bytes::new(signature.to_vec()),
                delegated_puzzle: delegated_spend.puzzle,
                delegated_solution: delegated_spend.solution,
            },
        )
    }

    pub fn domain_separator(&self) -> Bytes32 {
        let type_hash = keccak256(b"EIP712Domain(string name,bytes32 salt)");

        keccak256(ethers::abi::encode(&[
            ethers::abi::Token::FixedBytes(type_hash.to_vec()),
            ethers::abi::Token::FixedBytes(keccak256("Chia Coin Spend").to_vec()),
            ethers::abi::Token::FixedBytes(self.genesis_challenge.to_vec()),
        ]))
        .into()
    }

    pub fn prefix_and_domain_separator(&self) -> [u8; 34] {
        let mut pads = [0u8; 34];
        pads[0] = 0x19;
        pads[1] = 0x01;
        pads[2..].copy_from_slice(&self.domain_separator());
        pads
    }

    pub fn type_hash(&self) -> Bytes32 {
        keccak256(b"ChiaCoinSpend(bytes32 coin_id,bytes32 delegated_puzzle_hash)").into()
    }
}

impl Layer for P2Eip712MessageLayer {
    type Solution = P2Eip712MessageSolution<NodePtr, NodePtr>;

    fn construct_puzzle(&self, ctx: &mut SpendContext) -> Result<NodePtr, DriverError> {
        let curried = CurriedProgram {
            program: ctx.p2_eip712_message_puzzle()?,
            args: P2Eip712MessageArgs {
                prefix_and_domain_separator: self.prefix_and_domain_separator().to_vec().into(),
                type_hash: self.type_hash(),
                pubkey: self.pubkey.to_vec().into(),
            },
        };
        ctx.alloc(&curried)
    }

    fn construct_solution(
        &self,
        ctx: &mut SpendContext,
        solution: Self::Solution,
    ) -> Result<NodePtr, DriverError> {
        ctx.alloc(&solution)
    }

    fn parse_puzzle(_allocator: &Allocator, _puzzle: Puzzle) -> Result<Option<Self>, DriverError> {
        Ok(None)
    }

    fn parse_solution(
        allocator: &Allocator,
        solution: NodePtr,
    ) -> Result<Self::Solution, DriverError> {
        Ok(P2Eip712MessageSolution::from_clvm(allocator, solution)?)
    }
}

pub fn get_hash_to_sign(
    layer: &P2Eip712MessageLayer,
    coin_id: Bytes32,
    delegated_puzzle_hash: Bytes32,
) -> Bytes32 {
    /*
    bytes32 messageHash = keccak256(abi.encode(
        typeHash,
        coin_id,
        delegated_puzzle_hash
    ));
    */
    let mut to_hash = Vec::new();
    to_hash.extend_from_slice(&layer.type_hash());
    to_hash.extend_from_slice(&coin_id);
    to_hash.extend_from_slice(&delegated_puzzle_hash);

    let message_hash = keccak256(&to_hash);

    let mut to_hash = Vec::new();
    to_hash.extend_from_slice(&layer.prefix_and_domain_separator());
    to_hash.extend_from_slice(&message_hash);

    keccak256(&to_hash).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chia::consensus::consensus_constants::TEST_CONSTANTS;
    use chia::traits::Streamable;
    use chia_wallet_sdk::{Conditions, Simulator};
    use clvm_traits::clvm_quote;
    use ecdsa::signature::hazmat::PrehashSigner;
    use ecdsa::signature::hazmat::PrehashVerifier;
    use ecdsa::SigningKey;
    use ethers::core::rand::thread_rng;
    use ethers::prelude::*;
    use ethers::signers::LocalWallet;
    use hex::encode;
    use k256::ecdsa::{Signature as K1Signature, VerifyingKey as K1VerifyingKey};

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

        // actual test
        let ctx = &mut SpendContext::new();
        let mut sim = Simulator::new();

        let pubkey = wallet.signer().verifying_key().to_sec1_bytes().to_vec();
        println!("pubkey: {:?}", encode(pubkey.to_vec().clone()));
        let layer = P2Eip712MessageLayer::new(
            TEST_CONSTANTS.genesis_challenge,
            pubkey.to_vec().try_into().unwrap(),
        );
        let coin_puzzle_reveal = layer.construct_puzzle(ctx)?;
        let coin_puzzle_hash = ctx.tree_hash(coin_puzzle_reveal);

        let coin = sim.new_coin(coin_puzzle_hash.into(), 1337);

        let delegated_puzzle_ptr =
            clvm_quote!(Conditions::new().reserve_fee(1337)).to_clvm(&mut ctx.allocator)?;
        let delegated_solution_ptr = ctx.allocator.nil();

        let hash_to_sign = get_hash_to_sign(
            &layer,
            coin.coin_id(),
            ctx.tree_hash(delegated_puzzle_ptr).into(),
        );

        let signature_og: K1Signature = wallet.signer().sign_prehash(&hash_to_sign.to_vec())?;
        let signature: EthSignatureBytes = signature_og.to_vec().try_into().unwrap();

        println!("Coin id: {:}", coin.coin_id());
        let coin_spend = layer.construct_coin_spend(
            ctx,
            coin,
            P2Eip712MessageSolution {
                my_id: coin.coin_id(),
                signature: signature.to_vec().into(),
                delegated_puzzle: delegated_puzzle_ptr,
                delegated_solution: delegated_solution_ptr,
            },
        )?;

        println!("puzzle: {}", encode(coin_spend.puzzle_reveal.to_bytes()?));
        println!("solution: {}", encode(coin_spend.solution.to_bytes()?));
        println!("coin id: {:}", coin_spend.coin.coin_id());
        ctx.insert(coin_spend);

        let verifier =
            K1VerifyingKey::from_sec1_bytes(&wallet.signer().verifying_key().to_sec1_bytes())?;
        assert_eq!(verifier, *wallet.signer().verifying_key());
        let msg = hash_to_sign.to_vec();
        let sig = K1Signature::from_slice(&signature)?;
        assert_eq!(sig, K1Signature::from_slice(&signature_og.to_vec())?);
        let result = verifier.verify_prehash(msg.as_ref(), &sig);
        assert!(result.is_ok());

        sim.spend_coins(ctx.take(), &[])?;

        Ok(())
    }
}
