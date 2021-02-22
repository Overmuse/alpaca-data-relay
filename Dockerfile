FROM rust as planner
WORKDIR alpaca-data-relay
# We only pay the installation cost once, 
# it will be cached from the second build onwards
# To ensure a reproducible build consider pinning 
# the cargo-chef version with `--version X.X.X`
RUN cargo install cargo-chef 

COPY . .

RUN cargo chef prepare  --recipe-path recipe.json

FROM rust as cacher
WORKDIR alpaca-data-relay
RUN cargo install cargo-chef
COPY --from=planner /alpaca-data-relay/recipe.json recipe.json
RUN --mount=type=ssh cargo chef cook --release --recipe-path recipe.json

FROM rust as builder
WORKDIR alpaca-data-relay
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /alpaca-data-relay/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN --mount=type=ssh cargo build --release --bin alpaca-data-relay

FROM debian:buster-slim as runtime
WORKDIR alpaca-data-relay
COPY --from=builder /alpaca-data-relay/target/release/alpaca-data-relay /usr/local/bin
ENV RUST_LOG=alpaca-data-relay=debug
ENTRYPOINT ["/usr/local/bin/alpaca-data-relay"]
