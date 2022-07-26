#!/bin/bash

docker image tag image-scaling:latest ghcr.io/alpin11/image-scaling:latest
docker image push ghcr.io/alpin11/image-scaling:latest