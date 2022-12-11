use std::{env, fs, path::{Path, PathBuf}};


/// BareLang installed home
#[inline]
pub fn bas_home() -> PathBuf {
    let bas_home_str
    = env::var("BAS_HOME").unwrap_or(".".to_string());

    fs::canonicalize(Path::new(
        &bas_home_str
    )).unwrap()
}


/// Staticlib
#[inline]
pub fn libbas_o_path() -> PathBuf {
    bas_home().join("libbas.a")
}


#[inline]
pub fn boostrap_dir() -> PathBuf {
    bas_home().join("boostrap")
}


