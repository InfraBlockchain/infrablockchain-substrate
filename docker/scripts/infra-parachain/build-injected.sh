#!/usr/bin/env bash

PROJECT_ROOT=`git rev-parse --show-toplevel`

docker build -t infra-parachain -f $PROJECT_ROOT/docker/dockerfiles/infra-parachain/injected_infra-parachain.Dockerfile .