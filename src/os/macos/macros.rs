macro_rules! try_libc {
    (pid: $x:expr) => {
        {
            let pid = $x;
            if pid == -1 {
                return ::std::result::Result::Err(::std::io::Error::last_os_error());
            }
            pid
        }
    };
    (pid: $x:expr, $msg:tt $(,)* $($arg:expr),* $(,)*) => {
        {
            let pid = $x;
            if pid == -1 {
                let err = ::std::io::Error::last_os_error();
                error!($msg, err, $($arg)*);
                return ::std::result::Result::Err(err);
            }
            pid
        }
    };
    (fd: $x:expr) => {
        {
            let fd = $x;
            if fd == -1 {
                return ::std::result::Result::Err(::std::io::Error::last_os_error());
            }
            fd
        }
    };
    (ptr: $x:expr) => {
        {
            let p = $x;
            if p.is_null() {
                return ::std::result::Result::Err(::std::io::Error::last_os_error());
            }
            p
        }
    };
    ($x:expr) => {
        if $x != 0 {
            return ::std::result::Result::Err(::std::io::Error::last_os_error());
        }
    };
    ($x:expr, $msg:tt $(,)* $($arg:expr),* $(,)*) => {
        if $x != 0 {
            let err = ::std::io::Error::last_os_error();
            error!($msg, err, $($arg)*);
            return ::std::result::Result::Err(err);
        }
    };
}