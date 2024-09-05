default:
    cargo r -p elk sample.elk

pgo:
    # Needs cargo-pgo
    cargo pgo test
    cargo pgo optimize test
