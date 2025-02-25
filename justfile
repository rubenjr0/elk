default:
    cargo r -p elk sample.elk

test:
    # needs cargo-nextest
    cargo nextest run --no-fail-fast
