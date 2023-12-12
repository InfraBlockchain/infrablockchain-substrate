# This is the build stage for infrablockspace. Here we create the binary in a temporary image.
FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /infrablockspace

COPY . /infrablockspace

RUN cargo build --release --locked

# This is the 2nd stage: a very small image where we copy the infrablockspace binary."
FROM docker.io/library/ubuntu:20.04

COPY --from=builder /infrablockspace/target/release/infrablockspace /usr/local/bin
COPY --from=builder /infrablockspace/target/release/infrablockspace-execute-worker /usr/local/bin
COPY --from=builder /infrablockspace/target/release/infrablockspace-prepare-worker /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /infrablockspace infrablockspace && \
	mkdir -p /data /infrablockspace/.local/share && \
	chown -R infrablockspace:infrablockspace /data && \
	ln -s /data /infrablockspace/.local/share/infrablockspace && \
# check if executable works in this container
	/usr/local/bin/infrablockspace --version

USER infrablockspace

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/infrablockspace"]
