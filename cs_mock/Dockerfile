FROM rust:1.83.0-bullseye AS builder

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

FROM python:3.9.16-bullseye
WORKDIR /root
RUN apt-get update && apt-get install -y \
  curl \
  openssl \
  xz-utils

ADD scripts .

# ADD api-srv api-srv
RUN ["bash", "setup.bash"]

EXPOSE 8011

# ADD source/* ./MP-SPDZ/Programs/Source/

# RUN ["bash", "compile.bash"]

COPY --from=builder /usr/src/cs-mock/target/release/cs-mock /bin/cs-mock

# CMD ["npm", "--prefix", "api-srv", "run", "start"]

CMD ["cs-mock"]
