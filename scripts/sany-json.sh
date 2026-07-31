#!/usr/bin/env bash
# Compile (if needed) and run the sany-json bridge on a TLA+ spec.
# Usage: scripts/sany-json.sh <spec.tla> [includeDir...]
# Output: the resolved semantic tree as JSON (schema tla-ast/v1) on stdout.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "${HERE}")"
JAR="${ROOT}/bridge/lib/tla2tools.jar"
CLASSES="${ROOT}/bridge/target/classes"

if [ ! -f "${JAR}" ]; then
    echo "tla2tools.jar missing — run scripts/fetch-tla2tools.sh first" >&2
    exit 1
fi

mkdir -p "${CLASSES}"
javac -cp "${JAR}" -d "${CLASSES}" "${ROOT}/bridge/src/SanyJson.java"
java -cp "${JAR}:${CLASSES}" SanyJson "$@"
