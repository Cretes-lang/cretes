//! Session-local canonical types; children use IDs so destruction is not recursive.
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(pub usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(pub usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(pub usize);
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Error,
    Bool,
    Char,
    Text,
    Integer {
        signed: bool,
        bits: u8,
    },
    Usize,
    Float(u8),
    Bytes,
    Tuple(Vec<TypeId>),
    Builtin {
        name: String,
        arguments: Vec<TypeId>,
    },
    Reference {
        mutable: bool,
        target: TypeId,
    },
    Nominal(SymbolId),
}
#[derive(Debug)]
pub struct Types {
    values: Vec<Type>,
    index: BTreeMap<Type, TypeId>,
}
impl Default for Types {
    fn default() -> Self {
        let mut t = Self {
            values: vec![],
            index: BTreeMap::new(),
        };
        t.intern(Type::Error);
        t
    }
}
impl Types {
    pub const ERROR: TypeId = TypeId(0);
    pub fn intern(&mut self, ty: Type) -> TypeId {
        if let Some(id) = self.index.get(&ty) {
            return *id;
        }
        let id = TypeId(self.values.len());
        self.values.push(ty.clone());
        self.index.insert(ty, id);
        id
    }
    pub fn get(&self, id: TypeId) -> Option<&Type> {
        self.values.get(id.0)
    }
    pub fn iter(&self) -> impl Iterator<Item = (TypeId, &Type)> {
        self.values.iter().enumerate().map(|(i, t)| (TypeId(i), t))
    }
    pub fn primitive(&mut self, name: &str) -> Option<TypeId> {
        let ty = match name {
            "bool" => Type::Bool,
            "char" => Type::Char,
            "text" => Type::Text,
            "Bytes" => Type::Bytes,
            "usize" => Type::Usize,
            "i8" => Type::Integer {
                signed: true,
                bits: 8,
            },
            "i16" => Type::Integer {
                signed: true,
                bits: 16,
            },
            "i32" => Type::Integer {
                signed: true,
                bits: 32,
            },
            "i64" => Type::Integer {
                signed: true,
                bits: 64,
            },
            "u8" => Type::Integer {
                signed: false,
                bits: 8,
            },
            "u16" => Type::Integer {
                signed: false,
                bits: 16,
            },
            "u32" => Type::Integer {
                signed: false,
                bits: 32,
            },
            "u64" => Type::Integer {
                signed: false,
                bits: 64,
            },
            "f32" => Type::Float(32),
            "f64" => Type::Float(64),
            _ => return None,
        };
        Some(self.intern(ty))
    }
}
pub fn arity(name: &str) -> Option<usize> {
    match name {
        "Seq" | "Set" | "Option" | "Box" => Some(1),
        "Map" | "Result" => Some(2),
        _ => None,
    }
}
pub fn reserved(name: &str) -> bool {
    matches!(
        name,
        "bool"
            | "char"
            | "text"
            | "Bytes"
            | "usize"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "f32"
            | "f64"
            | "discard"
    ) || arity(name).is_some()
}
