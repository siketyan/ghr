# syntax = docker/dockerfile:1.4

FROM rust:1.69 AS base
SHELL ["/bin/bash", "-c"]
WORKDIR /src

RUN cargo install cargo-chef

# ---

FROM base as planner
COPY . .
RUN cargo chef prepare --recipe-path /recipe.json

# ---

FROM base as builder
SHELL ["/bin/bash", "-c"]

COPY --from=planner /recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json --bin ghr

COPY . .
RUN cargo build --release --bin ghr

# ---

FROM gcr.io/distroless/cc
COPY --from=builder /src/target/release/ghr /bin/ghr
ENTRYPOINT ["/bin/ghr"]

