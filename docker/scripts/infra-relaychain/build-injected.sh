#!/usr/bin/env bash

PROJECT_ROOT=`git rev-parse --show-toplevel`

docker build -t infra-relaychain -f $PROJECT_ROOT/docker/dockerfiles/infra-relaychain/injected_infra-relaychain_builder.Dockerfile .