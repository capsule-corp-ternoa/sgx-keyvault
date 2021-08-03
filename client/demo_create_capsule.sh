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

echo "------ STARTING DEMO -------------"
echo "Using node-port ${NPORT}"
echo " Waiting 10s to ensure all workers have started.. "
sleep 10



CLIENT="../bin/ternoa-client -p ${NPORT}"
ALICE="//Alice"
BOB="//BOB"

read MRENCLAVE <<< $($CLIENT list-workers | awk '/  MRENCLAVE: / { print $2; exit }')
    echo "Reading MRENCLAVE from worker list: ${MRENCLAVE}"
[[ -z $MRENCLAVE ]] && { echo "MRENCLAVE is empty. cannot continue" ; exit 1; }



INPUTFILENAME="input_file"
INPUTFILE="${INPUTFILENAME}.txt"
DECRYPTEDFILE="${INPUTFILENAME}.decrypted"

CIPHERFILE="${INPUTFILENAME}.ciphertext"
KEYFILE="${INPUTFILENAME}.aes256"


COPIED_FILENAME="input_file_copy"
CIPHERFILE_COPIED="${COPIED_FILENAME}.ciphertext"



aliceCreatesACapsule(){
echo ""
echo "Alice creates a new file to encrypt:"
touch ${INPUTFILE}
echo "These are very important data" > $INPUTFILE
read INPUTFILE_TEXT <<< $(cat ${INPUTFILE})
echo "> ${INPUTFILE}: ${INPUTFILE_TEXT}"
echo ""
echo "Alice encrypts file ${INPUTFILE}"
${CLIENT} encrypt ${INPUTFILE}
read CIPHERFILE_TEXT <<< $(cat ${CIPHERFILE})
echo "> ${CIPHERFILE} : ${CIPHERFILE_TEXT}"
echo "> ${KEYFILE}"
echo ""

echo "Alices creates a new NFT onchain..."
read NFTID <<< $(${CLIENT} nft create ${ALICE} ${CIPHERFILE})
echo "Received NFT id from node: ${NFTID}"
echo ""
}

aliceMutatesACapsule(){
getKeyvaults
URLS=()
readarray URLS < ${URLSFILE}
echo "Check if Keyvault with url ${URLS[0]} registered new nft id.. "
echo " "
read NFTIDS <<< $(${CLIENT} keyvault get-nft-registry ${URLS[0]} --mrenclave ${MRENCLAVE})
echo "Received the following NFT:"
echo "${NFTIDS}"
echo " "
echo "Alices copies the ciphertext into another file..."
cp $CIPHERFILE $CIPHERFILE_COPIED
read CIPHERFILE_COPY_TEXT <<< $(cat ${CIPHERFILE_COPIED})
echo "> ${CIPHERFILE_COPIED} : ${CIPHERFILE_COPY_TEXT}"
echo " "
echo "Alices mutates her NFT onchain..."
${CLIENT} nft mutate ${ALICE} ${NFTID} ${CIPHERFILE_COPIED}
echo "Wait 30s until keyvaults registered new blocks .. "
echo " "
sleep 30
echo "Check if Keyvault with url ${URLS[0]} mutated nft .."
echo " "
read NFTIDS <<< $(${CLIENT} keyvault get-nft-registry ${URLS[0]} --mrenclave ${MRENCLAVE})
echo "Received the following NFT:"
echo "${NFTIDS}"
echo " "
}

getKeyvaults(){
echo "Get registered sgx keyvaults from the onchain registry"
echo " "
${CLIENT} keyvault list
URLSFILE="./my_keyvaults/keyvault_pool.txt"
echo " "
}

aliceProvisionsKeyvaults(){
# Load file into array URLS
URLS=()
readarray URLS < ${URLSFILE}
echo "Alices provisiones the following keyvaults:"
for ELEMENT in ${URLS[@]}
do
echo "$ELEMENT"
done
URLSNUM=${#URLS[@]}

if [ $URLSNUM -eq 0 ]
then
    echo "No urls are registered. Cannot continue"; exit 1;
else
   SHAMITHRESHOLD=$(((2*${URLSNUM}+1)/3))
fi
echo ""
echo "Setting threshold to recover secret : ${SHAMITHRESHOLD}"
echo " "


echo "Create shamir shares and provision keyvaults"
${CLIENT} keyvault provision ${ALICE} ${NFTID} "keyvault_pool.txt" ${SHAMITHRESHOLD} ${KEYFILE} --mrenclave ${MRENCLAVE}
echo " "

URLSNFTFILE="./my_keyvaults/keyvault_nft_urls_${NFTID}.txt"
echo "Successfully provisioned the following keyvaults:"
text= cat ${URLSNFTFILE}
echo "$text"
echo " "
}


#function to retrieve the key shares from keyvault, called by Bob
bobRetrievesKeyShares() {
# Load file into array URLS
i=0
URLSNFT=()
readarray URLSNFT < ${URLSNFTFILE}
text= cat ${URLSNFTFILE}
echo "$text"
echo " "
for ELEMENT in ${URLSNFT[@]}
do
CURRENTURL=${URLSNFT[i++]}
    read CHECKED <<< $(${CLIENT} keyvault check ${BOB} ${NFTID} ${CURRENTURL} --mrenclave ${MRENCLAVE})
    read GOTTEN <<< $(${CLIENT} keyvault get ${BOB} ${NFTID} ${CURRENTURL} --mrenclave ${MRENCLAVE})
    echo "${CURRENTURL} returned for check: ${CHECKED}"
done
}


#Success Scenario
echo "------ Success senario -------------"
echo " "
aliceCreatesACapsule
echo "Wait 30s until keyvaults registered new blocks .. "
echo " "
sleep 30
aliceMutatesACapsule
aliceProvisionsKeyvaults
echo " "
echo "Transfer capsule to Bob "
${CLIENT} nft transfer ${ALICE} ${BOB} ${NFTID}
echo " "
echo "Wait 30s until keyvaults registered new blocks .. "
echo " "
sleep 30
echo "Bob retrieves his new shamir key shares"
bobRetrievesKeyShares
echo " "
KEYSHAREFILE="./my_shares/shares_nft_${NFTID}.txt"
${CLIENT} decrypt ${CIPHERFILE} ${KEYSHAREFILE}
read DECRYPTED_TEXT <<< $(cat ${DECRYPTEDFILE})
echo " Bob successfully decrypted"
echo "> ${CIPHERFILE} : ${CIPHERFILE_TEXT} to"
echo "> ${DECRYPTEDFILE} : ${DECRYPTED_TEXT}"



# adversary scenario 1
echo " "
echo " "
echo " "
echo "------ Adversary scenario 1 -------------"
echo " "
aliceCreatesACapsule
echo "Wait 30s until keyvaults registered new blocks .. "
echo " "
sleep 30
getKeyvaults
URLS=()
readarray URLS < ${URLSFILE}
aliceProvisionsKeyvaults
echo " "
echo "Bob should fail to retrieve the key shares"
bobRetrievesKeyShares
echo " "


#adversary scenario 2
echo " "
echo " "
echo " "
echo " "
echo "------ Adversary scenario 2 -------------"
echo " "
aliceCreatesACapsule
echo "Wait 30s until keyvaults registered new blocks .. "
echo " "
sleep 30
echo "Bob should fail to provision key shards:"
read BOBPROVISIONED <<< $(${CLIENT} keyvault provision ${BOB} ${NFTID} "keyvault_pool.txt" ${SHAMITHRESHOLD} ${KEYFILE} --mrenclave ${MRENCLAVE})
echo "${BOBPROVISIONED}"
echo " "
echo "All scenarios Done!"
