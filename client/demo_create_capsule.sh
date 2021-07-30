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
while getopts ":p:t:" opt; do
    case $opt in
        t)
            TEST=$OPTARG
            ;;
        p)
            NPORT=$OPTARG
            ;;
    esac
done

echo "Using node-port ${NPORT}"

CLIENT="../bin/ternoa-client -p ${NPORT}"
ALICE="//Alice"
BOB="//BOB"

read MRENCLAVE <<< $($CLIENT list-workers | awk '/  MRENCLAVE: / { print $2; exit }')
    echo "Reading MRENCLAVE from worker list: ${MRENCLAVE}"
[[ -z $MRENCLAVE ]] && { echo "MRENCLAVE is empty. cannot continue" ; exit 1; }


echo ""
echo "Create a new file to encrypt:"
INPUTFILENAME="input_file"
INPUTFILE="${INPUTFILENAME}.txt"
touch ${INPUTFILE}
echo "These are very important data" > INPUTFILE
echo "> ${INPUTFILE}"
echo ""

echo "Encrypt file"
${CLIENT} encrypt ${INPUTFILE}

CIPHERFILE="${INPUTFILENAME}.ciphertext"
KEYFILE="${INPUTFILENAME}.aes256"

echo "> ${CIPHERFILE}"
echo "> ${KEYFILE}"
echo ""

echo "Create a new NFT onchain..."
read NFTID <<< $(${CLIENT} nft create ${ALICE} ${CIPHERFILE})
echo "Received NFT id from node: ${NFTID}"
echo ""

echo "Get registered sgx keyvaults from the onchain registry"
${CLIENT} keyvault list
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
  $SHAMIRTHRESHOLD = $(((2*URLSNUM +1)/3))
fi
echo "Threshold to recover secret : ${SHAMIRTHRESHOLD}"
echo " "

echo "Create shamir shares and provision keyvaults"
${CLIENT} keyvault provision ${ALICE} ${URLSFILE} ${SHAMIRTHRESHOLD} ${NFTID} --mrenclave ${MRENCLAVE}
URLSNFTFILE="./bin/my_keyvaults/keyvault_nft_urls_${NFTID}.txt"
echo "NFT Urls "
echo "$(<URLSNFTFILE)"
echo " "

echo "Alice transfers ownership to Bob"
${CLIENT} nft transfer ${ALICE} ${BOB} ${NFTID}
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
    ${CLIENT} keyvault check ${BOB} ${NFTID} ${CURRENTURL}  --mrenclave ${MRENCLAVE}
    ${CLIENT} keyvault get ${NFTID} ${BOB}  ${CURRENTURL}  --mrenclave ${MRENCLAVE}
done

#${CLIENT} nft decrypt ${BOB} ${CIPHERFILE} ${KEYSHAREFILE}

#adversary scenario 1
