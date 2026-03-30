# `adhaan`

An Islamic prayer time implementation based on the [Adhan](https://github.com/batoulapps/Adhan) library by Batoul Apps.
It is an attempt to make an ergonomic Rust port of the aforementioned library.

## Basic usage

```rust
use adhaan::*;
let new_york_city = Coordinates::new(40.7128, -74.0059);
let date= chrono::NaiveDate::from_ymd(2019, 1, 25);
let params= Parameters::new(&prominent_methods::NorthAmerica, Madhhab::Hanafi);
let prayers= PrayerTimes::calculate(date, new_york_city, params);
```

## Acknowledgement

This library is based on the port [salah](https://github.com/insha/salah) of the [Adhan](https://github.com/batoulapps/Adhan) library by Batoul Apps. All astronomical calculations are high precision equations directly from the book [Astronomical Algorithms](http://www.willbell.com/math/mc1.htm) by Jean Meeus.

## License

Adhaan is licensed under a three clause BSD License. It basically means: do whatever you want with it as long as the copyright in Salah sticks around, the conditions are not modified and the disclaimer is present. Furthermore you must not use the names of the authors to promote derivatives of the software without written consent.

The full license text can be found in the `LICENSE` file.
