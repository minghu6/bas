use inkwellkit::{config::{EmitType, PrintTy}, targets::{Target, InitializationConfig, TargetMachine, RelocMode, CodeModel, FileType}};

use super::{CodeGen, CodeGenResult, CodeGenError};



impl<'ctx> CodeGen<'ctx> {
    ///////////////////////////////////////////////////////////////////////////
    //// Target Generation

    pub(crate) fn gen_file(&self) -> CodeGenResult {
        match self.config.emit_type {
            EmitType::Obj => self.emit_obj(),
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

        machine.write_to_file(
            &self.vmmod.module,
            FileType::Object,
            &self.config.print_type.get_path().unwrap(),
        )?;

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
}

