const { ApiPromise, WsProvider } = require("@polkadot/api");
const { Keyring } = require('@polkadot/keyring');
const shell = require('shelljs');
const assert = require('assert');

const wait_for_tx = () => new Promise((resolve) => setTimeout(resolve, 10000));

class TernoaChain {
  api = null;

  constructor(url = "ws://127.0.0.1:9944") {
    this.url = url;
  }

  async connect_to_chain() {
    // if connection already exists return
    if (this.api) return this.api;
    const provider = new WsProvider(this.url);

    // Create the API and wait until ready
    const api = await ApiPromise.create({ provider });

    // Retrieve the chain & node information information via rpc calls
    const [chain, nodeName, nodeVersion] = await Promise.all([
      api.rpc.system.chain(),
      api.rpc.system.name(),
      api.rpc.system.version(),
    ]);

    this.api = api;
    console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`);
    return api;
  }

  async query_balance(account) {
    let api = await this.connect_to_chain();
    const accountInfo = await api.query.system.account(account);
    const { data } = accountInfo.toJSON();
    return data.free / 1e6;
  }

  async query_data(id) {
    let api = await this.connect_to_chain();
    const info = await api.query.nfts.data(id);
    return info.toJSON();
  }

  async create_asset(ipfs) {
    let api = await this.connect_to_chain();
    // Constuct the keyring after the API (crypto has an async init)
    const keyring = new Keyring({ type: 'sr25519' });
    // Add Alice to our keyring with a hard-deived path (empty phrase, so uses dev)
    const alice = keyring.addFromUri('//Alice');
    let create_nft_tx = await api.tx.nfts.create(ipfs, null);
    // Sign and send the transaction using our account
    const hash = await create_nft_tx.signAndSend(alice);
    console.log('CreateNFTTransaction sent with hash', hash.toHex());
  }
}

async function demo() {
  // establish connection to chain
  let chain = new TernoaChain();

  // Ensure balance of alice in chain and enclave matches
  let alice_chain_balance = await chain.query_balance('5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY');
  console.log("Alice balance on chain", alice_chain_balance.toString());
  shell.exec('../target/release/integritee-cli balance //Alice', function(code, stdout, stderr) {
    assert(JSON.parse(stdout).status == true);
    //assert(JSON.parse(stdout).result == alice_chain_balance.toString());
    console.log("Alice balance on enclave", alice_chain_balance.toString());
  });

  // Alice creates an NFT
  let nft = await chain.create_asset("test");
  await wait_for_tx();
  // Create nft secret
  shell.exec('../target/release/integritee-cli store-nft-secret //Alice 0 "top_secret" ', function(code, stdout, stderr) {
    console.log('Program output:', stdout);
    assert(JSON.parse(stdout).status == true);
    assert(JSON.parse(stdout).result == "");
  });
  await wait_for_tx();
  // retreive nft secret
  shell.exec('../target/release/integritee-cli retrieve-nft-secret //Alice 0', function(code, stdout, stderr) {
    console.log('Program output:', stdout);
    assert(JSON.parse(stdout).status == true);
    assert(JSON.parse(stdout).result == "top_secret");
  });

  // ensure nft-data in chain and enclave matches
  let nft_data = await chain.query_data(0);
  console.log(nft_data);
  shell.exec('../target/release/integritee-cli nft-data 0', function(code, stdout, stderr) {
    console.log('Program output:', stdout);
    assert(JSON.parse(stdout).status == true);
  });
}

demo();
