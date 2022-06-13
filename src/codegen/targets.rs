use std::{
    path::{PathBuf, Path},
    time::{SystemTime, UNIX_EPOCH}, process::{Command, Stdio}, env, fs
};

use inkwellkit::{config::{EmitType, PrintTy}, targets::{Target, InitializationConfig, TargetMachine, RelocMode, CodeModel, FileType}};

use super::{CodeGen, CodeGenResult, CodeGenError};


fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

/// BareLang Installed Home
#[inline]
pub fn bas_home() -> PathBuf {
    let bas_home_str
    = env::var("BAS_HOME").unwrap_or(".".to_string());

    fs::canonicalize(Path::new(
        &bas_home_str
    )).unwrap()
}

/// staticlib
#[inline]
pub fn libbas_o_path() -> PathBuf {
    bas_home().join("libbas.a")
}


impl<'ctx> CodeGen<'ctx> {
    ///////////////////////////////////////////////////////////////////////////
    //// Target Generation

    fn tmp_obj_fname(&self) -> PathBuf {
        let src_path = self.config.print_type.get_path().unwrap();
        let filename = src_path.file_name().unwrap().to_str().unwrap();

        src_path
        .with_file_name(filename.to_owned() + "_" + &unique_suffix())
        .with_extension("o")
    }

    pub(crate) fn gen_file(&self) -> CodeGenResult {
        match self.config.emit_type {
            EmitType::Obj => {
                self.emit_obj()?;

                Ok(())
            },
            EmitType::LLVMIR => self.emit_llvmir(),
            _ => todo!(),
        }
    }

    fn emit_obj(&self) -> CodeGenResult {
        Target::initialize_native(&InitializationConfig::default())
        .map_err(|s| CodeGenError::new(&s))?;

        let triple = TargetMachine::get_default_triple();
        self.vmmod.module.set_triple(&triple);

        let target = Target::from_triple(&triple).unwrap();

        let machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                self.config.optlv.into(),
                RelocMode::Default,
                CodeModel::Default,
            )
            .unwrap();

        self.vmmod.module
            .set_data_layout(&machine.get_target_data().get_data_layout());

        let tmp_input = self.tmp_obj_fname();

        machine.write_to_file(
            &self.vmmod.module,
            FileType::Object,
            &tmp_input,
        )?;

        self.link_core(&tmp_input)?;

        self.clean_obj(&tmp_input)?;

        Ok(())
    }

    fn emit_llvmir(&self) -> CodeGenResult {
        if let PrintTy::File(ref path) = self.config.print_type {
            self.vmmod.module.print_to_file(path).map_err(|llvmstr| {
               CodeGenError::from(llvmstr)
            })
        } else {
            Ok(self.vmmod.module.print_to_stderr())
        }
    }

    fn link_core(&self, input: &Path) -> CodeGenResult {
        Command::new("gcc")
            .arg(input)
            .arg(libbas_o_path().to_str().unwrap())
            .arg("-o")
            .arg(self.config.print_type.get_path().unwrap())
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .status()
            .and_then(|_| Ok(()))
            .or_else(|st| Err(CodeGenError::new(&st.to_string())))
    }

    fn clean_obj(&self, input: &Path) -> CodeGenResult {
        fs::remove_file(input)
        .or_else(|st| Err(CodeGenError::new(&st.to_string())))
    }

}

