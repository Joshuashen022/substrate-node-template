const { ApiPromise, WsProvider } = require('@polkadot/api');

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
  
  // get_const(api);

  // await make_queries(api);

  // await read_last_block(api);

  // await read_some_block(api);

  await query_subscriptions(api);
}

main().catch(console.error).finally(() => process.exit());

function get_const(api){
  // console.log(api.consts.babe.epochDuration.toNumber());
  // console.log(api.consts.balances.existentialDeposit.toNumber());
  
  // console.log(api.consts.transactionPayment.transactionPayment.toNumber());
  console.log(api.consts.genesisHash);
}

async function make_queries(api){
  // The actual address that we will use
  const ADDR = '5DTestUPts3kjeXSTMyerHihn1uwMfLj8vU8sqF7qYrFabHE';

  // Retrieve the last timestamp
  const now = await api.query.timestamp.now();

  // Retrieve the account balance & nonce via the system module
  const { nonce, data: balance } = await api.query.system.account(ADDR);

  console.log(`${now}: balance of ${balance.free} and a nonce of ${nonce}`);
}

async function read_last_block(api){
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
}

// TODO:NOT WORKING
async function read_some_block(api){
  
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
  
}

// TODO:NOT WORKING
async function query_subscriptions(api){
  // Retrieve the current timestamp via subscription
  const unsub = await api.query.timestamp.now((moment) => {
    console.log(`The last block has a timestamp of ${moment}`);
  });

}

