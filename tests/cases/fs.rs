extern crate uuid;

use std::{fs, io, env, mem};
use std::path::{PathBuf};

use self::uuid::Uuid;

pub fn create_file_home() -> bool {
    let home = home_dir();
    let path = home.join(format!("sandbox_test_file_{}", Uuid::new_v4()));
    match fs::File::create(&path) {
        Ok(f) => {
            mem::drop(f);
            fs::remove_file(&path).unwrap();
            true
        },
        Err(ref err) if err.kind() == io::ErrorKind::PermissionDenied => false,
        Err(err) => panic!("unexpected error {}", err),
    }
}

pub fn open_nonexistent_file_home() -> bool {
    let home = home_dir();
    let path = home.join(format!("sandbox_test_file_{}", Uuid::new_v4()));
    match fs::File::open(&path) {
        Ok(_) => panic!("wut?"),
        Err(ref err) if err.kind() == io::ErrorKind::PermissionDenied => false,
        Err(ref err) if err.kind() == io::ErrorKind::NotFound => true,
        Err(err) => panic!("unexpected error {}", err),
    }
}

pub fn open_extant_file_home() -> bool {
    // FIXME: have broker create a known file in home for testing
    let home = home_dir();
    let path = home.join(".profile");
    match fs::File::open(&path) {
        Ok(_) => true,
        Err(ref err) if err.kind() == io::ErrorKind::PermissionDenied => false,
        Err(err) => panic!("unexpected error {}", err),
    }
}

pub fn list_home_directory() -> bool {
    let home = home_dir();
    check_perm_failure!(fs::read_dir(&home))
}

fn home_dir() -> PathBuf {
    PathBuf::from(env::var_os("SANDBOX_TEST_HOME").unwrap())
}