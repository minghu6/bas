use std::{path::PathBuf, fs::File, io::BufWriter};

use clap::Command;
use clap_complete::{Shell, generate};
use shellexpand::tilde;


pub fn gen_completions(r#gen: Shell, cmd: &mut Command) {
    match r#gen.to_string().to_uppercase().as_str() {
        "BASH" => {
            let t = tilde("~/.local/share/bash-completion/completions/");
            let dir = PathBuf::from(t.to_string());

            let bin_name = cmd.get_bin_name().unwrap().to_string();
            let fullpath = dir.join(&bin_name);

            let f = File::create(fullpath).unwrap();
            let mut writer = BufWriter::new(f);

            // generate(r#gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
            generate(r#gen, cmd, bin_name, &mut writer);
        }
        _ => unimplemented!(),
    }
}
