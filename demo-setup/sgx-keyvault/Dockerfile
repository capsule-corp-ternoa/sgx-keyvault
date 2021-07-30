#Tis is a image for a worker
FROM scssubstratee/substratee_dev:1804-2.12-1.1.3-001
RUN apt-get install -y git && apt-get install -y curl
RUN git clone  https://github.com/capsule-corp-ternoa/sgx-keyvault.git
RUN cd sgx-keyvault && \
    git checkout develop && \
    ./ci/install_rust.sh && \
    SGX_MODE=SW make
RUN cd sgx-keyvault && \
    mkdir -p bin3 && \
    cp bin/substratee-worker bin3/
RUN cd sgx-keyvault/bin && \
    touch spid.txt key.txt && \
    ./substratee-worker init-shard && \
    ./substratee-worker shielding-key && \
    ./substratee-worker signing-key && \
    ./substratee-worker mrenclave > ~/mrenclave.b58
# RUN cd sgx-keyvault/bin2 && \
#     touch spid.txt key.txt && \
#     ./substratee-worker init-shard && \
#     ./substratee-worker shielding-key && \
#     ./substratee-worker signing-key && \
#     ./substratee-worker mrenclave > ~/mrenclave.b58
# RUN cd sgx-keyvault/bin3 && \
#      ./substratee-worker init-shard && \
#      ./substratee-worker shielding-key && \
#      ./substratee-worker signing-key && \
#      ./substratee-worker mrenclave > ~/mrenclave.b58
EXPOSE 2910 3443
CMD cd sgx-keyvault/bin && sleep 30 && RUST_BACKTRACE=full ./substratee-worker -u ws://ternoa-node -P 2910 -p 9910 run --skip-ra