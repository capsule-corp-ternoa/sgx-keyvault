## How to run the demo

1. Install required npm packages

```
cd demo
npm i
```

2. Start the node
```
./target/release/ternoa purge-chain --dev
./target/release/ternoa --dev --tmp --ws-external --rpc-external --rpc-cors all
```

3. Build and start the worker

```
source /opt/intel/sgxsdk/environment
make clean && make && make setup && make run
```

4. Run the demo file
```
node index.js
```