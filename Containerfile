FROM rust:1.58.1-buster as builder
# Create a project
RUN user=root cargo new --bin routeros_exporter
WORKDIR ./routeros_exporter
# Copy cargo.toml
COPY ./Cargo.toml ./Cargo.toml
# Build blank project to get deps and save a layer
RUN cargo build --release
RUN rm src/*.rs

# Copy code
ADD . ./

# Remove anything that's been build
RUN rm ./target/release/deps/routeros_exporter*

# Build
RUN cargo build --release



# Actual container
FROM rust:1.58.1-slim-buster
ARG APP=/usr/src/app

ENV TZ=Etc/UTC \
    APP_USER=appuser

# Create user/group that the app will run as
RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP} \
    && mkdir -p ${APP}/config

# Add TZData and ca-certs
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /routeros_exporter/target/release/routeros_exporter ${APP}/routeros_exporter

# Copy config dir, and default config
COPY ./default.toml ${APP}/

# Chown to the app user
RUN chown -R $APP_USER:$APP_USER ${APP}

# Everything below to run as app user, in app dir
USER $APP_USER
WORKDIR ${APP}

CMD ["./routeros_exporter"]