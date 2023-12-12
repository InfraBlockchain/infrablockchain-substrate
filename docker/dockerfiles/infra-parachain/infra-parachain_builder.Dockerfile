# This is the build stage for infrablockspace. Here we create the binary in a temporary image.
FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /cumulus

COPY . /cumulus

RUN cargo build --release --locked -p infra-parachain-bin

# This is the 2nd stage: a very small image where we copy the infrablockspace binary."
FROM docker.io/library/ubuntu:20.04

COPY --from=builder /cumulus/target/release/infra-parachain /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /cumulus infra-parachain && \
    mkdir -p /data /cumulus/.local/share && \
    chown -R infra-parachain:infra-parachain /data && \
    ln -s /data /cumulus/.local/share/infra-parachain && \
# check if executable works in this container
    /usr/local/bin/infra-parachain --version

USER infra-parachain

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/infra-parachain"]
