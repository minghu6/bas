use inkwellkit::{ impl_fn_hdr, load_vm_common_ty, get_ctx, AddressSpace };

use super::CodeGen;


impl<'ctx> CodeGen<'ctx> {

    pub(crate) fn include_core(&mut self) {
        let module = &self.vmmod.module;

        impl_fn_hdr![module |

        ];
    }

}
