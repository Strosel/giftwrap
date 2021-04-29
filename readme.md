# Giftwrap
Wrap and unwrap your types the stylish way with derive macros for From/TryFrom in both directions

## How does it work?
`Giftwrap` exposes two derive macros, `Wrap` and `Unwrap` that derive `impl From<inner_type> for your_type` and `impl From<your_type> for inner_type` (or `TryFrom<your_type>` in the case of enums) respectively.
It works for any struct or enum variant that holds only a single type, and don't worry variants with multiple types or with types you want to convert yourself can be easily ignored with `#[noWrap]` and `#[noUnwrap]`.

## Examples
Consider the following `error.rs`
```rust
pub type Result<T> = std::result::Result<T, Error>;

macro_rules! impl_from {
    ($wrapper:expr, $inner:ty) => {
        impl From<$inner> for Error {
            fn from(e: $inner) -> Error {
                $wrapper(e)
            }
        }
    };
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    RppalGpio(rppal::gpio::Error),
    Reqwest(reqwest::Error),
    Qr(qrcodegen::DataTooLong),
    Other(String),
}

impl_from!(Error::Io, std::io::Error);
impl_from!(Error::RppalGpio, rppal::gpio::Error);
impl_from!(Error::Reqwest, reqwest::Error);
impl_from!(Error::Qr, qrcodegen::DataTooLong);
```
This might seem simple enough but adding new error types is not as easy as it could be.

However with `Giftwrap` it's as simple as it gets:
```rust
pub type Result<T> = std::result::Result<T, Error>;

use giftwrap::Wrap;
#[derive(Debug, Wrap)]
pub enum Error {
    Io(std::io::Error),
    RppalGpio(rppal::gpio::Error),
    Reqwest(reqwest::Error),
    Qr(qrcodegen::DataTooLong),
    #[noWrap]
    Other(String),
}
```
Now you could add a new error variant wrapping a type from any library and `Giftwrap` handles the rest for you

## Todo
`Giftwrap` does not yet support:
- [ ] Generics / Lifetimes
