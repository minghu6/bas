use itertools::Itertools;
use m6lexerkit::{Symbol, sym2str, str2sym};

use crate::ast_lowering::{AType, APriType};



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl AType {
    pub fn ident_name(&self) -> String {
        match self {
            Self::Pri(prity) => prity.ident_name(),
            Self::Arr(prity, d) => {
                format!("{}{}{}", "[".repeat(*d as _), prity.ident_name(),"]".repeat(*d as _) )
            },
            Self::AA(_) => todo!(),
            Self::Void => format!("()"),
            Self::PH => format!("???"),
            Self::Never => format!("!")
        }
    }
    #[allow(unused)]
    pub fn unident_name(s: &str) -> Option<Self> {
        // let ss = sym2str(sym);
        // let s = ss.as_str();

        if s.len() == 0 {
            return None;
        }

        match &s[0..1] {
            "[" => {
                let mut prefixn = 1;
                let mut contentn = 1;
                let mut postfixn = 1;
                let mut chars = s.chars().skip(1);

                while let Some(c) = chars.next() && c == '[' {
                    prefixn += 1;
                }

                while let Some(c) = chars.next() && c != ']' {
                    contentn += 1;
                }

                while let Some(c) = chars.next() && c == ']' {
                    postfixn += 1;
                }

                if prefixn == postfixn {
                    if let Some(prity) = APriType::unident_name(&s[prefixn..prefixn+contentn]) {
                        return Some(AType::Arr(prity, prefixn as u8))
                    }
                }
            },
            _ => {
                if let Some(prity) = APriType::unident_name(s) {
                    return Some(AType::Pri(prity))
                }
            }
        }

        None

    }
}


impl APriType {
    pub fn ident_name(&self) -> String {
        match self {
            Self::Float(byte) => {
                let bits = byte * 8;
                format!("f{bits}")
            }
            Self::Int(sbyte) => {
                let signed;

                if *sbyte < 0 {
                    signed = "i"
                } else {
                    signed = "u"
                };

                let bits = (sbyte * 8).abs();

                format!("{signed}{bits}")
            },
            Self::Ptr => {
                format!("ptr")
            },
            Self::OpaqueStruct(struct_name) => {
                format!("{{{}}}", sym2str(*struct_name))
            },
        }
    }

    pub(crate) fn unident_name(s: &str) -> Option<Self> {
        let mut chars = s.chars();

        if let Some(c) = chars.next() {
            match c {
                'i' | 'u' | 'f' => {
                    if let Ok(bits) = &s[1..].parse() {
                        if bits % 8 == 0 {
                            let bytes: u8 = bits / 8u8;

                            if c == 'i' {
                                return Some(APriType::Int(0-(bytes as i8)))
                            }
                            else if c == 'u' {
                                return Some(APriType::Int(bytes as i8))
                            }
                            else {
                                return Some(APriType::Float(bytes))
                            }
                        }
                    }
                },
                'p' => {
                    return Some(APriType::Ptr)
                },
                '{' => {
                    if let Some(delidx) = s.find('}') {
                        return Some(APriType::OpaqueStruct(str2sym(&s[1..delidx])))
                    }
                },
                _ => ()
            }
        }

        None
    }


}



////////////////////////////////////////////////////////////////////////////////
//// Function

pub fn mangling(name: Symbol, atys: &[AType]) -> Symbol {

    let param_postfix = atys
    .into_iter()
    .map(|aty| aty.ident_name())
    .join("#");

    str2sym(&format!("{}@{}", sym2str(name), param_postfix))
}

#[allow(unused)]
pub fn unmangling(mangling_name: Symbol) -> Option<(Symbol, Vec<AType>)> {

    let ss = sym2str(mangling_name);
    let s = ss.as_str();

    let split: Vec<&str> = s.split('@').collect();

    if split.len() == 2 {
        let base = str2sym(split[0]);
        let postfix = split[1];

        let tys: Vec<Option<AType>> = postfix
            .split('#')
            .map(|ty| AType::unident_name(ty))
            .collect();

        if tys.iter().any(|x| x.is_some()) {
            let tys = tys
                .into_iter()
                .map(|x| x.unwrap())
                .collect();

            return Some((base, tys));
        }
    }

    None
}



#[cfg(test)]
mod tests {

    #[test]
    fn test_mangling() {
        println!("{{{}}}", "abc");
    }
}
