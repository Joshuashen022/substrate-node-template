FROM rust:1.56.0 AS build_base
RUN apt-get update && apt-get install make pkg-config libssl-dev  libclang-dev \
    libjemalloc-dev git build-essential clang curl protobuf-compiler -y
RUN git config --global --add safe.directory '*'

WORKDIR /build

RUN rustup toolchain install nightly-2021-11-01
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2021-11-01

COPY . .

RUN cargo build --release


FROM rust:1.56.0

RUN apt-get update && apt-get install git inetutils-ping iproute2 -y
RUN git config --global --add safe.directory '*'

WORKDIR /root
COPY --from=build_base /build/target/release/node-template /root/node-template
CMD /root/node-template --no-mdns --dev
