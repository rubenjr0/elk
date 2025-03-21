default:
    cargo r -p elk samples/types.elk

test:
    # needs cargo-nextest
    cargo nextest run --no-fail-fast
