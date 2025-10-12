# `kreta-rs`

kreta api interface rust-ban mert miert ne

## features

- logging in and acquiring tokens, refreshing tokens
- timetable bulk querying and deserializing
- homework bulk query
- exams bulk query

anything else: no, pull requests welcome

## `timetable-to-ical`

library to convert kreta timetables info into the industry standard ical calendar format \
it has plenty of [options](./timetable-to-ical/src/lib.rs) with sensible defaults

## `timetable-to-ical-server`

self-hosted on-demand timetable to calendar service \
you can try it out locally (after cloning of course) with

```sh
cargo run -p timetable-to-ical-server
```

navigate to the url it tells you and the rest is pretty straight forward

### privacy & data handling

nowhere in the `timetable-to-ical-server` stack does your password escape your request unencrypted. i don't care about your grades. \
however, to avoid freaking out the kreta idp server, your access tokens are cached, and if possible reused and refreshed rather than doing the whole login sequence again. this means that these data points are saved across requests in memory:

- your username (oktatási azonosító)
- your school's id
- a sha256 hash of your password, to ensure later requests aren't using a false password
- your access & refresh tokens

[you can review the relevant code here](./timetable-to-ical-server/src/clients.rs)

timetable requests aren't cached, so one `timetable.ical` request = one kreta timetable query
