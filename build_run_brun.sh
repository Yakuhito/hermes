git clone -b keccak --single-branch https://github.com/Chia-Network/clvm_tools_rs.git
cd clvm_tools_rs
cargo build --release
mv target/release/run ..
mv target/release/brun ..
cd ..
rm -rf clvm_tools_rs
