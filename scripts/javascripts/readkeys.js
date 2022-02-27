import fs from 'fs';

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
    const promist = new Promise(function(resolve, reject){
        fs.readFile('keys', (err, data) =>{
            if (err) { reject(err)}
            else {
                // console.log(data.toString());
                resolve(data)
            };
        })
    })
    return promist
}

function check_input(input, index) {
    if (index %6 ==0 && input.search('Secret phrase') > -1) {
        return true
    }
    if (index %6 ==1 && input.search('Secret seed') > -1) {
        return true
    }
    if (index %6 ==2 && input.search('Public key (hex)') > -1) {
        return true
    }
    if (index %6 ==3 && input.search('Account ID') > -1) {
        return true
    }
    if (index %6 ==4 && input.search('Public key (SS58)') > -1) {
        return true
    }
    if (index %6 ==5 && input.search('SS58 Address') > -1) {
        return true
    }
    return false
}

function input(key, index, content) {
    if (index % 6 == 0) {
        key.secret_phrase = content;
    }
    if (index % 6 == 1) {
        key.secret_seed = content;
    }
    if (index % 6 == 2) {
        key.public_key_hex = content;
    }
    if (index % 6 == 3) {
        key.account_id = content;
    }
    if (index % 6 == 4) {
        key.public_key_ss58 = content;
    }
    if (index % 6 == 5) {
        key.ss58_address = content;
    }
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

function add_to_keyring(lines) {
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

async function main() {
    const content = await read_keys();
    const lines = content.toString().split("\n");
    const keyring = add_to_keyring(lines);
    console.log(keyring);
}

main();