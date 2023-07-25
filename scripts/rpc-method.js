const {ApiPromise,WsProvider} = require('@polkadot/api');

async function getRpcMethods() {
  const provider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({ provider });

  const rpcMethods = await api.rpc.rpc.methods();

  console.log(rpcMethods);

  process.exit();
}

getRpcMethods().catch(console.error);


