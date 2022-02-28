import { Keyring } from '@polkadot/api';
import fs from 'fs';
import { ApiPromise, WsProvider } from '@polkadot/api';
// const { ApiPromise, WsProvider } = require('@polkadot/api');
// import { stringToU8a, u8aToHex } from '@polkadot/util';

function Key(){
    this.secret_phrase = '';
    this.secret_seed = '';
    this.public_key_hex = '';
    this.account_id = '';
    this.public_key_ss58 = '';
    this.ss58_address = '';

    this.generate = function(phrase, seed, pkh, id, pks, address){
        this.secret_phrase = phrase;
        this.secret_seed = seed;
        this.public_key_hex = pkh;
        this.account_id = id;
        this.public_key_ss58 = pks;
        this.ss58_address = address;
    }
    this.is_empty = function(){
        if (this.secret_phrase == '') {
            return true 
        }
        if (this.secret_seed == '') {
            return true 
        }
        if (this.public_key_hex == '') {
            return true 
        }
        if (this.account_id == '') {
            return true 
        }
        if (this.public_key_ss58 == '') {
            return true 
        }
        if (this.ss58_address == '') {
            return true 
        }
        return false
    }
}

function read_keys(){
    const promise = new Promise(function(resolve, reject){
        fs.readFile('keys.data', (err, data) =>{
            if (err) { reject(err)}
            else {
                // console.log(data.toString());
                resolve(data)
            };
        })
    })
    return promise
}

function chunk (array, chunk_size) {
    const chunks = [];
    const items = [].concat(...array);

    while (items.length){
        chunks.push(
            items.splice(0, chunk_size)
        )
    }
    return chunks;
}

async function get_local_keyring() {

    const content = await read_keys();
    const lines = content.toString().split("\n");

    const keys_lines = chunk(lines, 6);
    var keyring = [];
    for (const key_line of keys_lines){
        if (key_line.length == 6){
            var key = new Key();
            
            const phrase = key_line[0].substr(21);
            const seed = key_line[1].substr(21);
            const pkh = key_line[2].substr(21);
            const id = key_line[3].substr(21);
            const pks = key_line[4].substr(21);
            const address = key_line[5].substr(21);
            key.generate(phrase, seed, pkh, id, pks, address);
            keyring.push(key);
        }
    }
    return keyring;
}

function make_a_transfer(api) {
    
    const keyring = new Keyring( {type: 'sr25519'});
    const alice = keyring.createFromUri('//Alice');
    console.log(`Sign address ${alice.address}`);

    
    const promise = new Promise(function(resolve, reject){
        // do something cost time

        const status = api.tx.session
        .helloWorld()
        .signAndSend(alice, (result) => {
            // console.log(`Current status is ${result.status}`);

            if (result.status.isReady){
                console.log('node has accepted our transaction proposal waiting on chain...');
            } else if (result.status.isInBlock) {
                console.log(`Transaction successfully included at block Hash:${result.status.asInBlock}`);
                // return value and exit promise
                resolve(result.status)
            }
        });

    })

    return promise

}

async function main() {
    const local_keys = await get_local_keyring();
    console.log(`we have ${local_keys.length} local keys`);
    
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
