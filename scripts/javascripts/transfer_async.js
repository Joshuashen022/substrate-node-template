import { Keyring } from '@polkadot/api';
import { ApiPromise, WsProvider } from '@polkadot/api';
// const { ApiPromise, WsProvider } = require('@polkadot/api');
import { stringToU8a, u8aToHex } from '@polkadot/util';

function make_a_transfer(api) {

    const keyring = new Keyring({typr: 'sr25519'}); // default ed25519
    const test_account = keyring.addFromMnemonic('//Test', { name: 'Test Account' }); //5EzVqQhKPeKyM4UkbERjZ7AQBsE8Aiag155r9dMVnovDNvW8
    console.log(`${test_account.meta.name}: has address ${test_account.address}`);// with publicKey [${alice.publicKey}]
    console.log(`now we have ${keyring.getPairs().length} keys`);
    
    const CHARLIE = '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y';
    console.log(`make a transfer to ${CHARLIE}`);

    const promise = new Promise(function(resolve, reject){
        // do something cost time
        
        const status = api.tx.balances
        .transfer(CHARLIE, 10000)
        .signAndSend(test_account, (result) => {
            // console.log(`Current status is ${result.status}`);
            
            if (result.status.isReady){
                console.log('ready');
            } else if (result.status.isInBlock) {
                console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
                
                // return value and exit promise
                resolve(result.status) 
            } 
        });
        
        

    })

    return promise

}

async function asyncCall() {
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

    const result = await make_a_transfer(api);
    console.log(`${result}`); // {"inBlock":"0xc84e9..."}
    console.log('program exit');
    process.exit();
}
  
asyncCall();
