# This is the build stage for infrablockchain. Here we create the binary in a temporary image.
FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /infrablockchain

COPY . /infrablockchain

RUN cargo build --release --locked

# This is the 2nd stage: a very small image where we copy the infrablockchain binary."
FROM docker.io/library/ubuntu:20.04

COPY --from=builder /infrablockchain/target/release/infra-relaychain /usr/local/bin
COPY --from=builder /infrablockchain/target/release/infra-relaychain-execute-worker /usr/local/bin
COPY --from=builder /infrablockchain/target/release/infra-relaychain-prepare-worker /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /infrablockchain infrablockchain && \
	mkdir -p /data /infrablockchain/.local/share && \
	chown -R infrablockchain:infrablockchain /data && \
	ln -s /data /infrablockchain/.local/share/infrablockchain && \
# check if executable works in this container
	/usr/local/bin/infrablockchain --version

USER infrablockchain

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/infrablockchain"]
