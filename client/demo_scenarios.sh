#!/bin/bash

# setup:
# run all on localhost:
#   ternoa-chain purge-chain --dev
#   ternoa-chain --dev -lruntime=debug
#   rm chain_relay_db.bin
#   substratee-worker init_shard
#   substratee-worker shielding-key
#   substratee-worker signing-key
#   substratee-worker run
#
# then run this script

# usage:
#  demo_scenarios.sh -p <NODEPORT> -P <WORKERPORT> -t <TEST_BALANCE_RUN> -m file
#
# TEST_BALANCE_RUN is either "first" or "second"
# if -m file is set, the mrenclave will be read from file

# using default port if none given as arguments
NPORT=${NPORT:-9944}
RPORT=${RPORT:-2000}

echo "Using node-port ${NPORT}"
echo "Using worker-rpc-port ${RPORT}"

AMOUNTSHIELD=50000000000
AMOUNTTRANSFER=40000000000


CLIENT="./ternoa-client -p ${NPORT} -P ${RPORT}"

if [ "$READMRENCLAVE" = "file" ]
then
    read MRENCLAVE <<< $(cat ~/mrenclave.b58)
    echo "Reading MRENCLAVE from file: ${MRENCLAVE}"
else
    # this will always take the first MRENCLAVE found in the registry !!
    read MRENCLAVE <<< $($CLIENT list-workers | awk '/  MRENCLAVE: / { print $2; exit }')
    echo "Reading MRENCLAVE from worker list: ${MRENCLAVE}"
fi
[[ -z $MRENCLAVE ]] && { echo "MRENCLAVE is empty. cannot continue" ; exit 1; }

echo ""
echo "* Create a new file to encrypt"
INPUTFILENAME= input_file
INPUTFILE = ${INPUTFILENAME}.txt
touch  ${INPUTFILE}
echo "These are very important data" > INPUTFILE
echo ""

echo ""
echo "* Create a new incognito account for Alice"
ICGACCOUNTALICE=//AliceIncognito
echo "  Alice's incognito account = ${ICGACCOUNTALICE}"
echo ""

echo "* Create a new incognito account for Bob"
ICGACCOUNTBOB=$(${CLIENT} trusted new-account --mrenclave ${MRENCLAVE})
echo "  Bob's incognito account = ${ICGACCOUNTBOB}"
echo ""

echo "* Shield ${AMOUNTSHIELD} tokens to Alice's incognito account"
${CLIENT} shield-funds //Alice ${ICGACCOUNTALICE} ${AMOUNTSHIELD} ${MRENCLAVE} ${WORKERPORT}
echo ""

echo "* Shield ${AMOUNTSHIELD} tokens to Bob's incognito account"
${CLIENT} shield-funds //Bob ${ICGACCOUNTBOB} ${AMOUNTSHIELD} ${MRENCLAVE} ${WORKERPORT}
echo ""

echo "* Waiting 10 seconds"
sleep 10
echo ""

echo "Get balance of Alice's incognito account"
${CLIENT} trusted balance ${ICGACCOUNTALICE} --mrenclave ${MRENCLAVE}
echo ""

echo "Get balance of Bob's incognito account"
${CLIENT} trusted balance ${ICGACCOUNTBOB} --mrenclave ${MRENCLAVE}
echo ""

echo "Alice creates a capsule"
echo "Encrypt file with aes256"
${CLIENT} encrypt ${INPUTFILE}

CIPHERFILE= ${INPUTFILENAME}.ciphertext
KEYFILE=${INPUTFILENAME}.aes256

echo "Create a new NFT "
NFTID= $(${CLIENT} nft create ${ICGACCOUNTALICE} ${CIPHERFILE}
echo "Alice NFT id = ${NFTID}"

echo "All urls registered in the enclave registry"
${CLIENT} trusted keyvault list ${ICGACCOUNTALICE} ${CIPHERFILE} --mrenclave ${MRENCLAVE}
URLSFILE="./bin/my_keyvaults/keyvault_pool.txt"

# Load file into array URLS
URLS=()
readarray URLS < $URLSFILE

echo "Urls found "
let i= 0
while (( ${#URLS[@]} > i )); do
    echo "${URLS[i++]}\n"
done
echo " "
URLSNUM = ${#URLS[@]}
SHAMITHRESHOLD = $URLSNUM
if [ URLSNUM = 0 ]
then
    echo "No urls are registered. Cannot continue"; exit 1;
else
  $SHAMITHRESHOLD = $(((2*URLSNUM +1)/3))
fi
echo "Threshold to recover secret : ${SHAMITHRESHOLD}"
echo " "

echo "Keyvault provision"
${CLIENT} trusted keyvault provision ${ICGACCOUNTALICE} ${URLS} ${SHAMITHRESHOLD} ${NFTID} --mrenclave ${MRENCLAVE}
URLSNFTFILE="./bin/my_keyvaults/keyvault_nft_urls_${NFTID}.txt"
echo "NFT Urls "
echo "$(<URLSNFTFILE)"
echo " "

echo "Get balance of Alice's incognito account"
${CLIENT} trusted balance ${ICGACCOUNTALICE} --mrenclave ${MRENCLAVE}
echo ""

echo "Alice transfers ownership to Bob"
${CLIENT} nft transfer ${ICGACCOUNTALICE} ${ICGACCOUNTBOB} ${NFTID}
echo ""

echo "Bob open capsule"

# Load file into array URLS
URLSNFT=()
readarray URLSNFT < URLSNFTFILE

KEYSHAREFILE=""
let i= 0
while (( ${#URLSNFT[@]} > i )); do
    CURRENTURL = ${URLSNFT[i++]}
    echo "${CURRENTURL}\n"
    ${CLIENT} trusted keyvault check ${ICGACCOUNTBOB} ${NFTID} ${CURRENTURL}  --mrenclave ${MRENCLAVE}
    ${CLIENT} trusted keyvault get ${NFTID} ${ICGACCOUNTBOB}  ${CURRENTURL}  --mrenclave ${MRENCLAVE}
done

${CLIENT} nft decrypt ${ICGACCOUNTBOB} ${CIPHERFILE} ${KEYSHAREFILE}

#adversary scenario 1
