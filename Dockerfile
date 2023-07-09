FROM rust:1.70.0-slim-buster as builder
WORKDIR /usr/src/app
RUN apt-get update && apt-get install pkg-config libssl-dev -y
COPY . .
RUN cargo install --path .


FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/ /usr/local/bin/
RUN apt-get update && apt-get install libssl-dev ca-certificates -y
ENTRYPOINT ["wedding_funcs"]

