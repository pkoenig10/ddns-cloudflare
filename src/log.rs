macro_rules! log {
    ($($args:tt)*) => ({
        println!("{} {}", jiff::Zoned::now().strftime("%F %T"), format_args!($($args)*))
    });
}

pub(crate) use log;
