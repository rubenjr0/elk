default:
    cargo r -p elk samples/types.elk

lint:
    cargo clippy -- -W clippy::nursery

test:
    # needs cargo-nextest
    cargo nextest run --no-fail-fast

llama:
    docker run --rm -v $(pwd)/models:/models \
        -p 11434:11434 \
        ghcr.io/ggml-org/llama.cpp:server-cuda13 \
        -m /models/ibm-granite_granite-4.0-h-tiny-Q5_K_L.gguf \
        --port 11434 --host 0.0.0.0
