use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use inkwellkit::{
    config::{EmitType, PrintTy, TargetType},
    targets::{
        CodeModel, FileType, InitializationConfig, RelocMode, Target,
        TargetMachine,
    },
};

use super::{CodeGen, CodeGenError, CodeGenResult2};
use crate::env::{libbas_o_path, core_lib_path};


fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
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

    pub(crate) fn gen_file(&self) -> CodeGenResult2 {
        match self.config.emit_type {
            EmitType::Obj => {
                self.emit_obj()?;

                Ok(())
            }
            EmitType::LLVMIR => self.emit_llvmir(),
            _ => todo!(),
        }
    }

    fn emit_obj(&self) -> CodeGenResult2 {
        if matches!(self.config.print_type, PrintTy::StdErr) {
            return Err(CodeGenError(
                format!(
                    "Unsupported output type {:?} for emit obj",
                    self.config.print_type
                )
            ))
        }

        Target::initialize_native(&InitializationConfig::default())
            .map_err(|s| CodeGenError(s))?;

        let triple = TargetMachine::get_default_triple();
        self.vmmod.module.set_triple(&triple);

        let target = Target::from_triple(&triple).unwrap();

        let machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                self.config.optlv.into(),
                self.reloc_mode(),
                CodeModel::Default,
            )
            .unwrap();

        self.vmmod
            .module
            .set_data_layout(&machine.get_target_data().get_data_layout());


        match self.config.target_type {
            TargetType::Bin => {
                let tmp_input = self.tmp_obj_fname();

                machine.write_to_file(
                    &self.vmmod.module,
                    FileType::Object,
                    &tmp_input,
                )?;

                self.link_core(&tmp_input)?;
                self.clean_obj(&tmp_input)?;
            }
            TargetType::ReLoc => {
                machine.write_to_file(
                    &self.vmmod.module,
                    FileType::Object,
                    &self.config.print_type.get_path().unwrap(),
                )?;
            }
            _ => todo!(),
        }

        Ok(())
    }

    fn emit_llvmir(&self) -> CodeGenResult2 {
        if let PrintTy::File(ref path) = self.config.print_type {
            self.vmmod
                .module
                .print_to_file(path)
                .map_err(|llvmstr| CodeGenError::from(llvmstr))
        } else {
            Ok(self.vmmod.module.print_to_stderr())
        }
    }

    fn reloc_mode(&self) -> RelocMode {
        match self.config.target_type {
            TargetType::Bin => RelocMode::Default,
            TargetType::ReLoc => RelocMode::PIC,
            TargetType::DyLib => RelocMode::DynamicNoPic,
        }
    }

    fn link_core(&self, input: &Path) -> CodeGenResult2 {
        Command::new("gcc")
            .arg(input)
            .arg(libbas_o_path())
            .arg(core_lib_path())
            .arg("-o")
            .arg(self.config.print_type.get_path().unwrap())
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .status()
            .and_then(|_| Ok(()))
            .or_else(|st| Err(CodeGenError(st.to_string())))
    }

    fn clean_obj(&self, input: &Path) -> CodeGenResult2 {
        fs::remove_file(input)
            .or_else(|st| Err(CodeGenError(st.to_string())))
    }
}
