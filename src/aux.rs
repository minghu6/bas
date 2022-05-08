#[macro_export]
macro_rules! opt_osstr_to_str {
    ($osstr: expr) => {
        $osstr.unwrap().to_str().unwrap()
    };
}
