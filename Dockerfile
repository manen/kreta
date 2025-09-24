FROM rust:1.90.0-slim-bookworm as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY kreta-rs ./kreta-rs
COPY login_test ./login_test
COPY timetable-to-ical ./timetable-to-ical
COPY timetable-to-ical-server ./timetable-to-ical-server
RUN cargo build --release -p timetable-to-ical-server

FROM debian:bookworm-slim

COPY --from=builder /app/target/release/timetable-to-ical-server /opt/timetable-to-ical-server

EXPOSE 8080
CMD ["/opt/timetable-to-ical-server"]
