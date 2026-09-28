#!/usr/bin/env bash
# Clone omdurman and run its Kani proof suite (Linux/macOS; needs git + cargo).
#
#   ./run-kani.sh                  # the whole suite
#   ./run-kani.sh --harness NAME   # one harness; any cargo-kani args pass through
#
# Writes only: kani-verifier into ~/.cargo/bin, Kani's bundle into ~/.kani,
# and $WORKDIR (the clone, its build artifacts and the log).
#
# Env: REPO_URL, BRANCH (main), WORKDIR (./omdurman-kani), KANI_VERSION (0.67.0),
#      KANI_JOBS (default: one harness per 32 GB of free RAM, at most one per CPU).
set -euo pipefail

REPO_URL=${REPO_URL:-https://github.com/barafael/omdurman.git}
BRANCH=${BRANCH:-main}
WORKDIR=${WORKDIR:-$PWD/omdurman-kani}
KANI_VERSION=${KANI_VERSION:-0.67.0}

# 1. Kani: prebuilt through cargo-binstall if that is already installed, else
#    built with cargo install. `cargo kani setup` fetches CBMC and Kani's nightly.
if ! cargo kani --version 2>/dev/null | grep -qxF "cargo-kani $KANI_VERSION"; then
  if cargo binstall -V >/dev/null 2>&1; then
    cargo binstall -y --locked "kani-verifier@$KANI_VERSION"
  else
    cargo install --locked "kani-verifier@$KANI_VERSION"
  fi
fi
[ -d "$HOME/.kani/kani-$KANI_VERSION" ] || cargo kani setup

# 2. Source: clone, or move an existing clone to the branch tip.
if [ -d "$WORKDIR/.git" ]; then
  git -C "$WORKDIR" fetch --quiet "$REPO_URL" "$BRANCH"
  git -C "$WORKDIR" checkout --quiet --detach FETCH_HEAD
else
  git clone --quiet --branch "$BRANCH" "$REPO_URL" "$WORKDIR"
fi

# 3. Parallelism: the heaviest harnesses peak at 13-14 GB.
if [ -z "${KANI_JOBS:-}" ]; then
  mem_gb=$(awk '/MemAvailable/ {print int($2 / 1048576)}' /proc/meminfo 2>/dev/null || echo 0)
  cpus=$(getconf _NPROCESSORS_ONLN)
  jobs=$((mem_gb / 32))
  KANI_JOBS=$((jobs < 1 ? 1 : jobs > cpus ? cpus : jobs))
fi

# 4. Run; build artifacts stay in the clone (scripts/kani.sh defaults to /tmp).
cd "$WORKDIR"
export KANI_JOBS KANI_TARGET_DIR="$WORKDIR/target/kani"
log="$WORKDIR/kani-$(date +%Y%m%d-%H%M%S).log"
echo "== $(git rev-parse --short HEAD), KANI_JOBS=$KANI_JOBS, log: $log"
status=0
./scripts/kani.sh -p omdurman-types -p omdurman-rules "$@" 2>&1 | tee "$log" || status=$?

echo "== summary"
grep -E "Verification failed for|out of memory|Complete -" "$log" || true
exit "$status"
