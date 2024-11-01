from eth_account.messages import encode_typed_data

domain = {
    'name': 'Chia Coin Spend',
    'salt': '0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855'
}
types = {
    'ChiaCoinSpend': [
        {'name': 'coin_id', 'type': 'bytes32'},
        {'name': 'delegated_puzzle_hash', 'type': 'bytes32'}
    ]
}

encoded_data = encode_typed_data(
    domain,
    types,
    {
        'coin_id': '0x8b2107b5aee551f03163841793d343bf8e2fdb4dee8629f8f3b90c1ef839c17d',
        'delegated_puzzle_hash': '0x2298d705f78bb3da01f74717d7ae36991fd312e1a597152bd31652b0a5a522a3',
    }
)

print("domainSeparator:", "0x" + encoded_data.header.hex())
print("hashStruct(message):", "0x" + encoded_data.body.hex())