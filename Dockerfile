FROM rust:1.27.1

RUN rustup toolchain install nightly
COPY ./Cargo.toml /tmp
COPY ./Cargo.lock /tmp
RUN mkdir tmp/src
RUN touch tmp/src/main.rs
RUN cd /tmp && cargo fetch
RUN mkdir /work

COPY Cargo.lock /work/Cargo.lock
COPY Cargo.toml /work/Cargo.toml
COPY src /work/src
RUN cd /work && cargo build --release

RUN mkdir /app && mkdir /app/data
RUN cp /work/target/release/ryazo /app
RUN rm -rf /work
