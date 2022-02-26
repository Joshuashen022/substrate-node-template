import { Keyring } from '@polkadot/api';
import { ApiPromise, WsProvider } from '@polkadot/api';


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
    const keyring = new Keyring( {type: 'sr25519'});
    console.log(keyring.createFromUri('//Alice').address);
    console.log(keyring.createFromUri('//Bob').address);
    console.log(keyring.createFromUri('//Charlie').address);
    console.log(keyring.createFromUri('//Dave').address);

    const keyring2 = new Keyring( { type: 'sr25519'});// ss58Format:'babe',
    console.log(keyring.createFromUri('//Alice').address);

    console.log('program exit');
    process.exit();
}

main();