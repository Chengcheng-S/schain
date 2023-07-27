const {ApiPromise,WsProvider} = require('@polkadot/api');

async function TestFcuntion() {
  // const provider = new WsProvider('ws://localhost:9944');
  // const api = await ApiPromise.create({ provider });
  const api = await connetToChain();

  let count = 0;
  const chain = await api.rpc.system.chain();

  const unsubHeads = await api.derive.chain.subscribeNewHeads((lastHeader)=>{
    console.log(`${chain}: last block #: ${lastHeader.number},hash #: ${lastHeader.hash}`);
    
    if (++count == 10){
      unsubHeads();
      process.exit();
    }
  });

  // console.log(runtimeMetadata); 
}

async function getapiMethods(){
  const provider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({ provider });
  const rpcMethods = await api.rpc.rpc.methods();
  console.log(rpcMethods);
  process.exit();
}

// getapiMethods();



async function getapidata(){
  const api = await connetToChain();
  console.log(await api.rpc.system.version());
  const smulitisig = await api.query.smultisigRpc.multisigMembers();  
  console.log(smulitisig);
}
getapidata();

async function connetToChain(){
    const provider = new WsProvider('ws://localhost:9944');
    const api= await ApiPromise.create({provider});
    return api;
}


// TestFcuntion();
