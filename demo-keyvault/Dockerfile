FROM scssubstratee/substratee_dev:1804-2.12-1.1.3-001
# prepare the linux system
RUN apt-get update && \
    apt-get install -y git && apt-get install -y curl && \
    apt-get install -y build-essential && \
    apt-get install -y clang && \
    apt-get install -y libssl-dev && \
    apt-get install -y llvm && \
    apt-get install -y libudev-dev
#create base dir
RUN mkdir /ternoa
#install rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default stable && \
	rustup update && \
	rustup toolchain install nightly-2021-05-11 && \
    rustup target add wasm32-unknown-unknown --toolchain nightly-2021-05-11
RUN cargo --help
RUN rustup show
#prepare chain
RUN cd /ternoa && \
    git clone https://github.com/capsule-corp-ternoa/chain.git
RUN cd /ternoa/chain && \
    git checkout add-skip-ra-feature && \
    ls && \
	cargo build --release --features skip-ias-check
# prepare workers
RUN cd /ternoa && \
    git clone  https://github.com/capsule-corp-ternoa/sgx-keyvault.git
RUN rustup show
RUN cd /ternoa/sgx-keyvault && \
    git checkout develop && \
    SGX_MODE=SW make
# copy the shell scripts to the docker to launch to node and the workers
#copy scenarios scrip too
COPY scripts/* /ternoa/
WORKDIR /ternoa
