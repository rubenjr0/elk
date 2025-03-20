default:
    cargo r -p elk samples/simple_sample.elk

test:
    # needs cargo-nextest
    cargo nextest run --no-fail-fast
