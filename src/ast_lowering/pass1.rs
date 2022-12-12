use m6lexerkit::{str2sym, Span, SrcFileInfo, Symbol};

use super::{ ExtSymSet, AnItem, TokenTree2};
use crate::{
    ast_lowering::{
        analyze_attrs, analyze_pat_no_top, analyze_ty,
        get_fullname_by_fn_header, write_diagnosis, A3ttrName, A3ttrs, AFnDec,
        AMod, AParamPat, AType, SemanticError,
        SemanticErrorReason as R, AnExtFnDec,
    },
    opt_osstr_to_str,
    parser::{SyntaxType as ST, TokenTree},
};


pub struct SemanticAnalyzerPass1 {
    pub src: SrcFileInfo,
    pub amod: AMod,
    pub ess: ExtSymSet,
    cause_lists: Vec<(R, Span)>,
}

pub(crate) type Pass1Result = Result<Pass1Export, SemanticError>;


pub struct Pass1Export {
    pub src: SrcFileInfo,
    pub tt2: TokenTree2,
    pub amod: AMod,
    pub ess: ExtSymSet
}


impl SemanticAnalyzerPass1 {
    pub(crate) fn run(src: SrcFileInfo, tt: TokenTree, ess: ExtSymSet) -> Pass1Result {
        let amod = AMod::init(str2sym(opt_osstr_to_str!(&src
            .get_path()
            .file_stem())));

        let it = Self {
            src,
            ess,
            amod,
            cause_lists: vec![],
        };

        it.analyze(tt)
    }

    fn analyze(mut self, tt: TokenTree) -> Pass1Result {
        let mut items = vec![];

        for (ty, sn) in tt.subs.into_iter() {
            if ty == ST::Item {
                if let Some(anitem) = self.do_analyze_item(sn.into_tt()) {
                    items.push(anitem);
                }
            }
        }

        if self.cause_lists.is_empty() {
            Ok(Pass1Export {
                src: self.src,
                tt2: TokenTree2 { items },
                amod: self.amod,
                ess: self.ess
            })
        } else {
            Err(SemanticError {
                cause_lists: self.cause_lists,
                src: self.src,
            })
        }
    }

    pub(crate) fn do_analyze_item(&mut self, mut tt: TokenTree) -> Option<AnItem> {
        let mut p = 0;

        let attrs;
        if tt[p].0 == ST::Attrs {
            attrs = self.analyze_attrs(&tt[p].1.as_tt());
            p += 1;
        }
        else {
            attrs = A3ttrs::new();
        }

        if tt[p].0 == ST::Function {
            self.do_analyze_fn(attrs, tt.move_elem(p).1.into_tt())
        }
        else {
            unreachable!()
        }
    }

    /// Function Definition or Exrernal Function Declare
    pub(crate) fn do_analyze_fn(&mut self, attrs: A3ttrs, tt: TokenTree) -> Option<AnItem> {
        // Syntax Node Stream
        let mut p = 0;

        /* Analyze fn name, params and ret */

        let idt = tt[p].1.as_tok().clone();
        let fn_base_name = idt.value;
        p += 1;

        debug_assert_eq!(tt[p].0, ST::FnParams);
        let params = self.analyze_fn_params(tt[p].1.as_tt());
        p += 1;

        let ret;
        if tt[p].0 == ST::Type {
            ret = self.analyze_ty(tt[p].1.as_tt());
            p += 1;
        } else {
            ret = AType::Void;
        }

        let full_name;
        if attrs.has(A3ttrName::NoMangle) {
            full_name = fn_base_name
        } else {
            full_name = get_fullname_by_fn_header(fn_base_name, &params);
        }

        if let Some(_afn) = self.find_func_by_name(full_name) {
            write_diagnosis(
                &mut self.cause_lists,
                R::DupItemDef {
                    name: full_name,
                },
                idt.span(),
            );
            return None;
        }

        if tt[p].0 == ST::semi {  // It's external declare
            let afn = AnExtFnDec {
                attrs,
                full_name,
                params,
                ret,
                symbol_name: full_name
            };

            self.amod.efns.insert(full_name, afn);

            None
        }
        else {
            debug_assert_eq!(tt[p].0, ST::BlockExpr);

            let afn = AFnDec {
                // body_idx: None,
                idt,
                attrs,
                name: full_name,
                params,
                ret,
            };

            self.amod.afns.insert(full_name, afn);

            Some(AnItem::Fn { name: full_name, body: tt.subs.into_iter().last().unwrap().1.into_tt() })
        }

    }

    pub(crate) fn analyze_fn_params(
        &mut self,
        tt: &TokenTree,
    ) -> Vec<AParamPat> {
        let mut sns = tt.subs.iter().peekable();
        let mut params = vec![];

        while !sns.is_empty() && sns.peek().unwrap().0 == ST::FnParam {
            let (param_ty, param_sn) = sns.next().unwrap();

            // println!("analyze fn_param_pats: param_sn {param_sn:#?}");

            if *param_ty == ST::id {
                write_diagnosis(
                    &mut self.cause_lists,
                    R::LackFormalParam,
                    param_sn.as_tok().span(),
                );
            }

            params.push(self.analyze_fn_param(param_sn.as_tt()));
        }

        params
    }

    pub(crate) fn analyze_fn_param(&mut self, tt: &TokenTree) -> AParamPat {
        // PatNoTop
        let formal = self.analyze_pat_no_top(tt[0].1.as_tt());
        let ty = self.analyze_ty(&tt[1].1.as_tt());

        AParamPat { formal, ty }
    }

    pub(crate) fn analyze_pat_no_top(&mut self, tt: &TokenTree) -> Symbol {
        analyze_pat_no_top(tt)
    }

    pub(crate) fn analyze_ty(&mut self, tt: &TokenTree) -> AType {
        analyze_ty(&mut self.cause_lists, tt)
    }

    pub(crate) fn analyze_attrs(&mut self, tt: &TokenTree) -> A3ttrs {
        analyze_attrs(&mut self.cause_lists, tt)
    }

    // pub(crate) fn find_func(
    //     &self,
    //     name: &str,
    //     atys: &[AType],
    // ) -> Option<AnExtFnDec> {
    //     let fullname = mangling(str2sym(name), atys);

    //     self.find_func_by_name(fullname)
    // }

    pub(crate) fn find_func_by_name(
        &self,
        fullname: Symbol,
    ) -> Option<AnExtFnDec> {
        if let Some(afndec) = self.amod.afns.get(&fullname) {
            Some(afndec.as_ext_fn_dec())
        } else if let Some(afndec) = self.amod.efns.get(&fullname) {
            Some(afndec.clone())
        }
        else if let Some(afndec) = self.ess.find_func_by_name(fullname) {
            Some(afndec.clone())
        } else {
            None
        }
    }
}
