# This is the build stage for infra-relaychain. Here we create the binary in a temporary image.
FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /infra-relaychain

COPY . /infra-relaychain

RUN cargo build --release --locked

# This is the 2nd stage: a very small image where we copy the infra-relaychain binary."
FROM docker.io/library/ubuntu:22.04

COPY --from=builder /infra-relaychain/target/release/infra-relaychain /usr/local/bin
COPY --from=builder /infra-relaychain/target/release/infra-relaychain-execute-worker /usr/local/bin
COPY --from=builder /infra-relaychain/target/release/infra-relaychain-prepare-worker /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /infra-relaychain infra-relaychain && \
	mkdir -p /data /infra-relaychain/.local/share && \
	chown -R infra-relaychain:infra-relaychain /data && \
	ln -s /data /infra-relaychain/.local/share/infra-relaychain && \
# check if executable works in this container
	/usr/local/bin/infra-relaychain --version

USER infra-relaychain

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/infra-relaychain"]
