FROM rust:1.56.0 AS build_base
RUN apt-get update && apt-get install make pkg-config libssl-dev  libclang-dev \
    libjemalloc-dev git build-essential clang curl protobuf-compiler -y
RUN git config --global --add safe.directory '*'

WORKDIR /build

RUN rustup toolchain install nightly-2021-11-01
RUN #rustup update nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2021-11-01

ENV CARGO_HOME=/build/.cargo
COPY .cargo /build/.cargo

#RUN cargo install cargo-chef
COPY . .
#
#RUN cargo chef prepare --recipe-path recipe.json
#COPY recipe.json .
# sudo apt install build-essential
#RUN cargo chef cook --release --recipe-path recipe.json
#RUN rustup default 1.56.0
#RUN apt-get install libclang-dev -y
#RUN apt-get install build-essential clang curl protobuf-compiler --assume-yes
RUN cargo build --release -j 3


FROM rust:1.56.0

RUN apt-get update && apt-get install git -y
RUN git config --global --add safe.directory '*'

WORKDIR /root
COPY --from=build_base /build/target/release/node-template /root/node-template
CMD /root/node-template --no-mdns --dev
