#![allow(dead_code)]
#![allow(unused_macros)]

macro_rules! return_errcode {
    ( $( $tokens:tt )* ) => {
        match $( $tokens )* {
            Ok(val) => val,
            Err(e) => { return ErrCode::from(e).into(); },
        }
    }
}

macro_rules! from_error {
    ( $( $path:path ),* ) => {
        $(
            impl From<$path> for ErrCode {
                fn from(v: $path) -> Self {
                    println!("{}", v);
                    ErrCode(-1)
                }
            }
        )*
    }
}

pub fn main() {
}
