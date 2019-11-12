# syntax=docker/dockerfile-upstream:experimental

FROM peterhuene/azure-functions-rs-build:0.11.0 AS build-image

WORKDIR /src
COPY . /src

# Run with mounted cache
RUN --mount=type=cache,target=/src/target \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    ["cargo", "run", "--release", "--", "init", "--script-root", "/home/site/wwwroot", "--sync-extensions"]

FROM mcr.microsoft.com/azure-functions/base:2.0 as runtime-image

FROM mcr.microsoft.com/dotnet/core/runtime-deps:2.2

ENV AzureWebJobsScriptRoot=/home/site/wwwroot \
    HOME=/home \
    FUNCTIONS_WORKER_RUNTIME=Rust \
    languageWorkers__workersDirectory=/home/site/wwwroot/workers

# Copy the Azure Functions host from the runtime image
COPY --from=runtime-image [ "/azure-functions-host", "/azure-functions-host" ]

# Copy the script root contents from the build image
COPY --from=build-image ["/home/site/wwwroot", "/home/site/wwwroot"]

WORKDIR /home/site/wwwroot
CMD [ "/azure-functions-host/Microsoft.Azure.WebJobs.Script.WebHost" ]
