FROM ubuntu
RUN apt-get update
RUN apt-get install -y git && apt-get install -y curl
RUN git clone https://github.com/capsule-corp-ternoa/chain.git
RUN cd chain && \
    git checkout add-skip-ra-feature && \
    ls && \
    apt-get install -y build-essential && \
    apt-get install -y clang && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    export PATH="$PATH:$HOME/.cargo/bin" && \
	rustup toolchain install nightly-2021-05-11 && \
	rustup target add wasm32-unknown-unknown --toolchain nightly-2021-05-11 && \
	rustup default stable && \
	cargo build --release --features skip-ias-check
EXPOSE 9910
CMD ./chain/target/release/ternoa --dev --tmp --ws-port 9910