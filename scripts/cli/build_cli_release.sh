#!/bin/bash
# Copyright © Aptos Foundation
# SPDX-License-Identifier: Apache-2.0

###########################################
# Build and package a release for the CLI #
###########################################

# Note: This must be run from the root of the aptos-core repository

set -e

NAME='aptos-cli'
CRATE_NAME='aptos'
CARGO_PATH="crates/$CRATE_NAME/Cargo.toml"
PLATFORM_NAME="$1"
EXPECTED_VERSION="${2:-}"
SKIP_CHECKS="${3:-false}"
COMPATIBILITY_MODE="${4:-false}"
EXPECTED_ARCH="${5:-}"

# Grab system information
ARCH=$(uname -m)
OS=$(uname -s)
VERSION=$(sed -n '/^\w*version = /p' "$CARGO_PATH" | sed 's/^.*=[ ]*"//g' | sed 's/".*$//g')

if [[ -n "$EXPECTED_ARCH" && "$ARCH" != "$EXPECTED_ARCH" ]]; then
  echo "Expected runner arch $EXPECTED_ARCH, but uname -m returned $ARCH"
  exit 4
fi

if [[ "$SKIP_CHECKS" != "true" && -n "$EXPECTED_VERSION" ]]; then
  EXPECTED_VERSION="${EXPECTED_VERSION#v}"
  EXPECTED_VERSION=$(printf '%s\n' "$EXPECTED_VERSION" | sed -nE 's/^([0-9]+\.[0-9]+\.[0-9]+).*/\1/p')

  if [[ -z "$EXPECTED_VERSION" ]]; then
    echo "Release version is malformed, must start with x.y.z or vx.y.z"
    exit 1
  fi

  if [[ "$EXPECTED_VERSION" != "$VERSION" ]]; then
    echo "Wanted to release $EXPECTED_VERSION, but Cargo.toml says the version is $VERSION"
    exit 2
  fi
elif [[ "$SKIP_CHECKS" == "true" ]]; then
  echo "WARNING: Skipping version checks!"
fi

echo "Building release $VERSION of $NAME for $OS-$PLATFORM_NAME on $ARCH"
if [[ "$COMPATIBILITY_MODE" == "true" ]]; then
  RUSTFLAGS="-C target-cpu=generic --cfg tokio_unstable -C target-feature=-sse4.2,-avx" cargo build -p "$CRATE_NAME" --profile cli
else
  cargo build -p "$CRATE_NAME" --profile cli
fi

file "target/cli/$CRATE_NAME"

cd target/cli/

# Compress the CLI
ZIP_NAME="$NAME-$VERSION-$PLATFORM_NAME-$ARCH.zip"

echo "Zipping release: $ZIP_NAME"
zip "$ZIP_NAME" "$CRATE_NAME"
zip -T "$ZIP_NAME"
mv "$ZIP_NAME" ../..
