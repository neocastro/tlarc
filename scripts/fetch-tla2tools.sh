#!/usr/bin/env bash
# Fetch the pinned, SHA-256-verified tla2tools.jar used by the SANY bridge
# and the differential-testing harness (see docs/adr/0002).
#
# The version and checksum below are the contract; bump both together and
# re-verify with `sha256sum` after downloading a new release from
# https://github.com/tlaplus/tlaplus/releases
set -euo pipefail

VERSION="v1.7.4"
SHA256="936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88"
URL="https://github.com/tlaplus/tlaplus/releases/download/${VERSION}/tla2tools.jar"

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEST="${HERE}/../bridge/lib/tla2tools.jar"

mkdir -p "$(dirname "${DEST}")"

# Already present and valid? Nothing to do.
if [ -f "${DEST}" ] && echo "${SHA256}  ${DEST}" | sha256sum -c - >/dev/null 2>&1; then
    echo "tla2tools.jar ${VERSION}: already present and checksum-valid"
    exit 0
fi

echo "Downloading tla2tools.jar ${VERSION} ..."
curl -fsSL -o "${DEST}.tmp" "${URL}"

# Verify before replacing anything.
echo "${SHA256}  ${DEST}.tmp" | sha256sum -c -
mv "${DEST}.tmp" "${DEST}"

echo "tla2tools.jar ${VERSION}: downloaded and verified"
