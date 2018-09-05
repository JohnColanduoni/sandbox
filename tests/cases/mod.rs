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

pub struct TestCases {
    pub create_file_home: bool,
    pub list_home_directory: bool,
    pub open_extant_file_home: bool,
    pub open_nonexistent_file_home: bool,
    pub os_rng: bool,
}

macro_rules! run_case {
    ($expected:expr => $f:expr) => {
        match ::std::panic::catch_unwind(|| {
            $f
        }) {
            ::std::result::Result::Ok(accessible) => if accessible == $expected {
                eprintln!("test case {} passed", stringify!($expected));
                true
            } else {
                eprintln!("test case {} was expected to {}, but it did not", stringify!($expected), if $expected { "succeed" } else { "fail" });
                false
            },
            ::std::result::Result::Err(err) => {
                eprintln!("test case {} panicked: {:?}", stringify!($expected), err);
                false
            }
        }
    };
}

macro_rules! run_cases {
    ($($expected:expr => $f:expr;)*) => {
        {
            let mut passed = true;
            $(
                passed = run_case!($expected => $f) && passed;
            )*
            passed
        }
    };
}

impl TestCases {
    pub fn none() -> TestCases {
        TestCases {
            create_file_home: false,
            list_home_directory: false,
            open_extant_file_home: false,
            open_nonexistent_file_home: false,
            os_rng: false,
        }
    }

    pub fn run(&self) -> bool {
        run_cases! {
            self.create_file_home => fs::create_file_home();
            self.list_home_directory => fs::list_home_directory();
            self.open_extant_file_home => fs::open_extant_file_home();
            self.open_nonexistent_file_home => fs::open_nonexistent_file_home();
            self.os_rng => rand::os_rng();
        }
    }
}