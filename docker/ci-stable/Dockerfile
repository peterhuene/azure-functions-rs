FROM rust:slim
LABEL MAINTAINER Peter Huene <peterhuene@protonmail.com>

RUN    rustup component add rustfmt-preview \
    && rustup component add clippy \
    && apt-get update \
    && apt-get upgrade -y \
    && apt-get install -y wget unzip \
    && wget https://github.com/google/protobuf/releases/download/v3.6.1/protoc-3.6.1-linux-x86_64.zip \
    && unzip protoc-3.6.1-linux-x86_64.zip -d /usr \
    && rm protoc-3.6.1-linux-x86_64.zip \
    && apt-get install -y g++ cmake libssl-dev git \
    && apt-get remove -y --purge wget unzip \
    && apt-get autoremove -y;

ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /root

CMD ["/bin/true"]
