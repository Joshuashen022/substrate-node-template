import { Keyring } from '@polkadot/api';
import { ApiPromise, WsProvider } from '@polkadot/api';
// const { ApiPromise, WsProvider } = require('@polkadot/api');
import { stringToU8a, u8aToHex } from '@polkadot/util';


var module1 = new Object({
  transaction: async function(api){
  

    // Sign and send a transfer from Alice to Bob
    
    const alice = get_alice();
  
    const txHash = await api.tx.balances
    .transfer(BOB, 12345)
    .signAndSend(alice);
  
    // Show the hash
    console.log(`Submitted with hash ${txHash}`);
  },
  
  get_alice: function(){
  
    const keyring = new Keyring({typr: 'sr25519'}); // default ed25519
  
    // Some mnemonic phrase
    const PHRASE = 'entire material egg meadow latin bargain dutch coral blood melt acoustic thought';
  
    // Add an account, straight mnemonic
    const newPair = keyring.addFromUri(PHRASE);
  
    // (Advanced) add an account with a derivation path (hard & soft)
    // TODO:Error: A soft key was found in the path and is not supported
    // const newDeri = keyring.addFromUri(`${PHRASE}//hard-derived/soft-derived`);
  
    // add a hex seed, 32-characters in length
    const hexPair = keyring.addFromUri('0x1234567890123456789012345678901234567890123456789012345678901234');
  
    // add a string seed, internally this is padded with ' ' to 32-bytes in length
    const strPair = keyring.addFromUri('Janice');
  
    // (Advanced, development-only) add with an implied dev seed and hard derivation
    const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });
  
    console.log(`${alice.meta.name}: has address ${alice.address} with publicKey [${alice.publicKey}]`);
  
    return alice
  },

  alice_sign_verify: function(alice){
    // Convert message, sign and then verify
  const message = stringToU8a('this is our message');
  const signature = alice.sign(message);
  const isValid = alice.verify(message, signature, alice.publicKey);

  // Log info
  console.log(`The signature ${u8aToHex(signature)}, is ${isValid ? '' : 'in'}valid`);
  }
})

var module2 = new Object({
  
  get_const:function(api){
    // console.log(api.consts.babe.epochDuration.toNumber());
    // console.log(api.consts.balances.existentialDeposit.toNumber());
    
    // console.log(api.consts.transactionPayment.transactionPayment.toNumber());
    console.log(api.consts.genesisHash);
  },

  make_queries: async function (api){
    // The actual address that we will use
    const ADDR = '5DTestUPts3kjeXSTMyerHihn1uwMfLj8vU8sqF7qYrFabHE';

    // Retrieve the last timestamp
    const now = await api.query.timestamp.now();

    // Retrieve the account balance & nonce via the system module
    const { nonce, data: balance } = await api.query.system.account(ADDR);

    console.log(`${now}: balance of ${balance.free} and a nonce of ${nonce}`);
  },

  read_last_block: async function(api){
    // The actual address that we will use
    const ADDR = '5DTestUPts3kjeXSTMyerHihn1uwMfLj8vU8sqF7qYrFabHE';

    // Retrieve last block timestamp, account nonce & balances
    const [now, { nonce, data: balance }] = await Promise.all([
      api.query.timestamp.now(),
      api.query.system.account(ADDR)
    ]);

    console.log(`${now}: balance of ${balance.free} and a nonce of ${nonce}`);
    
    // Retrieve the chain name
    const chain = await api.rpc.system.chain();

    // Retrieve the latest header
    const lastHeader = await api.rpc.chain.getHeader();

    // Log the information
    console.log(`${chain}: last block #${lastHeader.number} has hash ${lastHeader.hash}`);
  },

// TODO:NOT WORKING
  read_some_block: async function(api){
    
    // Subscribe to the new headers
    await api.rpc.chain.subscribeNewHeads((lastHeader) => {
      console.log(`${chain}: last block #${lastHeader.number} has hash ${lastHeader.hash}`);
    });

    console.log(`You are connected to the International`);
    let count = 0;

    // Subscribe to the new headers
    const unsubHeads = await api.rpc.chain.subscribeNewHeads((lastHeader) => {
      console.log(`${chain}: last block #${lastHeader.number} has hash ${lastHeader.hash}`);

      if (++count === 10) {
        unsubHeads();
      }
    });

    console.log(`You are connected to the International ${count}`);
    const unsub = await api.derive.chain.subscribeNewHeads((lastHeader) => {
      console.log(`#${lastHeader.number} was authored by ${lastHeader.author}`);
    });
    
  },

// TODO:NOT WORKING
  query_subscriptions: async function query_subscriptions(api){
    // Retrieve the current timestamp via subscription
    const unsub = await api.query.timestamp.now((moment) => {
      console.log(`The last block has a timestamp of ${moment}`);
    });

  },

})

async function main () {
  // Initialise the provider to connect to the local node
  const provider = new WsProvider('ws://127.0.0.1:9944');

  // Create the API and wait until ready
  const api = await ApiPromise.create({ provider });

  // Retrieve the chain & node information information via rpc calls
  const [chain, nodeName, nodeVersion] = await Promise.all([
    api.rpc.system.chain(),
    api.rpc.system.name(),
    api.rpc.system.version()
  ]);
  
  console.log(`You are connected to chain  -${chain} using -${nodeName} v-${nodeVersion}`);
  
  const alice = await module1.get_alice(api);

  module1.alice_sign_verify(alice);
  
}

main().catch(console.error).finally(() => process.exit());