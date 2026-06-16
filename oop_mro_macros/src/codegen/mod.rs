use std::collections::{BTreeMap, HashSet};

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, GenericParam, Generics, Ident, Type};

use crate::ast::{
    AssociatedConstDef, ClassDef, ClassItem, ConstructorDef, MethodDef, StaticFieldDef,
};
use crate::generics::{
    async_output_type_with_substitutions, method_signature_key_in_context, substitute_type,
    substituted_return_type, substitutions_for_class_type,
};
use crate::model::{Graph, MethodInfo, ReceiverKind, VtableSlot};
use crate::names::{
    base_cast_method_ident, base_field_ident, default_base_trait_ident, private_module_ident,
    static_field_ident, virtual_base_field_ident, virtual_impl_ident, vtable_cast_mut_field_ident,
    vtable_cast_mut_function_ident, vtable_cast_ref_field_ident, vtable_cast_ref_function_ident,
    vtable_factory_ident, vtable_field_ident, vtable_function_ident, vtable_ident,
};
use crate::types::{
    ancestor_type, ancestor_type_for_path, async_dispatch_lifetime, async_output_type,
    base_cast_trait_for_actual_class, boxed_future_type, cast_target_key, class_constructor,
    class_type, class_type_tokens, constructor_arg_idents, find_base_path,
    generics_with_async_lifetime, signature_in_context, type_key, type_with_elided_refs_lifetime,
    vtable_type_for_actual_class, vtable_type_for_class,
};
use crate::validate::public_if_inherited;

mod casts;
mod constructors;
mod downcast;
mod impls;
mod metadata;
mod statics;
mod structs;
mod vtables;

pub(crate) fn generate(graph: &Graph) -> TokenStream2 {
    let private_module = structs::generate_private_module(graph);
    let static_fields = statics::generate_static_fields(graph);
    let vtable_structs = graph
        .classes
        .iter()
        .enumerate()
        .filter(|(index, _)| needs_runtime_metadata(graph, *index))
        .map(|(index, class)| vtables::generate_vtable_struct(graph, index, class));
    let structs = graph
        .classes
        .iter()
        .enumerate()
        .map(|(index, class)| structs::generate_struct(graph, index, class));
    let base_cast_traits = graph
        .classes
        .iter()
        .enumerate()
        .map(|(index, class)| casts::generate_base_cast_trait(graph, index, class));
    let impls = graph
        .classes
        .iter()
        .enumerate()
        .map(|(index, class)| impls::generate_impls(graph, index, class));
    let base_cast_impls = casts::generate_base_cast_impls(graph);
    let vtable_items = vtables::generate_vtable_items(graph);
    let downcast_impls = downcast::generate_downcast_impls(graph);

    quote! {
        #private_module
        #static_fields
        #(#vtable_structs)*
        #(#structs)*
        #(#base_cast_traits)*
        #(#impls)*
        #base_cast_impls
        #vtable_items
        #downcast_impls
    }
}

fn interface_methods(graph: &Graph, index: usize) -> Vec<MethodInfo> {
    let mut methods = BTreeMap::new();

    for (name, method) in &graph.selected_methods[index] {
        methods.insert(name.clone(), method.clone());
    }

    for (name, method) in &graph.abstract_methods[index] {
        methods.insert(name.clone(), method.clone());
    }

    methods.into_values().collect()
}

fn has_virtual_interface(graph: &Graph, index: usize) -> bool {
    !graph.selected_methods[index].is_empty() || !graph.abstract_methods[index].is_empty()
}

fn needs_runtime_metadata(graph: &Graph, index: usize) -> bool {
    has_virtual_interface(graph, index) || has_virtual_inheritance(graph, index)
}

fn has_virtual_inheritance(graph: &Graph, index: usize) -> bool {
    graph.base_edges[index]
        .iter()
        .any(|edge| edge.is_virtual || has_virtual_inheritance(graph, edge.base))
}

fn vtable_slots(graph: &Graph, index: usize) -> Vec<VtableSlot> {
    let mut slots = Vec::new();
    collect_vtable_slots(graph, index, index, Vec::new(), &mut slots);
    slots
}

#[derive(Clone)]
struct AncestorView {
    class_index: usize,
    actual: Type,
    path: Vec<usize>,
}

fn ancestor_views(graph: &Graph, start: usize) -> Vec<AncestorView> {
    let mut views = Vec::new();
    let mut seen = HashSet::new();
    let own_actual = class_type(&graph.classes[start]);
    seen.insert(cast_target_key(start, &own_actual));
    views.push(AncestorView {
        class_index: start,
        actual: own_actual,
        path: Vec::new(),
    });
    collect_ancestor_views(graph, start, start, Vec::new(), &mut seen, &mut views);
    views
}

fn collect_ancestor_views(
    graph: &Graph,
    root: usize,
    current: usize,
    path: Vec<usize>,
    seen: &mut HashSet<String>,
    views: &mut Vec<AncestorView>,
) {
    for edge in &graph.base_edges[current] {
        let mut next_path = path.clone();
        next_path.push(edge.base);
        let actual = ancestor_type_for_path(graph, root, &next_path);
        if seen.insert(cast_target_key(edge.base, &actual)) {
            views.push(AncestorView {
                class_index: edge.base,
                actual,
                path: next_path.clone(),
            });
        }
        collect_ancestor_views(graph, root, edge.base, next_path, seen, views);
    }
}

fn collect_vtable_slots(
    graph: &Graph,
    complete: usize,
    index: usize,
    path: Vec<usize>,
    slots: &mut Vec<VtableSlot>,
) {
    if needs_runtime_metadata(graph, index) {
        slots.push(VtableSlot {
            ancestor: index,
            path: path.clone(),
        });
    }

    for edge in &graph.base_edges[index] {
        let base = edge.base;
        let mut base_path = path.clone();
        base_path.push(base);
        if edge.is_virtual {
            let actual = ancestor_type_for_path(graph, complete, &base_path);
            if canonical_virtual_owner_path(graph, complete, base, &actual).as_deref()
                == Some(&path)
            {
                collect_vtable_slots(graph, complete, base, base_path, slots);
            }
        } else {
            collect_vtable_slots(graph, complete, base, base_path, slots);
        }
    }
}

fn accessor_body(graph: &Graph, start: usize, target: usize, mutable: bool) -> TokenStream2 {
    if path_has_virtual_edge(graph, start, target) {
        return dynamic_accessor_body(graph, start, target, mutable);
    }

    static_ref_expr(graph, start, target, quote! { self }, mutable)
}

fn cast_target_id(graph: &Graph, class_index: usize, actual: &Type) -> usize {
    graph.cast_target_ids[&cast_target_key(class_index, actual)]
}

fn dynamic_accessor_body(
    graph: &Graph,
    start: usize,
    target: usize,
    mutable: bool,
) -> TokenStream2 {
    let target_ty = ancestor_type(graph, start, target);
    dynamic_accessor_body_for_actual(graph, start, target, &target_ty, mutable)
}

fn dynamic_accessor_body_for_actual(
    graph: &Graph,
    start: usize,
    target: usize,
    target_ty: &Type,
    mutable: bool,
) -> TokenStream2 {
    let start_ty = class_type_tokens(&graph.classes[start]);
    let target_id = cast_target_id(graph, target, target_ty);

    if mutable {
        let cast_mut = vtable_cast_mut_field_ident();
        quote! {
            unsafe {
                let ptr = (self.__oop_vtable.#cast_mut)(
                    self as *mut #start_ty,
                    #target_id,
                ).expect("virtual base cast target must exist");
                &mut *(ptr as *mut #target_ty)
            }
        }
    } else {
        let cast_ref = vtable_cast_ref_field_ident();
        quote! {
            unsafe {
                let ptr = (self.__oop_vtable.#cast_ref)(
                    self as *const #start_ty,
                    #target_id,
                ).expect("virtual base cast target must exist");
                &*(ptr as *const #target_ty)
            }
        }
    }
}

fn static_ref_expr(
    graph: &Graph,
    complete: usize,
    target: usize,
    root: TokenStream2,
    mutable: bool,
) -> TokenStream2 {
    if complete == target {
        return root;
    }

    let path = find_base_path(graph, complete, target).expect("ancestor path must exist");
    static_ref_expr_for_path(graph, complete, &path, root, mutable)
}

fn static_ref_expr_for_path(
    graph: &Graph,
    complete: usize,
    path: &[usize],
    root: TokenStream2,
    mutable: bool,
) -> TokenStream2 {
    let (tokens, is_ref) = static_access_expr_for_path(graph, complete, path, root, mutable);
    if is_ref {
        tokens
    } else if mutable {
        quote! { &mut (#tokens) }
    } else {
        quote! { &(#tokens) }
    }
}

fn static_access_expr_for_path(
    graph: &Graph,
    complete: usize,
    path: &[usize],
    root: TokenStream2,
    mutable: bool,
) -> (TokenStream2, bool) {
    if path.is_empty() {
        return (root, true);
    }

    let mut current = complete;
    let mut tokens = root.clone();
    let mut tokens_are_ref = true;
    let mut traversed = Vec::new();

    for &next in path {
        traversed.push(next);
        if edge_is_virtual(graph, current, next) {
            let actual = ancestor_type_for_path(graph, complete, &traversed);
            tokens = virtual_base_ref_expr(graph, complete, next, &actual, root.clone(), mutable);
            tokens_are_ref = true;
        } else {
            let field = base_field_ident(&graph.names[next]);
            tokens = quote! { (#tokens).#field };
            tokens_are_ref = false;
        }
        current = next;
    }

    (tokens, tokens_are_ref)
}

fn path_has_virtual_edge_for_path(graph: &Graph, start: usize, path: &[usize]) -> bool {
    let mut current = start;
    for &next in path {
        if edge_is_virtual(graph, current, next) {
            return true;
        }
        current = next;
    }
    false
}

fn virtual_base_ref_expr(
    graph: &Graph,
    complete: usize,
    target: usize,
    actual: &Type,
    root: TokenStream2,
    mutable: bool,
) -> TokenStream2 {
    let slot = virtual_base_slot_expr(graph, complete, target, actual, root, mutable);
    if mutable {
        quote! { unsafe { #slot.assume_init_mut() } }
    } else {
        quote! { unsafe { #slot.assume_init_ref() } }
    }
}

fn virtual_base_slot_expr(
    graph: &Graph,
    complete: usize,
    target: usize,
    actual: &Type,
    root: TokenStream2,
    mutable: bool,
) -> TokenStream2 {
    let owner_path = canonical_virtual_owner_path(graph, complete, target, actual)
        .expect("virtual owner must exist");
    let owner = owner_path.last().copied().unwrap_or(complete);
    let owner_ref = if owner == complete {
        root
    } else {
        static_ref_expr(graph, complete, owner, root, mutable)
    };
    let field = virtual_base_field_ident(&graph.names[target]);
    quote! { (#owner_ref).#field }
}

fn path_has_virtual_edge(graph: &Graph, start: usize, target: usize) -> bool {
    if start == target {
        return false;
    }

    let Some(path) = find_base_path(graph, start, target) else {
        return false;
    };
    let mut current = start;
    for next in path {
        if edge_is_virtual(graph, current, next) {
            return true;
        }
        current = next;
    }
    false
}

fn edge_is_virtual(graph: &Graph, current: usize, base: usize) -> bool {
    graph.base_edges[current]
        .iter()
        .find(|edge| edge.base == base)
        .map(|edge| edge.is_virtual)
        .unwrap_or(false)
}

fn canonical_virtual_owner_path(
    graph: &Graph,
    complete: usize,
    target: usize,
    actual: &Type,
) -> Option<Vec<usize>> {
    find_canonical_virtual_owner_path(graph, complete, complete, target, actual, Vec::new())
}

fn find_canonical_virtual_owner_path(
    graph: &Graph,
    complete: usize,
    current: usize,
    target: usize,
    actual: &Type,
    path: Vec<usize>,
) -> Option<Vec<usize>> {
    for edge in &graph.base_edges[current] {
        let mut child_path = path.clone();
        child_path.push(edge.base);
        let edge_actual = ancestor_type_for_path(graph, complete, &child_path);
        if edge.is_virtual && edge.base == target && type_key(&edge_actual) == type_key(actual) {
            return Some(path.clone());
        }

        if let Some(found) = find_canonical_virtual_owner_path(
            graph, complete, edge.base, target, actual, child_path,
        ) {
            return Some(found);
        }
    }

    None
}

fn virtual_base_views(graph: &Graph, complete: usize) -> Vec<AncestorView> {
    let mut views = Vec::new();
    let mut seen = HashSet::new();
    collect_virtual_base_views(graph, complete, complete, Vec::new(), &mut seen, &mut views);
    views
}

fn collect_virtual_base_views(
    graph: &Graph,
    complete: usize,
    current: usize,
    path: Vec<usize>,
    seen: &mut HashSet<String>,
    views: &mut Vec<AncestorView>,
) {
    for edge in &graph.base_edges[current] {
        let mut next_path = path.clone();
        next_path.push(edge.base);
        if edge.is_virtual {
            let actual = ancestor_type_for_path(graph, complete, &next_path);
            if seen.insert(cast_target_key(edge.base, &actual)) {
                views.push(AncestorView {
                    class_index: edge.base,
                    actual,
                    path: next_path.clone(),
                });
            }
        }
        collect_virtual_base_views(graph, complete, edge.base, next_path, seen, views);
    }
}

fn constructor_base_call_matches(
    base_call: &crate::ast::ConstructorBaseCall,
    actual: &Type,
) -> bool {
    !type_has_explicit_generics(&base_call.ty) || type_key(&base_call.ty) == type_key(actual)
}

fn type_has_explicit_generics(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.first() else {
        return false;
    };
    matches!(
        &segment.arguments,
        syn::PathArguments::AngleBracketed(arguments) if !arguments.args.is_empty()
    )
}
