FROM golang:1.23.4-bullseye as builder

WORKDIR /app
ADD . .
RUN go build .

FROM ghcr.io/carbynestack/spdz:642d11f_no-offline

RUN mkdir -p /app

COPY --from=builder /app/ephemeral /app/ephemeral

CMD ["/app/ephemeral"]
