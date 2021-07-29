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

while getopts ":m:p:P:t:" opt; do
    case $opt in
        t)
            TEST=$OPTARG
            ;;
        m)
            READMRENCLAVE=$OPTARG
            ;;
        p)
            NPORT=$OPTARG
            ;;
        P)
            RPORT=$OPTARG
            ;;
    esac
done

# using default port if none given as arguments
NPORT=${NPORT:-9944}
RPORT=${RPORT:-2000}

echo "Using node-port ${NPORT}"
echo "Using worker-rpc-port ${RPORT}"

AMOUNTSHIELD=50000000000
AMOUNTTRANSFER=40000000000

ACCOUNTALICE=//Alice
ACCOUNTBOB=//Bob

CLIENT="../bin/ternoa-client -p ${NPORT} -P ${RPORT}"

echo "* Query on-chain enclave registry:"
${CLIENT} list-workers
echo ""

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

#test file
echo "* Test input file to encrypt"
INPUTFILENAME=input_file
INPUTFILE=${INPUTFILENAME}.txt
touch  $INPUTFILE
echo "These are very important data" > $INPUTFILE
text= cat ${INPUTFILE}
echo "$text"
echo ""

CIPHERFILE=${INPUTFILENAME}.ciphertext
KEYFILE=${INPUTFILENAME}.aes256

#function to register a NFT but stops before provisioning the shards to the keyvaults for Alice
aliceCreatesACapsuleButNoProvision() {
echo "Encrypt file with aes256"
${CLIENT} encrypt ${INPUTFILE}
echo "Generate shamir shards"
echo "All urls registered in the enclave registry:"
read registered_keyvaults <<< $(${CLIENT} keyvault list)
echo "found keyvaults ${registered_keyvaults}"
URLSFILE="./my_keyvaults/keyvault_pool.txt"
# Load file into array URLS
URLS=()
readarray URLS < ${URLSFILE}
for ELEMENT in ${URLS[@]}
do
echo "URL $ELEMENT"
done
URLSNUM=${#URLS[@]}

if [ $URLSNUM -eq 0 ]
then
    echo "No urls are registered. Cannot continue"; exit 1;
else
   SHAMITHRESHOLD=$(((2*${URLSNUM}+1)/3))
fi
echo "Threshold to recover secret : ${SHAMITHRESHOLD}"
echo " "

echo "Create a new NFT "
read NFTID <<< $(${CLIENT} nft create ${ACCOUNTALICE} ${CIPHERFILE})
echo "NFT id = ${NFTID}"
}

#function to create a capsule for Alice
aliceCreatesACapsule() {
echo "Alice creates a capsule"
aliceCreatesACapsuleButNoProvision
echo " "
echo "Keyvault provision"
read PROVISIONED <<< $(${CLIENT} keyvault provision ${ACCOUNTALICE} ${NFTID} "keyvault_pool.txt" ${SHAMITHRESHOLD} ${KEYFILE} --mrenclave ${MRENCLAVE})
echo "Keyvault provision = ${PROVISIONED}"

URLSNFTFILE="./my_keyvaults/keyvault_nft_urls_${NFTID}.txt"
echo "NFT Urls"
text= cat ${URLSNFTFILE}
echo "$text"
echo " "
}

#function to retrieve the key shares from keyvault, called by Bob
bobRetrievesKeyShares() {
# Load file into array URLS
URLSNFT=()
readarray URLSNFT < ${URLSNFTFILE}
text= cat ${URLSNFTFILE}
echo "$text"
echo " "
for ELEMENT in ${URLSNFT[@]}
do
CURRENTURL=${URLSNFT[i++]}
    echo "${CURRENTURL}\n"
    read CHECKED <<< $(${CLIENT} keyvault check ${ACCOUNTBOB} ${NFTID} ${CURRENTURL}  --mrenclave ${MRENCLAVE})
    ${CLIENT} keyvault get ${NFTID} ${ACCOUNTBOB}  ${CURRENTURL}  --mrenclave ${MRENCLAVE}
done
}


#Success Scenario
echo "------ Success senario -------------"
aliceCreatesACapsule
echo " "
echo "Transfer capsule to Bob "
${CLIENT} nft transfer ${ACCOUNTALICE} ${ACCOUNTBOB} ${NFTID}
echo " "
echo "Bob open capsule"
bobRetrievesKeyShares
echo " "
KEYSHAREFILE="./my-shares/shares_nft_${NFTID}.txt"
${CLIENT} decrypt ${CIPHERFILE} ${KEYSHAREFILE}
echo " "

#adversary scenario 1
echo "------ Adversary scenario 1 -------------"
aliceCreatesACapsule
echo " "
echo "Bob fails to retrieve the key shares"
bobRetrievesKeyShares
echo " "
#adversary scenario 2
echo "------ Adversary scenario 2 -------------"
aliceCreatesACapsuleButNoProvision
echo " "
echo "Bob fails to provision key shards"
read BOBPROVISIONED <<< $(${CLIENT} keyvault provision ${ACCOUNTBOB} ${NFTID} "keyvault_pool.txt" ${SHAMITHRESHOLD} ${KEYFILE} --mrenclave ${MRENCLAVE})
echo "Keyvault provision = ${BOBPROVISIONED}"
echo " "
echo "All scenarios Done!"
