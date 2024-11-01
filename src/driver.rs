use chia::{
    clvm_utils::TreeHash,
    protocol::{Bytes32, Coin},
    puzzles::standard::StandardSolution,
};
use chia_wallet_sdk::{Conditions, DriverError, Spend, SpendContext};
use clvmr::NodePtr;
use hex_literal::hex;

pub const P2_EIP712_MESSAGE_PUZZLE: [u8; 226] = hex!("ff02ffff01ff02ffff03ffff22ffff09ff17ffff0cffff3eff5f80ffff010cffff01208080ffff8413d61f00ff5fffff3eff05ffff3eff0bff2fffff02ff06ffff04ff02ffff04ff82017fff808080808080ff81bf8080ffff01ff04ffff04ff04ffff04ff2fff808080ffff02ff82017fff8202ff8080ffff01ff08ffff01846e6f70658080ff0180ffff04ffff01ff46ff02ffff03ffff07ff0580ffff01ff0bffff0102ffff02ff06ffff04ff02ffff04ff09ff80808080ffff02ff06ffff04ff02ffff04ff0dff8080808080ffff01ff0bffff0101ff058080ff0180ff018080");
pub const P2_EIP712_MESSAGE_PUZZLE_HASH: TreeHash = TreeHash::new(hex!(
    "
    d96d95b0e5184e43dd135297b385afdd39ecbbba6e16fc9240b9d3089a932360
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct P2Eip712MessageLayer {
    pub genesis_challenge: Bytes32,
    pub address: [u8; 20],
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, ToClvm, FromClvm)]
// #[clvm(solution)]
// pub struct StandardSolution<P, S> {
//     pub original_public_key: Option<PublicKey>,
//     pub delegated_puzzle: P,
//     pub solution: S,
// }

// impl P2Eip712MessageLayer {
//     pub fn new(genesis_challenge: Bytes32, address: [u8; 20]) -> Self {
//         Self {
//             genesis_challenge,
//             address,
//         }
//     }

//     pub fn spend(
//         &self,
//         ctx: &mut SpendContext,
//         coin: Coin,
//         conditions: Conditions,
//     ) -> Result<(), DriverError> {
//         let spend = self.spend_with_conditions(ctx, conditions)?;
//         ctx.spend(coin, spend)
//     }

//     pub fn delegated_inner_spend(
//         &self,
//         ctx: &mut SpendContext,
//         spend: Spend,
//     ) -> Result<Spend, DriverError> {
//         self.construct_spend(
//             ctx,
//             StandardSolution {
//                 original_public_key: None,
//                 delegated_puzzle: spend.puzzle,
//                 solution: spend.solution,
//             },
//         )
//     }
// }

// impl Layer for StandardLayer {
//     type Solution = StandardSolution<NodePtr, NodePtr>;

//     fn construct_puzzle(&self, ctx: &mut SpendContext) -> Result<NodePtr, DriverError> {
//         let curried = CurriedProgram {
//             program: ctx.standard_puzzle()?,
//             args: StandardArgs::new(self.synthetic_key),
//         };
//         ctx.alloc(&curried)
//     }

//     fn construct_solution(
//         &self,
//         ctx: &mut SpendContext,
//         solution: Self::Solution,
//     ) -> Result<NodePtr, DriverError> {
//         ctx.alloc(&solution)
//     }

//     fn parse_puzzle(allocator: &Allocator, puzzle: Puzzle) -> Result<Option<Self>, DriverError> {
//         let Some(puzzle) = puzzle.as_curried() else {
//             return Ok(None);
//         };

//         if puzzle.mod_hash != STANDARD_PUZZLE_HASH {
//             return Ok(None);
//         }

//         let args = StandardArgs::from_clvm(allocator, puzzle.args)?;

//         Ok(Some(Self {
//             synthetic_key: args.synthetic_key,
//         }))
//     }

//     fn parse_solution(
//         allocator: &Allocator,
//         solution: NodePtr,
//     ) -> Result<Self::Solution, DriverError> {
//         Ok(StandardSolution::from_clvm(allocator, solution)?)
//     }
// }

// impl SpendWithConditions for StandardLayer {
//     fn spend_with_conditions(
//         &self,
//         ctx: &mut SpendContext,
//         conditions: Conditions,
//     ) -> Result<Spend, DriverError> {
//         let delegated_puzzle = ctx.alloc(&clvm_quote!(conditions))?;
//         self.construct_spend(
//             ctx,
//             StandardSolution {
//                 original_public_key: None,
//                 delegated_puzzle,
//                 solution: NodePtr::NIL,
//             },
//         )
//     }
// }

// impl ToTreeHash for StandardLayer {
//     fn tree_hash(&self) -> TreeHash {
//         StandardArgs::curry_tree_hash(self.synthetic_key)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::core::rand::thread_rng;
    use ethers::prelude::*;
    use ethers::signers::{LocalWallet, Signer};
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
        // println!("keccak256(Public Key): 0x{:}", encode(output));
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

        /*
        ;; bytes32 domainSeparator = keccak256(abi.encode(
        ;;    keccak256("EIP712Domain(string name, bytes32 salt)"),
        ;;    keccak256(bytes("Chia Coin Spend")),
        ;;    salt
        ;; ));
         */
        let type_hash = keccak256(b"EIP712Domain(string name,bytes32 salt)");
        let domain_separator = keccak256(ethers::abi::encode(&[
            ethers::abi::Token::FixedBytes(type_hash.to_vec()),
            ethers::abi::Token::FixedBytes(keccak256("Chia Coin Spend").to_vec()),
            ethers::abi::Token::FixedBytes(
                hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").to_vec(),
            ),
        ]));

        // let domain_separator = keccak256(&to_hash);
        println!("Domain Separator: 0x{:}", encode(domain_separator));

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
        println!("hashStruct(message): 0x{:}", encode(message_hash));

        let mut to_hash = Vec::new();
        to_hash.extend_from_slice(&[0x19, 0x01]); // "\x19\x01",
        to_hash.extend_from_slice(&domain_separator);
        to_hash.extend_from_slice(&message_hash);

        let hash_to_sign = keccak256(&to_hash);
        println!(
            "Hash To Sign (hand-calculated): 0x{:}",
            encode(hash_to_sign)
        );

        Ok(())
    }
}
