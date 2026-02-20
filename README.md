# `kreta`

kreta api interface rust-ban mert miert ne \
es tok sok mas is amugy

## [`timetable-to-ical-server`](./timetable-to-ical-server)

see all your lessons, exams, homework, etc in your native calendar app

self-hosted on-demand timetable to calendar service \
you can try it out locally (after cloning of course) with

```sh
cargo run -p timetable-to-ical-server
```

navigate to the url it tells you and the rest is pretty straight forward

## [`timetable-to-ical`](./timetable-to-ical)

library to convert kreta timetables info into the industry standard ical calendar format \
it has plenty of [options](./timetable-to-ical/src/lib.rs) with sensible defaults


## [`kreta-rs`](./kreta-rs)

minimal api for querying data from the e-kreta system

### features

- logging in and acquiring tokens, refreshing tokens
- timetable bulk querying and deserializing
- homework bulk query
- exams bulk query
- absence bulk query
- workaround for query time constraints using the [`timerange`](./timerange) feature

anything else: no, pull requests welcome

## [`absence-analyzer`](./absence-analyzer)

small crate that pokes around your absences to extract statistics and display them in html form. html statistics can be queried through a `timetable-to-ical-server` server

### privacy & data handling

nowhere in the `timetable-to-ical-server` stack does your password escape your request unencrypted. i don't care about your grades. \
however, to avoid freaking out the kreta idp server, your access tokens are cached, and if possible reused and refreshed rather than doing the whole login sequence again. this means that these data points are saved across requests in memory:

- your username (oktatási azonosító)
- your school's id
- a sha256 hash of your password, to ensure later requests aren't using a false password
- your access & refresh tokens

[you can review the relevant code here](./timetable-to-ical-server/src/clients.rs)

when using the default k8 credentials system, your login details are encrypted using [age](https://crates.io/crates/age), only decrypted on the server, making the `.ical` requests safe(r) over bare http. a k8 generated from one `timetable-to-ical-server` instance will not be vaild on another.

timetable requests aren't cached, so one `timetable.ical` request = one kreta timetable query, and \
one `combine.ical` request = 8 kreta api calls (currently) \
one `absences.html` request = many kreta api calls
