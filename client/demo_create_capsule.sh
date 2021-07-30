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


aliceCreatesACapsule(){
echo ""
echo "Create a new file to encrypt:"
INPUTFILENAME="input_file"
INPUTFILE="${INPUTFILENAME}.txt"
touch ${INPUTFILE}
echo "These are very important data" > INPUTFILE
INPUTFILETEXT=cat ${INPUTFILE}
echo "> ${INPUTFILE}: ${INPUTFILETEXT}"
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
}

echo "Get registered sgx keyvaults from the onchain registry"
read registered_keyvaults <<< $(${CLIENT} keyvault list)
echo "found keyvaults ${registered_keyvaults}"
URLSFILE="./my_keyvaults/keyvault_pool.txt"


aliceProvisionsKeyvaults(){
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


echo "Create shamir shares and provision keyvaults"
read PROVISIONED <<< $(${CLIENT} keyvault provision ${ALICE} ${NFTID} "keyvault_pool.txt" ${SHAMITHRESHOLD} ${KEYFILE} --mrenclave ${MRENCLAVE})
echo "Provisioned keyvaults = ${PROVISIONED}"

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
aliceProvisionsKeyvaults
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
aliceProvisionsKeyvaults
echo " "
echo "Bob fails to retrieve the key shares"
bobRetrievesKeyShares
echo " "
#adversary scenario 2
echo "------ Adversary scenario 2 -------------"
aliceCreatesACapsule
echo " "
echo "Bob fails to provision key shards"
read BOBPROVISIONED <<< $(${CLIENT} keyvault provision ${ACCOUNTBOB} ${NFTID} "keyvault_pool.txt" ${SHAMITHRESHOLD} ${KEYFILE} --mrenclave ${MRENCLAVE})
echo "Keyvault provision = ${BOBPROVISIONED}"
echo " "
echo "All scenarios Done!"
