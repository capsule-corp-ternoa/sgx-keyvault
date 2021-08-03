#  SgxKeyvault Demo Tutorial
 How to Create a Capsule

*this demo is intended to be an acceptance test of the Proof of Concept. The underlying features are not production-ready.*

In the following demo we show three different scenarios:
* Success scenario: Alice creates a file locally, encrypts it and creates a capsule onchain. The Keyvaults offchain registries register the capsule by importing the onchain block. Alice mutates the content of the capsule onchain and it is checked if the keyvaults mutate the local regsitry as well. Alice then creates shamir shares, the number depends on the number of keyvault registered. She provisions all keyvaults with the created shares. After successful provision, she transfers the ownership of the NFT to Bob. Bob then successfully retrieves the keys from the keyvaults and decrypts the previously encrypted file.
* Adversary scenario 1: Alices creates a capsule and provisions the keyvaults with the generated shamir key shares. Bob, not the owner of the capsule, tries to retrieve the keys, but fails to do so.
* Adversary scenario 2: Alices creates a capsule. Bob, not the owner of the capsule, tries to provision shamir key shares to the keyvaults, but gets rejected by them.


## Setup

Build worker, client and node in our docker:

```bash
# get the docker image
# check for updates on https://hub.docker.com/repository/docker/scssubstratee/substratee_dev
docker pull scssubstratee/substratee_dev:1804-2.12-1.1.3-001

# create a dedicated demo directory
mkdir demo && cd demo

# clone and build the node
git clone https://github.com/capsule-corp-ternoa/chain
cd chain
git fetch origin add-skip-ra-feature
git checkout add-skip-ra-feature
# initialize wasm build environment
./scripts/init.sh
# build the node
cargo build --release --features skip-ias-check
# another 10min
cd ..

#(optional, in case of no sgx available) start the docker container (with sgx support)
docker run -it -v $(pwd):/root/work scssubstratee/substratee_dev:1804-2.12-1.1.3-001 /bin/bash
cd work

# clone and build the worker and the client
git clone https://github.com/capsule-corp-ternoa/sgx-keyvault.git
cd sgx-keyvault
git fetch origin develop
git checkout develop
rustup target add wasm32-unknown-unknown
# With docker
SGX_MODE=SW make
# Without docker
make
# this might take 10min+ on a fast machine
```

## Launch worker and node in terminal 1
```bash
cd bin && touch spid.txt key.txt
cd ..
./local-setup/launch.py ./local-setup/simple-config.json
```
wait a little until worker 3 has been launched

## Open a second terminal to run demo
Open a new bash session in a new terminal.

```bash
cd demo
# If you work with docker: exec into the running container:
docker exec -it [container-id] bash
cd work
# run the script with:
cd sgx-keyvault/local-setup
./run_demo.sh
```

you can remove the tmux session of the script by running
```bash
tmux kill-session -t substratee_logger
```

## Cleanup for docker
The files created in the docker container belong to `root`. This can make it impossible to delete them on your host system. We now give them back to your standard user. (Alternatively, you can just delete everything in `work`)

Note: This step is optional.

```bash
cd /root/work
ls -la

# write down the numbers on the line containing '.'
# example output: drwxrwxr-x 17 1002 1002   4096 Nov  2 15:10 .
#  where the numbers are 1002 (NUMBER1) and 1002 (NUMBER2)

# give all files back to the external user
chown -R <NUMBER1>:<NUMBER2> substraTEE-worker substraTEE-node
```
