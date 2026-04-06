# Stage 1 - build
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2 - run
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/verses-api /app/
RUN mkdir /app/csv
COPY csv /app/csv
CMD ["/app/verses-api"]
