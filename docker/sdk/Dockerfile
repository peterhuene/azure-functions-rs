FROM rustlang/rust:nightly-slim AS builder

ENV DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1

RUN    apt-get update \
    && apt-get install -y wget gnupg apt-transport-https \
    && wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > microsoft.asc.gpg \
    && mv microsoft.asc.gpg /etc/apt/trusted.gpg.d/ \
    && wget -q https://packages.microsoft.com/config/debian/9/prod.list \
    && mv prod.list /etc/apt/sources.list.d/microsoft-prod.list \
    && apt-get update \
    && apt-get install -y cmake g++ golang dotnet-sdk-2.1 \
    && apt-get remove -y --purge wget \
    && apt-get autoremove -y

ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /root

CMD ["/bin/true"]
