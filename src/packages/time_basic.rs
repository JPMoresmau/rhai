#![cfg(not(feature = "no_std"))]

use super::{arithmetic::make_err as make_arithmetic_err, math_basic::MAX_INT};

use crate::def_package;
use crate::dynamic::Dynamic;
use crate::plugin::*;
use crate::result::EvalAltResult;
use crate::INT;

#[cfg(not(feature = "no_float"))]
use crate::FLOAT;

use crate::stdlib::boxed::Box;

#[cfg(not(target_arch = "wasm32"))]
use crate::stdlib::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
use instant::{Duration, Instant};

def_package!(crate:BasicTimePackage:"Basic timing utilities.", lib, {
    // Register date/time functions
    combine_with_exported_module!(lib, "time", time_functions);
});

#[export_module]
mod time_functions {
    pub fn timestamp() -> Instant {
        Instant::now()
    }

    #[rhai_fn(name = "elapsed", get = "elapsed", return_raw)]
    pub fn elapsed(timestamp: &mut Instant) -> Result<Dynamic, Box<EvalAltResult>> {
        #[cfg(not(feature = "no_float"))]
        {
            if *timestamp > Instant::now() {
                Err(make_arithmetic_err("Time-stamp is later than now"))
            } else {
                Ok((timestamp.elapsed().as_secs_f64() as FLOAT).into())
            }
        }

        #[cfg(feature = "no_float")]
        {
            let seconds = timestamp.elapsed().as_secs();

            if cfg!(not(feature = "unchecked")) && seconds > (MAX_INT as u64) {
                Err(make_arithmetic_err(format!(
                    "Integer overflow for timestamp.elapsed: {}",
                    seconds
                )))
            } else if *timestamp > Instant::now() {
                Err(make_arithmetic_err("Time-stamp is later than now"))
            } else {
                Ok((seconds as INT).into())
            }
        }
    }

    #[rhai_fn(return_raw, name = "-")]
    pub fn time_diff(ts1: Instant, ts2: Instant) -> Result<Dynamic, Box<EvalAltResult>> {
        #[cfg(not(feature = "no_float"))]
        {
            Ok(if ts2 > ts1 {
                -(ts2 - ts1).as_secs_f64() as FLOAT
            } else {
                (ts1 - ts2).as_secs_f64() as FLOAT
            }
            .into())
        }

        #[cfg(feature = "no_float")]
        if ts2 > ts1 {
            let seconds = (ts2 - ts1).as_secs();

            if cfg!(not(feature = "unchecked")) && seconds > (MAX_INT as u64) {
                Err(make_arithmetic_err(format!(
                    "Integer overflow for timestamp duration: -{}",
                    seconds
                )))
            } else {
                Ok((-(seconds as INT)).into())
            }
        } else {
            let seconds = (ts1 - ts2).as_secs();

            if cfg!(not(feature = "unchecked")) && seconds > (MAX_INT as u64) {
                Err(make_arithmetic_err(format!(
                    "Integer overflow for timestamp duration: {}",
                    seconds
                )))
            } else {
                Ok((seconds as INT).into())
            }
        }
    }

    #[cfg(not(feature = "no_float"))]
    pub mod float_functions {
        fn add_impl(x: Instant, seconds: FLOAT) -> Result<Instant, Box<EvalAltResult>> {
            if seconds < 0.0 {
                subtract_impl(x, -seconds)
            } else if cfg!(not(feature = "unchecked")) {
                if seconds > (MAX_INT as FLOAT) {
                    Err(make_arithmetic_err(format!(
                        "Integer overflow for timestamp add: {}",
                        seconds
                    )))
                } else {
                    x.checked_add(Duration::from_millis((seconds * 1000.0) as u64))
                        .ok_or_else(|| {
                            make_arithmetic_err(format!(
                                "Timestamp overflow when adding {} second(s)",
                                seconds
                            ))
                        })
                }
            } else {
                Ok(x + Duration::from_millis((seconds * 1000.0) as u64))
            }
        }
        fn subtract_impl(x: Instant, seconds: FLOAT) -> Result<Instant, Box<EvalAltResult>> {
            if seconds < 0.0 {
                add_impl(x, -seconds)
            } else if cfg!(not(feature = "unchecked")) {
                if seconds > (MAX_INT as FLOAT) {
                    Err(make_arithmetic_err(format!(
                        "Integer overflow for timestamp add: {}",
                        seconds
                    )))
                } else {
                    x.checked_sub(Duration::from_millis((seconds * 1000.0) as u64))
                        .ok_or_else(|| {
                            make_arithmetic_err(format!(
                                "Timestamp overflow when adding {} second(s)",
                                seconds
                            ))
                        })
                }
            } else {
                Ok(x - Duration::from_millis((seconds * 1000.0) as u64))
            }
        }

        #[rhai_fn(return_raw, name = "+")]
        pub fn add(x: Instant, seconds: FLOAT) -> Result<Dynamic, Box<EvalAltResult>> {
            add_impl(x, seconds).map(Into::<Dynamic>::into)
        }
        #[rhai_fn(return_raw, name = "+=")]
        pub fn add_assign(x: &mut Instant, seconds: FLOAT) -> Result<Dynamic, Box<EvalAltResult>> {
            *x = add_impl(*x, seconds)?;
            Ok(().into())
        }
        #[rhai_fn(return_raw, name = "-")]
        pub fn subtract(x: Instant, seconds: FLOAT) -> Result<Dynamic, Box<EvalAltResult>> {
            subtract_impl(x, seconds).map(Into::<Dynamic>::into)
        }
        #[rhai_fn(return_raw, name = "-=")]
        pub fn subtract_assign(
            x: &mut Instant,
            seconds: FLOAT,
        ) -> Result<Dynamic, Box<EvalAltResult>> {
            *x = subtract_impl(*x, seconds)?;
            Ok(().into())
        }
    }

    fn add_impl(x: Instant, seconds: INT) -> Result<Instant, Box<EvalAltResult>> {
        if seconds < 0 {
            subtract_impl(x, -seconds)
        } else if cfg!(not(feature = "unchecked")) {
            x.checked_add(Duration::from_secs(seconds as u64))
                .ok_or_else(|| {
                    make_arithmetic_err(format!(
                        "Timestamp overflow when adding {} second(s)",
                        seconds
                    ))
                })
        } else {
            Ok(x + Duration::from_secs(seconds as u64))
        }
    }
    fn subtract_impl(x: Instant, seconds: INT) -> Result<Instant, Box<EvalAltResult>> {
        if seconds < 0 {
            add_impl(x, -seconds)
        } else if cfg!(not(feature = "unchecked")) {
            x.checked_sub(Duration::from_secs(seconds as u64))
                .ok_or_else(|| {
                    make_arithmetic_err(format!(
                        "Timestamp overflow when adding {} second(s)",
                        seconds
                    ))
                })
        } else {
            Ok(x - Duration::from_secs(seconds as u64))
        }
    }

    #[rhai_fn(return_raw, name = "+")]
    pub fn add(x: Instant, seconds: INT) -> Result<Dynamic, Box<EvalAltResult>> {
        add_impl(x, seconds).map(Into::<Dynamic>::into)
    }
    #[rhai_fn(return_raw, name = "+=")]
    pub fn add_assign(x: &mut Instant, seconds: INT) -> Result<Dynamic, Box<EvalAltResult>> {
        *x = add_impl(*x, seconds)?;
        Ok(().into())
    }
    #[rhai_fn(return_raw, name = "-")]
    pub fn subtract(x: Instant, seconds: INT) -> Result<Dynamic, Box<EvalAltResult>> {
        subtract_impl(x, seconds).map(Into::<Dynamic>::into)
    }
    #[rhai_fn(return_raw, name = "-=")]
    pub fn subtract_assign(x: &mut Instant, seconds: INT) -> Result<Dynamic, Box<EvalAltResult>> {
        *x = subtract_impl(*x, seconds)?;
        Ok(().into())
    }

    #[rhai_fn(name = "==")]
    pub fn eq(x: Instant, y: Instant) -> bool {
        x == y
    }
    #[rhai_fn(name = "!=")]
    pub fn ne(x: Instant, y: Instant) -> bool {
        x != y
    }
    #[rhai_fn(name = "<")]
    pub fn lt(x: Instant, y: Instant) -> bool {
        x < y
    }
    #[rhai_fn(name = "<=")]
    pub fn lte(x: Instant, y: Instant) -> bool {
        x <= y
    }
    #[rhai_fn(name = ">")]
    pub fn gt(x: Instant, y: Instant) -> bool {
        x > y
    }
    #[rhai_fn(name = ">=")]
    pub fn gte(x: Instant, y: Instant) -> bool {
        x >= y
    }
}
