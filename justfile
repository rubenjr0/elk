default:
    cargo r -p elk sample.elk

test:
    # needs cargo-nextest
    cargo nextest run --no-fail-fast

cov:
    # needs cargo-nextest and cargo-llvm-cov
    cargo llvm-cov nextest --html

pgo:
    # Needs cargo-pgo
    cargo pgo test
    cargo pgo optimize test
