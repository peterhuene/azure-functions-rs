# syntax=docker/dockerfile-upstream:experimental

FROM peterhuene/azure-functions-rs-build:0.6.0 AS build-image

WORKDIR /src
COPY . /src

# Run with mounted cache
RUN --mount=type=cache,target=/src/target \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    ["cargo", "run", "--release", "--", "init", "--script-root", "/home/site/wwwroot", "--sync"]

FROM mcr.microsoft.com/azure-functions/base:2.0 as runtime-image

FROM microsoft/dotnet:2.2-aspnetcore-runtime

ENV AzureWebJobsScriptRoot=/home/site/wwwroot \
    HOME=/home \
    FUNCTIONS_WORKER_RUNTIME=Rust \
    languageWorkers__workersDirectory=/home/site/wwwroot/workers

# Copy the Azure Functions host from the runtime image
COPY --from=runtime-image [ "/azure-functions-host", "/azure-functions-host" ]

# Copy configuration to the Azure Functions Host
COPY --from=build-image ["/src/appsettings.json", "/azure-functions-host/appsettings.json"]

# Copy the script root contents from the build image
COPY --from=build-image ["/home/site/wwwroot", "/home/site/wwwroot"]

WORKDIR /home/site/wwwroot
CMD [ "dotnet", "/azure-functions-host/Microsoft.Azure.WebJobs.Script.WebHost.dll" ]
