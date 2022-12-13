#[macro_export]
macro_rules! opt_osstr_to_str {
    ($osstr: expr) => {
        $osstr.unwrap().to_str().unwrap()
    };
}


#[macro_export]
macro_rules! ref_source {
    ($span:expr, $c:literal, $f: ident, $src:expr) => {
        let loc = $src.boffset2srcloc($span.from);
        let linestr = $src.linestr($span.from).unwrap();

        let width = std::cmp::min($span.len(), linestr.chars().count());

        writeln!($f, "{linestr}")?;
        writeln!($f, "{}{}", " ".repeat(loc.col - 1), $c.repeat(width))?;
        writeln!(
            $f,
            "--> {}:{}:{}",
            $src.get_path().to_string_lossy(),
            loc.ln,
            loc.col
        )?;
    };
}
