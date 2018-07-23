FROM peterhuene/azure-functions-rs-ci:latest AS build-env

COPY . /src

ARG EXAMPLE

RUN if [ -z "$EXAMPLE" ]; then echo "The EXAMPLE argument is required."; exit 1; fi \
    && cd /src/examples/$EXAMPLE \
    && cargo run --release -- init --worker-path /usr/local/bin/rust_worker --script-root /home/site/wwwroot

FROM microsoft/azure-functions-dotnet-core2.0:dev-nightly

COPY --from=build-env ["/usr/local/bin/rust_worker", "/usr/local/bin/rust_worker"]
COPY --from=build-env ["/home/site/wwwroot", "/home/site/wwwroot"]
COPY --from=build-env ["/src/azure-functions/worker.config.json", "/azure-functions-host/workers/rust/worker.config.json"]
