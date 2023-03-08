#!/bin/bash

docker volume rm rust-registry-offline_backend-index || true
docker volume rm rust-registry-offline_cargo-cache || true
docker volume rm rust-registry-offline_local-uploads || true
docker volume rm rust-registry-offline_postgres-data || true
docker volume rm rust-registry-offline_target-cache || true
