FROM docker.io/library/ubuntu:22.04

COPY ./target/release/polkadot /usr/local/bin
COPY ./target/release/polkadot-execute-worker /usr/local/bin
COPY ./target/release/polkadot-prepare-worker /usr/local/bin

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/polkadot"]