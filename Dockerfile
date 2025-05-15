# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.84.0
ARG APP_NAME=rust
ARG RAILWAY_SERVICE_ID

################################################################################
# Create a stage for building the application.
FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
ARG RAILWAY_SERVICE_ID
WORKDIR /app


RUN echo $RAILWAY_SERVICE_ID

# Install host build dependencies.
RUN apk add --no-cache clang lld musl-dev git

# Copy source files into the container.
COPY src /app/src
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock

# Build the application.
# Use cache mount IDs in the format s/<service-id>-<target-path> as per Railway's requirements.
RUN --mount=type=cache,id=s/$RAILWAY_SERVICE_ID-/app/target,sharing=locked,target=/app/target \
    --mount=type=cache,id=s/$RAILWAY_SERVICE_ID-/usr/local/cargo/git,sharing=locked,target=/usr/local/cargo/git \
    --mount=type=cache,id=s/$RAILWAY_SERVICE_ID-/usr/local/cargo/registry,sharing=locked,target=/usr/local/cargo/registry \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server

################################################################################
# Create a new stage for running the application.
FROM alpine:3.18 AS final

# Create a non-privileged user for the application.
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the executable from the build stage
COPY --from=build /bin/server /bin/

# Command to run the application.
CMD ["/bin/server"]