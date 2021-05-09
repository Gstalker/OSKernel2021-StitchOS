use core::fmt::{self, Write};
use crate::sbi::console_putchar;

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! printk {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("[Kernel]  ",concat!($fmt, "\n")) $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! LOG {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[34m[Log]     ",concat!(concat!($fmt, "\n"),"\x1b[0m")) $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! WARN {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[93m[Warn]    ",concat!(concat!($fmt, "\n"),"\x1b[0m")) $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! ERROR {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[31m[ERROR]   ",concat!(concat!($fmt, "\n"),"\x1b[0m")) $(, $($arg)+)?));
    }
}
