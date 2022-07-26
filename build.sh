#!/bin/bash

docker build -t image-scaling . 
docker image tag image-scaling:latest ghcr.io/alpin11/image-scaling:latest