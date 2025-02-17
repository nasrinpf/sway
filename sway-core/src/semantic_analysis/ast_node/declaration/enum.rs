use sway_error::handler::{ErrorEmitted, Handler};

use crate::{
    language::{parsed::*, ty, CallPath},
    semantic_analysis::*,
    type_system::*,
};

impl ty::TyEnumDecl {
    pub fn type_check(
        handler: &Handler,
        ctx: TypeCheckContext,
        decl: EnumDeclaration,
    ) -> Result<Self, ErrorEmitted> {
        let EnumDeclaration {
            name,
            type_parameters,
            variants,
            span,
            attributes,
            visibility,
            ..
        } = decl;

        // create a namespace for the decl, used to create a scope for generics
        let mut decl_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut decl_namespace);

        // Type check the type parameters.
        let new_type_parameters =
            TypeParameter::type_check_type_params(handler, ctx.by_ref(), type_parameters)?;

        // Insert them into the current namespace.
        for p in &new_type_parameters {
            p.insert_into_namespace(handler, ctx.by_ref())?;
        }

        // type check the variants
        let mut variants_buf = vec![];
        for variant in variants {
            variants_buf.push(
                match ty::TyEnumVariant::type_check(handler, ctx.by_ref(), variant.clone()) {
                    Ok(res) => res,
                    Err(_) => continue,
                },
            );
        }

        let mut call_path: CallPath = name.into();
        call_path = call_path.to_fullpath(ctx.namespace);

        // create the enum decl
        let decl = ty::TyEnumDecl {
            call_path,
            type_parameters: new_type_parameters,
            variants: variants_buf,
            span,
            attributes,
            visibility,
        };
        Ok(decl)
    }
}

impl ty::TyEnumVariant {
    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        variant: EnumVariant,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();
        let mut type_argument = variant.type_argument;
        type_argument.type_id = ctx
            .resolve_type_with_self(
                handler,
                type_argument.type_id,
                ctx.self_type(),
                &type_argument.span,
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err)));
        Ok(ty::TyEnumVariant {
            name: variant.name.clone(),
            type_argument,
            tag: variant.tag,
            span: variant.span,
            attributes: variant.attributes,
        })
    }
}
