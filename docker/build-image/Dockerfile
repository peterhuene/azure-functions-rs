FROM rust:slim
LABEL MAINTAINER Peter Huene <peterhuene@protonmail.com>

ENV PATH "$PATH:/root/.cargo/bin"
ENV DOTNET_SKIP_FIRST_TIME_EXPERIENCE true
ENV DOTNET_CLI_TELEMETRY_OPTOUT true

RUN apt-get update \
    && apt-get upgrade -y \
    && apt-get install -y wget unzip apt-transport-https gnupg \
    && wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > microsoft.asc.gpg \
    && mv microsoft.asc.gpg /etc/apt/trusted.gpg.d/ \
    && wget -q https://packages.microsoft.com/config/debian/9/prod.list \
    && mv prod.list /etc/apt/sources.list.d/microsoft-prod.list \
    && apt-get update \
    && wget https://github.com/google/protobuf/releases/download/v3.6.1/protoc-3.6.1-linux-x86_64.zip \
    && unzip protoc-3.6.1-linux-x86_64.zip -d /usr \
    && rm protoc-3.6.1-linux-x86_64.zip \
    && apt-get install -y dotnet-sdk-2.2 \
    && apt-get remove -y --purge wget unzip apt-transport-https gnupg \
    && apt-get autoremove -y \
    && apt-get clean \
    && rm -rf /usr/share/dotnet/sdk/NuGetFallbackFolder/*

WORKDIR /root

CMD ["/bin/true"]
