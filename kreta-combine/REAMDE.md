# `kreta-combine`

a library built on top of [`kreta-rs`](../kreta-rs) that combines data from:

- timetable
- bulk homework
- bulk announced exams

to generate rich timetable data filled out with everything there's to know about a given lesson, which is the assigned homework (displayed on the date of the deadline) and announced exam, with information to go along with both

## warning

this crate is made for a pretty specific usecase, as part of [`timetable-to-ical`](../timetable-to-ical/)'s rich timetable generation. for a more generic usecase, you're probably better off using [`kreta-rs`](../kreta-rs) as-is.
