#![feature(exit_status_error)]

use std::{
    error::Error,
    path::Path,
    process::{Command, Stdio},
};

use inkwellkit::{
    get_ctx,
    module::{Linkage, Module},
    ret_as_bv,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    AddressSpace, OptimizationLevel, VMMod,
    // types::{ RetTypeEnum }
};
use inkwellkit_macros::{impl_fn_hdr, load_vm_common_ty};

fn action() -> Result<(), Box<dyn Error>> {
    let module_name = "posix_llvm2";
    let vmmod = VMMod::new(module_name);

    VMMod::include_stdio(&vmmod.module);
    VMMod::include_unistd(&vmmod.module);
    VMMod::include_fcntl(&vmmod.module);

    let builder = VMMod::get_builder();
    load_vm_common_ty!(get_ctx());

    // begin main
    let blk_main = vmmod.append_main();
    builder.position_at_end(blk_main);

    // open it
    let fn_open = vmmod.module.get_function("open").unwrap();
    let (fns, _) = vmmod.build_local_str(&builder, "hello.txt");
    let flags = i32_t.const_int(
        (libc::O_CREAT | libc::O_WRONLY | libc::O_APPEND)
            .try_into()
            .unwrap(),
        true,
    );
    let mode = i32_t.const_int(0o6666, false);

    let fd = ret_as_bv!(builder.build_call(fn_open, &[fns.into(), flags.into(), mode.into()], ""));

    // write to it
    let fn_write = vmmod.module.get_function("write").unwrap();
    let (content, content_len) = vmmod.build_local_str(&builder, "_____写入了");
    let write_ok = ret_as_bv!(builder.build_call(
        fn_write,
        &[fd.into(), content.into(), content_len.into()],
        "",
    ));

    vmmod.build_call_printf(&builder, "write res: %d\n", &[write_ok.into()]);

    let fn_main = vmmod.module.get_function("main").unwrap();

    let struct_vec_t = get_ctx().opaque_struct_type("dynvec");
    let vec_t = struct_vec_t.ptr_type(AddressSpace::Generic);
    // let vec_t = struct_vec_t;


    let module = &vmmod.module;

    impl_fn_hdr![module|
        vec_new_i32(i32) -> vec;
        vec_push_i32(vec, i32) -> i32;
        vec_get_i32(vec, i32) -> i32;
    ];

    // let fn_vec_new_i32_t = vec_t.fn_type(&[i32_t.into()], false);

    // let fn_vec_push_i32_t = i32_t.fn_type(&[vec_t.into(), i32_t.into()], false);

    // let fn_vec_get_i32_t = i32_t.fn_type(&[vec_t.into(), i32_t.into()], false);

    // let fn_vec_new_i32 =
    //     vmmod
    //         .module
    //         .add_function("vec_new_i32", fn_vec_new_i32_t, None);
    // let fn_vec_push_i32 =
    //     vmmod
    //         .module
    //         .add_function("vec_push_i32", fn_vec_push_i32_t, None);
    // let fn_vec_get_i32 =
    //     vmmod
    //         .module
    //         .add_function("vec_get_i32", fn_vec_get_i32_t, None);

    let v = ret_as_bv!(builder.build_call(
        vmmod.get_unchecked_fn("vec_new_i32"),
        &[vmmod.i32(20).into()],
        ""
    ));
    builder.build_call(
        vmmod.get_unchecked_fn("vec_push_i32"),
        &[v.into(), vmmod.i32(1).into()],
        "",
    );
    builder.build_call(
        vmmod.get_unchecked_fn("vec_push_i32"),
        &[v.into(), vmmod.i32(4).into()],
        "",
    );
    builder.build_call(
        vmmod.get_unchecked_fn("vec_push_i32"),
        &[v.into(), vmmod.i32(9).into()],
        "",
    );

    let get_0 = ret_as_bv!(builder.build_call(
        vmmod.get_unchecked_fn("vec_get_i32"),
        &[v.into(), vmmod.i32(0).into()],
        ""
    ));

    vmmod.build_call_printf(&builder, "get0: %d\n", &[get_0.into()]);

    // // TEST IF ELSE
    // // compare int res > 5
    // let write_ok_cast =
    //     builder.build_int_truncate(write_ok.into_int_value(), i32_t, "");
    // let if_cond =
    //     vmmod.bsgt(&builder, write_ok_cast, vmmod.i32(123));
    // // let (then_builder, builder) = vmmod.bif(&builder, if_cond, fn_main);

    // let (then_builder, else_builder, builder) =
    //     vmmod.bif_else(&builder, if_cond, fn_main);

    // vmmod.build_call_printf(
    //     &then_builder,
    //     &module,
    //     " Enter Then: write res: %d\n",
    //     &[write_ok.into()],
    // );
    // vmmod.build_call_printf(
    //     &else_builder,
    //     &module,
    //     " Enter Else: write res: %d\n",
    //     &[write_ok.into()],
    // );

    // close it

    // end main
    builder.build_return(Some(i64_t.const_zero()));
    fn_main.verify(true);

    print_obj(&vmmod.module, OptimizationLevel::None)?;
    println!("->: {}", module_name);
    let bin_output = module_name.to_owned() + ".out";
    link(&bin_output, &["./output.o", "libbas.a"])?;
    run_bin(&bin_output)
}

fn print_obj<'ctx>(module: &Module<'ctx>, optlv: OptimizationLevel) -> Result<(), Box<dyn Error>> {
    Target::initialize_native(&InitializationConfig::default())?;

    let triple = TargetMachine::get_default_triple();
    module.set_triple(&triple);

    let target = Target::from_triple(&triple).unwrap();

    let machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            optlv,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    module.set_data_layout(&machine.get_target_data().get_data_layout());

    module.print_to_stderr();

    machine.write_to_file(&module, FileType::Assembly, Path::new("./output.asm"))?;

    machine.write_to_file(&module, FileType::Object, Path::new("./output.o"))?;

    Ok(())
}

#[inline]
pub fn link(output: &str, input_list: &[&str]) -> Result<(), Box<dyn Error>> {
    Command::new("gcc")
        .args(input_list)
        // cargo rustc -- --print native-static-libs
        .args("-lgcc_s -lutil -lrt -lpthread -lm -ldl -lc".split(" "))
        .arg("-o")
        .arg(output)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .spawn()?
        .wait()?
        .exit_ok()?;

    Ok(())
}

#[inline]
pub fn run_bin(output: &str) -> Result<(), Box<dyn Error>> {
    Command::new(&format!("./{}", output))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .status()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    action()
}
