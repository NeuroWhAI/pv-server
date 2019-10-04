FROM rust:latest
LABEL maintainer="NeuroWhAI"

RUN apt-get update
RUN rustup install nightly

RUN mkdir /usr/pv-server
WORKDIR /usr/pv-server

ADD src ./src
ADD Cargo.toml .
ADD Rocket.toml .
ADD key.json .
RUN cargo +nightly build --release

ENV ROCKET_PORT 8042

CMD ["target/release/pv-server"]

EXPOSE ${ROCKET_PORT}