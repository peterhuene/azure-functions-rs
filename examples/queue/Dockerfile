# syntax=docker/dockerfile-upstream:experimental

FROM peterhuene/azure-functions-rs-build:0.4.0 AS builder

WORKDIR /src
COPY . /src

# Run with mounted cache
RUN --mount=type=cache,target=/src/target \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    ["cargo", "run", "--release", "--", "init", "--worker-path", "/usr/local/bin/rust_worker", "--script-root", "/home/site/wwwroot", "--sync"]

FROM microsoft/azure-functions-dotnet-core2.0:2.0

# Copy the worker from the build image
COPY --from=builder ["/usr/local/bin/rust_worker", "/usr/local/bin/rust_worker"]

# Copy configuration to the Azure Functions Host
COPY --from=builder ["/src/appsettings.json", "/azure-functions-host/appsettings.json"]
ADD ["https://raw.githubusercontent.com/peterhuene/azure-functions-rs/v0.4.0/azure-functions/worker.config.json", "/azure-functions-host/workers/rust/worker.config.json"]

# Copy the script root contents from the build image
COPY --from=builder ["/home/site/wwwroot", "/home/site/wwwroot"]
