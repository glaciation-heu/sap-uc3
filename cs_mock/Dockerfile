FROM rust:alpine AS builder

RUN apk add build-base curl libressl-dev

# Install dependecies first for caching.
WORKDIR /usr/src/cs-mock
ADD Cargo.* . 
#RUN mkdir ./src
RUN echo "fn main() {}" > ./dummy.rs
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release

# Install
ADD . .
RUN cargo build --release

FROM alpine:latest
# RUN apk add envsubst openjdk11

# get binaries from builder
COPY --from=builder /usr/src/cs-mock/target/release/cs-mock /usr/local/bin/cs-mock

CMD ["cs-mock"]
