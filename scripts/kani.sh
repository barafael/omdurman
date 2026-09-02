#!/usr/bin/env sh
# Run the Kani model checker over this workspace.
#
# Kani has no native Windows support, so on Windows the proofs run inside WSL
# (Debian) against the repo mounted at /mnt/c/... . On Linux/macOS `cargo kani`
# is invoked directly. All arguments are forwarded, e.g.
#
#   ./scripts/kani.sh -p omdurman-types
#   ./scripts/kani.sh -p omdurman-rules --harness verification::distance_is_symmetric
#   ./scripts/kani.sh -Z concrete-playback --concrete-playback=print --harness <name>
#
# `-Z stubbing` and `--features kani` are enabled by default. The stubbing
# feature lets harnesses stub heavy-but-property-neutral cascades (e.g.
# `end_player_turn` in `advance_phase_is_atomic`); the `kani` feature compiles
# the engine's `debug!` tracing call sites out, whose formatting machinery
# would otherwise dominate CBMC's SAT instance. Extra args are forwarded, so
# concrete-playback and friends still work.
#
# Kani's Linux build artifacts are kept in a separate CARGO_TARGET_DIR so they
# never collide with the host's target/ directory.
set -eu

KANI_TARGET_DIR="${KANI_TARGET_DIR:-/tmp/kani-target}"
repo_root="$(cd "$(dirname "$0")/.." && pwd)"

to_wsl_path() {
    # Git Bash reports /c/foo/bar (or C:/foo/bar); WSL needs /mnt/c/foo/bar.
    # Only the drive letter is lowercased -- the rest of the path is
    # case-sensitive inside WSL.
    p=$1
    case $p in
        [A-Za-z]:*) drive=${p%%:*}; rest=${p#*:} ;;
        /[A-Za-z]/*) rest=${p#/?}; drive=${p#/}; drive=${drive%%/*} ;;
        *) printf '%s' "$p"; return ;;
    esac
    drive=$(printf '%s' "$drive" | tr 'A-Z' 'a-z')
    printf '/mnt/%s%s' "$drive" "$rest"
}

case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN* | Windows_NT)
        repo_wsl=$(to_wsl_path "$repo_root")
        exec wsl.exe -d Debian -- bash -lc \
            "cd '$repo_wsl' && CARGO_TARGET_DIR='$KANI_TARGET_DIR' cargo kani -Z stubbing --features kani $*"
        ;;
    *)
        cd "$repo_root"
        CARGO_TARGET_DIR="$KANI_TARGET_DIR" exec cargo kani -Z stubbing --features kani "$@"
        ;;
esac
