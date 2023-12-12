FROM docker.io/library/ubuntu:20.04

COPY ./target/release/infrablockspace /usr/local/bin
COPY ./target/release/infrablockspace-execute-worker /usr/local/bin
COPY ./target/release/infrablockspace-prepare-worker /usr/local/bin

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/infrablockspace"]
