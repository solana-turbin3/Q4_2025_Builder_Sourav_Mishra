import bs58 from 'bs58';
import { writeFileSync } from 'fs';

// Get private key from command line argument
const privateKey = process.argv[2];

if (!privateKey) {
  console.error('Error: Private key is required');
  console.log('Usage: tsx phantom-to-wallet.ts <base58-private-key>');
  process.exit(1);
}

try {
  const walletBytes = bs58.decode(privateKey);
  const walletArray = Array.from(walletBytes);

  console.log('Wallet byte array:');
  console.log(JSON.stringify(walletArray));

  // Save to file
  writeFileSync('./turb_wallet.json', JSON.stringify(walletArray));
  console.log('\nSaved to turb_wallet.json');
} catch (error) {
  console.error('Error:', error);
  console.log('\nMake sure you provided a valid base58-encoded private key');
  process.exit(1);
}
