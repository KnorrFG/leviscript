/// generates a type for an enum node, which always only contains variants with one unnamed field,
/// which are further nodes. It implements From<T> for each child type, and it implements
/// Compilable, by calling compile() on the child
macro_rules! mk_enum_node{
    ($name:ident $(, $child_ty:tt)+) => {
        #[derive(Debug, Serialize, Deserialize, Clone, AstNode)]
        pub enum $name {
            $($child_ty($child_ty),)*
        }

    $(
        impl From<$child_ty> for $name {
            fn from(child: $child_ty) -> Self {
                Self::$child_ty(child)
            }
        }
    )*

        impl crate::compiler::Compilable for $name {
            fn compile(
                &self,
                builder: crate::core::ByteCodeBuilder,
                expr_types: &crate::type_inference::TypeIndex,
            ) -> crate::compiler::Result<crate::core::ByteCodeBuilder> {
                match self {
                    $(Self::$child_ty(t) => t.compile(builder, expr_types),)*
                }
            }
        }

        impl crate::type_inference::TypeInferable for $name {
            fn infer_types(
                &self,
                env: crate::type_inference::Environment,
                type_idx: crate::type_inference::TypeIndex,
            ) -> crate::type_inference::Result<(
                crate::type_inference::Environment,
                crate::type_inference::TypeIndex)> {
                match self {
                    $(Self::$child_ty(t) => t.infer_types(env, type_idx),)*
                }
            }
        }
    };
}
pub(crate) use mk_enum_node;

macro_rules! define_ast_node_ref {
    ($($types:tt,)+) => {
        #[derive(Clone, Copy)]
        pub enum AstNodeRef<'a> {
            $(
                $types(&'a $types),
            )*
        }

        $(
            impl<'a> From<&'a $types> for AstNodeRef<'a> {
                fn from(x: &'a $types) -> AstNodeRef<'a> {
                    Self::$types(x)
                }
            }
        )*
            impl<'a> Deref for AstNodeRef<'a> {
                type Target = dyn AstNode;
                fn deref(&self) -> &Self::Target {
                    match self {
                        $(Self::$types(x) => *x,)*
                    }
                }
            }
    };
}
pub(crate) use define_ast_node_ref;
