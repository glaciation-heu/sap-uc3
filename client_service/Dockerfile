FROM rust:1.83.0 AS builder

RUN apt update
RUN apt install curl libudev-dev pkg-config openssl libssl-dev libpq-dev postgresql jq gcc clang --yes

# Get cs.jar
ENV CLI_VERSION=0.4.1
RUN curl -o /usr/local/cs.jar -L https://github.com/carbynestack/cli/releases/download/cli-v$CLI_VERSION/cli-$CLI_VERSION.jar

# Install dependecies first for caching.
WORKDIR /usr/src/client_service
ADD Cargo.* . 
RUN mkdir ./src
RUN echo "fn main() {}" > ./dummy.rs
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release

# Install
ADD . .
# RUN cargo update -p openssl-sys
RUN cargo build --release

FROM debian:stable-slim
RUN apt-get update
RUN apt-get install gettext-base openjdk-17-jre openssl libssl-dev libpq-dev --yes

# get binaries from builder
COPY --from=builder /usr/local/cs.jar /usr/local/cs.jar
COPY --from=builder /usr/src/client_service/target/release/client_service /usr/local/bin/client_service

COPY ./startup.sh /usr/local/bin/startup.sh
RUN chmod +x /usr/local/bin/startup.sh

ENV CS_JAR_LOCATION="/usr/local/cs.jar"

CMD ["/usr/local/bin/startup.sh"]
