# Debug image for developmend
FROM rust:1.83.0

RUN apt update
RUN apt install curl git -y

RUN apt install curl libudev-dev pkg-config openssl libssl-dev libpq-dev postgresql jq gcc clang --yes
RUN apt install python3 python3-pip --yes

# Get cs.jar
ENV CLI_VERSION=0.4.1
RUN curl -o /usr/local/cs.jar -L https://github.com/carbynestack/cli/releases/download/cli-v$CLI_VERSION/cli-$CLI_VERSION.jar

RUN apt-get update
RUN apt-get install gettext-base openjdk-17-jre openssl libssl-dev libpq-dev --yes

ENTRYPOINT ["tail", "-f", "/dev/null"]