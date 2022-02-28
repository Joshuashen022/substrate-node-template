import { Keyring } from '@polkadot/api';
import { ApiPromise, WsProvider } from '@polkadot/api';

function make_a_transfer(api) {

    const keyring = new Keyring({type: 'sr25519'}); 
    const ALICE = keyring.createFromUri('//Alice');
    const CHARLIE = '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y';
    console.log(`${ALICE.address} make a transfer to ${CHARLIE}`);

    const promise = new Promise(function(resolve, reject){
        // do something cost time
        const status = api.tx.balances
        .transfer(CHARLIE, 10000)
        .signAndSend(ALICE, (result) => {
            if (result.status.isReady){
                console.log('node has accepted our transaction proposal waiting on chain...');
            } else if (result.status.isInBlock) {
                console.log(`Transaction successfully included at block Hash:${result.status.asInBlock}`);
                resolve(result.status)
            }
        });
    })
    return promise

}

async function main() {
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
    if (result.isInBlock){
        console.log('Test success'); // {"inBlock":"0xc84e9..."}
    }

    console.log('program exit');
    process.exit();
}

main();
