import { createKeyPairSignerFromBytes, createSolanaRpc, devnet } from "@solana/kit";
import turbWallet from "./turb_wallet.json";

const keypair = await createKeyPairSignerFromBytes(new Uint8Array(turbWallet));
const rpc = createSolanaRpc(devnet("https://api.devnet.solana.com"));

console.log(`Turb Wallet Address: ${keypair.address}`);
const { value: balance } = await rpc.getBalance(keypair.address).send();
console.log(`Balance: ${balance} lamports (${Number(balance) / 1_000_000_000} SOL)`);
