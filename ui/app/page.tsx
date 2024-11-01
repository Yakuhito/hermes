"use client";

import { useState } from "react";
import { recoverPublicKey } from "viem";
import { useAccount, useSignTypedData } from "wagmi";
import { _TypedDataEncoder } from "@ethersproject/hash";

export default function Home() {
  const { isConnected } = useAccount();
  const { signTypedDataAsync } = useSignTypedData()

  const [pk, setPk] = useState('');
  const [sig, setSig] = useState('');
  const [coinId, setCoinId] = useState('');
  const [delegatedPuzzleHash, setDelegatedPuzzleHash] = useState('');

  return (
    <main className="min-h-screen px-8 py-0 pb-12 flex-1 flex flex-col items-center bg-white">
      <header className="w-full py-4">
        <div className="items-center">
          <div className="hidden sm:inline text-xl font-bold">Example HW Wallet App</div>
        </div>
        <div className="items-center">
          <w3m-button />
        </div>
        <div className="mt-8">
          <button
            className="mt-2 px-4 py-2 w-64 border-2 border-green-500 hover:bg-green-500 hover:text-black font-medium rounded-xl text-green-500"
            onClick={async () => {
              let sig = await signTypedDataAsync({
                domain: {},
                types: {
                  Text: [
                    {name: "message", type: "string"},
                  ]
                },
                primaryType: "Text",
                message: {
                  message: "Hello, Chia!"
                }
              })

              const msgHash = _TypedDataEncoder.hash({}, {
                Text: [
                  {name: "message", type: "string"},
                ]
              }, {
                message: "Hello, Chia!"
              });
            
              console.log({ sig, msgHash })
              setPk(await recoverPublicKey({ hash: msgHash as `0xstring`, signature: sig }))
            }}
          >Reveal pk</button>
          <div className="mt-8">Pk: {pk}</div>
        </div>
        <div className="mt-8 flex flex-col items-center">
          <input
            type="text"
            value={coinId}
            onChange={(e) => setCoinId(e.target.value)}
            placeholder="Enter coin_id"
            className="mt-2 px-4 py-2 border-2 border-gray-300 rounded-xl w-64"
          />
          <input
            type="text"
            value={delegatedPuzzleHash}
            onChange={(e) => setDelegatedPuzzleHash(e.target.value)}
            placeholder="Enter delegated_puzzle_hash"
            className="mt-2 px-4 py-2 border-2 border-gray-300 rounded-xl w-64"
          />
          <button
            className="mt-2 px-4 py-2 w-64 border-2 border-green-500 hover:bg-green-500 hover:text-black font-medium rounded-xl text-green-500"
            onClick={async () => {
              let domain =  {
                name: "Chia Coin Spend",
                salt: "0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
              };
              let types = {
                ChiaCoinSpend: [
                  {name: "coin_id", type: "bytes32"},
                  {name: "delegated_puzzle_hash", type: "bytes32"}
                ]
              };
              let message = {
                coin_id: coinId as `0x${string}`,
                delegated_puzzle_hash: delegatedPuzzleHash as `0x${string}`
              };

              const msgHash = _TypedDataEncoder.hash(domain, types, message);

              console.log({ msgHash })
              let sig = await signTypedDataAsync({
                domain: {
                  name: "Chia Coin Spend",
                  salt: "0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                },
                types,
                primaryType: "ChiaCoinSpend",
                message,
              })

              console.log({ sig })
              setSig(sig)
            }}
          >Generate sig</button>
          <div className="mt-8">Sig: {sig}</div>
        </div>
      </header>
    </main>
  );
}