#!/usr/bin/env bash

PROJECT_ROOT=`git rev-parse --show-toplevel`

docker build -t infra-parachain -f $PROJECT_ROOT/docker/dockerfiles/infra-parachain/infra-parachain_builder.Dockerfile .