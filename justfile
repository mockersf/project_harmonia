#!/usr/bin/env just --justfile

raw-coverage $RUSTC_BOOTSTRAP="1" $LLVM_PROFILE_FILE="target/coverage/profile-%p.profraw" $RUSTFLAGS="-C instrument-coverage --cfg coverage":
  cargo test 

coverage *ARGS: raw-coverage
  grcov target/coverage \
    --binary-path target/debug/ \
    --source-dir . \
    --excl-start "mod tests" \
    --excl-line "#\[" \
    --ignore "/*" \
    --ignore "src/ui*" \
    --ignore "*/main.rs" \
    --ignore "src/core/cli.rs" \
    {{ ARGS }}
