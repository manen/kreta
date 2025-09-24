FROM rust:latest

COPY ./target/release/timetable-to-ical-server /opt/timetable-to-ical-server

RUN /opt/timetable-to-ical-server
