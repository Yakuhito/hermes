# Hermes

Chia EIP-712-based wallet puzzle enabling hardware wallet support (PoC)

## How to use

1. Go to the 'ui' folder and run `npm i --force`. Then, run `npm run dev`.
2. Go to the web interface at `http://localhost:3000`.
3. Connect your Ledger/Trezor to Metamask (if not already connected), and then click 'Connect Wallet' in the UI to connect Metamask to the website.
4. To get an XCH address, the puzzle needs to know your public key. Click the 'Reveal pk' button and sign the "Hello, Chia!" message with your Ledger/Trezor. Copy the hex string that appears after 'Pk: '.
5. In the root directory, run the main application with `cargo run`. Paste the public key from step 4 when you're asked for it.
6. The application will start a simulator network and create a coin for you to spend. Copy the 'coin_id' and 'delegated_puzzle_hash' from the console output.
7. On the website UI, paste the two values, then click 'Generate sig'. Sign the message with your Ledger/Trezor. Copy the hex string that appears after 'Sig: '.
8. Return to the Rust application and paste in your signature from step 7. The application will spend the coin you created. If you see a success message, you're done!

