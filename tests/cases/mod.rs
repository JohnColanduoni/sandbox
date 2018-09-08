macro_rules! check_perm_failure {
    ($x:expr) => {
        match $x {
            Ok(_) => true,
            Err(ref err) if err.kind() == ::std::io::ErrorKind::PermissionDenied => false,
            Err(err) => panic!("unexpected error {}", err),
        }
    }
}

mod fs;
mod rand;

macro_rules! define_cases {
    ($($module:ident::$name:ident),*$(,)*) => {
        pub struct TestCases {
            $(pub $name: bool,)*
        }

        impl TestCases {
            pub fn none() -> TestCases {
                TestCases {
                    $($name: false,)*
                }
            }

            pub fn run(&self) -> bool {
                let mut all_passed = true;
                $(
                    match ::std::panic::catch_unwind(|| {
                        $module::$name()
                    }) {
                        ::std::result::Result::Ok(accessible) => if accessible == self.$name {
                            eprintln!("test case {} passed", stringify!($name));
                        } else {
                            eprintln!("test case {} was expected to {}, but it did not", stringify!($name), if self.$name { "succeed" } else { "fail" });
                            all_passed = false;
                        },
                        ::std::result::Result::Err(err) => {
                            eprintln!("test case {} panicked: {:?}", stringify!($name), err);
                            all_passed = false;
                        }
                    }
                )*
                all_passed
            }
        }
    }
}

define_cases! {
    fs::create_file_home,
    fs::list_home_directory,
    fs::open_extant_file_home,
    fs::open_nonexistent_file_home,
    rand::os_rng,
}