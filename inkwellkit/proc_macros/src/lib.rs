extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, Span};
use quote::quote;
use syn::parenthesized;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Expr, Ident, Token};


////////////////////////////////////////////////////////////////////////////////
//// Load VM Common Type
struct LoadVMCommonType {
    ctx: Expr,
}

impl Parse for LoadVMCommonType {
    fn parse(input: ParseStream) -> Result<Self> {
        let context_name = input.parse()?;

        Ok(Self { ctx: context_name })
    }
}

#[proc_macro]
pub fn load_vm_common_ty(input: TokenStream) -> TokenStream {
    let LoadVMCommonType { ctx } =
        parse_macro_input!(input as LoadVMCommonType);

    TokenStream::from(quote! {
        // use inkwell::AddressSpace;

        // Int type
        let i8_t = #ctx.i8_type();
        let i32_t = #ctx.i32_type();
        let i64_t = #ctx.i64_type();
        let i128_t = #ctx.i128_type();

        #[cfg(target_pointer_width = "64")]
        let size_t = #ctx.i64_type();

        #[cfg(target_pointer_width = "32")]
        let size_t = #ctx.i32_type();

        let sizeptr_t = size_t.ptr_type(AddressSpace::Generic);

        // Ret Void Type
        let void_t = #ctx.void_type();

        // Ptr Type
        let i8ptr_t = i8_t.ptr_type(AddressSpace::Generic);
        let i32ptr_t = i32_t.ptr_type(AddressSpace::Generic);
        let i64ptr_t = i64_t.ptr_type(AddressSpace::Generic);
        let i128ptr_t = i128_t.ptr_type(AddressSpace::Generic);

        let i8ptr2_t = i8ptr_t.ptr_type(AddressSpace::Generic);

        // Float Type
        let f64_t = #ctx.f64_type();

    })
}



////////////////////////////////////////////////////////////////////////////////
//// Add VM Function Header (External)


enum VMPriTy {
    TS(TokenStream2),
    Ellipsis,
}

impl Parse for VMPriTy {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![...]) {
            input.parse::<Token![...]>()?;

            return Ok(Self::Ellipsis);
        }

        let mut ptrlv = 0u8;
        while input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            ptrlv += 1;
        }

        let ty = input.parse::<Ident>()?;

        let mut ty_ts = match ty.to_string().as_str() {
            "i8" | "u8" => quote! {
                i8_t
            },
            "usize" | "isize" => quote! {
                size_t
            },
            "i32" | "u32" => quote! {
                i32_t
            },
            "i64" | "u64" => quote! {
                i64_t
            },
            "i128" => quote! {
                i128_t
            },
            "void" => quote! {
                void_t
            },
            other => {
                let other = Ident::new(&format!("{}_t", other), Span::call_site());
                quote! {
                    #other
                }
            },
        };
        for _ in 0..ptrlv {
            ty_ts.extend(quote! {
                .ptr_type(AddressSpace::Generic)
            })
        }

        Ok(Self::TS(ty_ts))
    }
}


struct FunHdr {
    name: Ident,
    args: Vec<TokenStream2>,
    ret: TokenStream2,
    is_var: bool,
}

impl Parse for FunHdr {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;

        let args_buf;
        parenthesized!(args_buf in input);

        let args_prity
            = args_buf.parse_terminated::<VMPriTy, Token![,]>(VMPriTy::parse)?;

        let mut args = vec![];
        let mut is_var = false;
        for arg in args_prity {
            match arg {
                VMPriTy::TS(ts) => args.push(ts),
                VMPriTy::Ellipsis => {
                    is_var = true;
                    break;
                }
            }
        }

        let ret;
        if input.peek(Token![-]) {
            input.parse::<Token![->]>()?;
            ret = if let VMPriTy::TS(ts) = input.parse::<VMPriTy>()? {
                ts
            } else {
                unreachable!()
            };
        } else {
            ret = quote! { void_t };
        }

        Ok(Self {
            name,
            args,
            ret,
            is_var,
        })
    }
}


struct ImplFunHdr {
    module: Ident,
    funhdrs: Punctuated<FunHdr, Token![;]>,
}

impl Parse for ImplFunHdr {
    fn parse(input: ParseStream) -> Result<Self> {
        let module = input.parse::<Ident>()?;
        input.parse::<Token![|]>()?;

        let funhdrs = input.parse_terminated(FunHdr::parse)?;

        Ok(Self { module, funhdrs })
    }
}

/// Cooperate with load_vm_common_ty
#[proc_macro]
pub fn impl_fn_hdr(input: TokenStream) -> TokenStream {
    let ImplFunHdr { module, funhdrs } =
        parse_macro_input!(input as ImplFunHdr);

    let mut ts = quote! {
        load_vm_common_ty!(get_ctx());
    };

    for funhdr in funhdrs {
        let fname = funhdr.name;
        let ret_ts = funhdr.ret;

        let mut args_ts = quote! {};
        for arg in funhdr.args {
            args_ts.extend(quote! {
                #arg.into(),
            });
        }
        let is_var = funhdr.is_var;

        ts.extend(quote! {
            #module.add_function(
                stringify!(#fname),
                #ret_ts
                    .fn_type(&[#args_ts], #is_var),
                Some(Linkage::External)
            );
        });
    }

    TokenStream::from(ts)
}
