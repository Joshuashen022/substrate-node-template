import { Keyring } from '@polkadot/api';
import { ApiPromise, WsProvider } from '@polkadot/api';
// const { ApiPromise, WsProvider } = require('@polkadot/api');
import { stringToU8a, u8aToHex } from '@polkadot/util';
import fs from 'fs';


var makeTransaction = new Object({
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
    console.log(`newPair: has address ${newPair.address}`);
    
    // (Advanced) add an account with a derivation path (hard & soft)
    // TODO:Error: A soft key was found in the path and is not supported
    // const newDeri = keyring.addFromUri(`${PHRASE}//hard-derived/soft-derived`);
  
    // add a hex seed, 32-characters in length
    const hexPair = keyring.addFromUri('0x1234567890123456789012345678901234567890123456789012345678901234');
    console.log(`hexPair: has address ${hexPair.address}`);

    // add a string seed, internally this is padded with ' ' to 32-bytes in length
    const strPair = keyring.addFromUri('Janice');
    console.log(`strPair: has address ${strPair.address}`);

    // (Advanced, development-only) add with an implied dev seed and hard derivation
    const alice_default = keyring.addFromUri('//Alice', { name: 'Alice default' });
    // console.log(`${alice_default.meta.name}: has address ${alice_default.address}`);// with publicKey [${alice.publicKey}]
    
    const alice = keyring.addFromAddress('5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY', { name: 'Alice' })
    console.log(`${alice.meta.name}: has address ${alice.address}`);// with publicKey [${alice.publicKey}]

    
    console.log(`now we have ${keyring.getPairs().length} keys`);

    return alice
  },

  alice_sign_verify: function(alice){
      // Convert message, sign and then verify
    const message = stringToU8a('this is our message');
    const signature = alice.sign(message);
    const isValid = alice.verify(message, signature, alice.publicKey);

    // Log info
    console.log(`The signature ${u8aToHex(signature)}, is ${isValid ? '' : 'in'}valid`);
  },

  make_a_transfer: async function(api){
    console.log(`make a transfer`);
    const keyring = new Keyring({typr: 'sr25519'}); // default ed25519

    const BOB = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';
    const CHARLIE = '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y';
    // const alice = keyring.addFromUri('//Alice');
    const ALICE_STASH = keyring.addFromAddress('5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY', { name: 'Alice Stash' })
    // console.log(`${ALICE_STASH.meta.name}: has address ${ALICE_STASH.address} with publicKey [${ALICE_STASH.publicKey}]`);

    // Make a transfer from Alice to BOB, waiting for inclusion
    const unsub = await api.tx.balances
    .transfer(BOB, 12345)
    .signAndSend(ALICE_STASH, (result) => {
      console.log(`Current status is ${result.status}`);

      if (result.status.isInBlock) {
        console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
      } else if (result.status.isFinalized) {
        console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
        unsub();
      }
    });
  },

  get_test_account: function(){
    
    const keyring = new Keyring({typr: 'sr25519'}); // default ed25519
    // const input = {"encoded":"tmDWRZMYxcDzgz7Xnc+9fKpbAPNBXaPbu0f1s5D3QLMAgAAAAQAAAAgAAADT0CqO46sIoYqLd+6RL3+kKnI2avhjFrGzCQyliXERUJjtepEkROtrU8Boc9mGUqkCO30lTABujlV8tiEsieYkzkq+UJYMCDEUdtutR4ckSyBNoixtOI1QtQhbD781zf6prf8aUembVtZOw5UYBEEAdNu+V+rIUcl8RPUQJ1cwLEDEwX5KILO2Ebdfm9FvM0iQBzkso1YUBEeqsFKn","encoding":{"content":["pkcs8","sr25519"],"type":["scrypt","xsalsa20-poly1305"],"version":"3"},"address":"5Cw1piEXaSMZoqy5HhyBYua2bYsJ1SFKGiJWALk2qgQza75Z","meta":{"isHardware":false,"name":"test account","tags":[],"whenCreated":1643271962504}};
    const test_account = keyring.addFromMnemonic('//Test', { name: 'Test Account' }); //5EzVqQhKPeKyM4UkbERjZ7AQBsE8Aiag155r9dMVnovDNvW8
    console.log(`${test_account.meta.name}: has address ${test_account.address}`);// with publicKey [${alice.publicKey}]

    console.log(`now we have ${keyring.getPairs().length} keys`);

    return test_account
  },

  transfer_to_charlie_from: async function(api, account){
    const CHARLIE = '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y';
    console.log(`make a transfer to ${CHARLIE}`);
    
    const unsub = await api.tx.balances
    .transfer(CHARLIE, 2)
    .signAndSend(account, (result) => {
      console.log(`Current status is ${result.status}`);

      if (result.status.isInBlock) {
        console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
      } else if (result.status.isFinalized) {
        console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
        unsub();
      }
    });
  }

})

var makeQuery = new Object({
  
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

var getJson = new Object({
  read_json: function(){
    
    // const fs = require('fs');

    // read JSON object from file
    // const res = fs.readFile('test.json', 'utf-8', (err, data) => {
    //     if (err) {
    //         console.log(err);
    //         throw err;
    //     }

    //     // parse JSON object
    //     // const test = JSON.parse(data.toString());

    //     // print JSON object
    //     console.log(data);
    //     return data
    // });
    const [data] = Promise.all([
      fs.readFile('test.json', 'utf-8', (err, data) => {
        if (err) {
            console.log(err);
            throw err;
        }

        // parse JSON object
        // const test = JSON.parse(data.toString());

        // print JSON object
        console.log(data);
        return data
      })
    ]);

    return data
  },

  import_json: function(){
    const data = require('./test.json')
  }

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

  const test_account = await makeTransaction.get_test_account(api);

  // makeTransaction.alice_sign_verify(alice);

  await makeTransaction.transfer_to_charlie_from(api, test_account);
  console.log(`make a transfer to done too`);
}

main().catch(console.error).finally(() => process.exit());