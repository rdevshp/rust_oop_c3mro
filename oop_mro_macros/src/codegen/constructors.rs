use super::*;

pub(super) fn generate_constructor_hook(
    graph: &Graph,
    index: usize,
    class: &ClassDef,
) -> TokenStream2 {
    let constructor = class_constructor(class);
    let inputs = constructor
        .map(|constructor| constructor.inputs.as_slice())
        .unwrap_or(&[]);
    let virtual_base_calls = generate_constructor_virtual_base_calls(graph, index, constructor);
    let base_calls = generate_constructor_base_calls(graph, index, constructor);
    let body = constructor.map(|constructor| &constructor.body);
    let args = constructor.map(constructor_arg_idents).unwrap_or_default();

    quote! {
        fn __oop_ctor_complete(&mut self #(, #inputs)*) {
            #virtual_base_calls
            self.__oop_ctor(#(#args),*);
        }

        fn __oop_ctor(&mut self #(, #inputs)*) {
            #base_calls
            #body
        }
    }
}

pub(super) fn generate_constructor_new(
    graph: &Graph,
    index: usize,
    class: &ClassDef,
) -> TokenStream2 {
    if class.is_abstract {
        return quote! {};
    }

    let Some(constructor) = class_constructor(class) else {
        return quote! {};
    };

    let attrs = &constructor.attrs;
    let vis = public_if_inherited(&constructor.vis);
    let inputs = &constructor.inputs;
    let args = constructor_arg_idents(constructor);
    let trait_name = default_base_trait_ident(&graph.names[index]);

    quote! {
        #(#attrs)*
        #vis fn new(#(#inputs),*) -> Self {
            let mut __oop_value = <Self as #trait_name>::__oop_default_base();
            __oop_value.__oop_ctor_complete(#(#args),*);
            __oop_value
        }
    }
}

fn generate_constructor_virtual_base_calls(
    graph: &Graph,
    index: usize,
    constructor: Option<&ConstructorDef>,
) -> TokenStream2 {
    let Some(constructor) = constructor else {
        return quote! {};
    };

    let virtual_targets = virtual_base_views(graph, index);
    let calls = virtual_targets.into_iter().filter_map(|view| {
        let target = view.class_index;
        let base_name = &graph.names[target];
        constructor
            .base_calls
            .iter()
            .find(|base_call| {
                base_call.base == base_name.as_str()
                    && constructor_base_call_matches(base_call, &view.actual)
            })
            .map(|base_call| {
                let target_ref =
                    static_ref_expr_for_path(graph, index, &view.path, quote! { self }, true);
                let args = &base_call.args;
                quote! {
                    (#target_ref).__oop_ctor(#(#args),*);
                }
            })
    });

    quote! {
        #(#calls)*
    }
}

fn generate_constructor_base_calls(
    graph: &Graph,
    index: usize,
    constructor: Option<&ConstructorDef>,
) -> TokenStream2 {
    let calls = graph.base_edges[index]
        .iter()
        .filter(|edge| !edge.is_virtual)
        .filter_map(|edge| {
            let base = edge.base;
            let base_name = &graph.names[base];
            let accessor = format_ident!("__oop_as_mut_{}", base_name);
            constructor.and_then(|constructor| {
                constructor
                    .base_calls
                    .iter()
                    .find(|base_call| base_call.base == base_name.as_str())
                    .map(|base_call| {
                        let args = &base_call.args;
                        quote! {
                            self.#accessor().__oop_ctor(#(#args),*);
                        }
                    })
            })
        });

    quote! {
        #(#calls)*
    }
}

pub(super) fn generate_default_base_impl(
    graph: &Graph,
    index: usize,
    class: &ClassDef,
) -> TokenStream2 {
    let name = &class.name;
    let (impl_generics, ty_generics, where_clause) = class.generics.split_for_impl();
    let trait_name = default_base_trait_ident(&graph.names[index]);
    let vtable_initializer = needs_runtime_metadata(graph, index).then(|| {
        let vtable = vtable_factory_ident(
            graph,
            index,
            &VtableSlot {
                ancestor: index,
                path: Vec::new(),
            },
        );
        quote! {
            __oop_vtable: #vtable(),
        }
    });
    let base_initializers = graph.base_edges[index].iter().map(|edge| {
        let base = edge.base;
        if edge.is_virtual {
            let field = virtual_base_field_ident(&graph.names[base]);
            quote! {
                #field: ::oop_mro::VirtualBaseSlot::uninit()
            }
        } else {
            let field = base_field_ident(&graph.names[base]);
            let base_ty = ancestor_type(graph, index, base);
            let base_trait = default_base_trait_ident(&graph.names[base]);
            quote! {
                #field: <#base_ty as #base_trait>::__oop_default_subobject()
            }
        }
    });
    let field_initializers = class.items.iter().filter_map(|item| match item {
        ClassItem::Field(field) => {
            let ident = &field.ident;
            let value = field
                .initializer
                .as_ref()
                .map(ToTokens::to_token_stream)
                .unwrap_or_else(|| quote! { ::core::default::Default::default() });
            Some(quote! {
                #ident: #value
            })
        }
        ClassItem::Method(_) => None,
        ClassItem::Constructor(_) => None,
        ClassItem::AssociatedConst(_) => None,
        ClassItem::StaticField(_) => None,
        ClassItem::UnsupportedAssociatedType(_) => None,
    });

    quote! {
        trait #trait_name {
            fn __oop_default_base() -> Self;
            fn __oop_default_subobject() -> Self;
        }

        impl #impl_generics #trait_name for #name #ty_generics #where_clause {
            fn __oop_default_base() -> Self {
                let mut value = <Self as #trait_name>::__oop_default_subobject();
                value.__oop_init_virtual_bases();
                value.__oop_init_vtables();
                value
            }

            fn __oop_default_subobject() -> Self {
                let mut value = Self {
                    #vtable_initializer
                    #(#base_initializers,)*
                    #(#field_initializers,)*
                };
                value
            }
        }
    }
}

pub(super) fn generate_default_impl(graph: &Graph, index: usize, class: &ClassDef) -> TokenStream2 {
    if class.is_abstract {
        return quote! {};
    }

    let name = &class.name;
    let (impl_generics, ty_generics, where_clause) = class.generics.split_for_impl();
    let trait_name = default_base_trait_ident(&graph.names[index]);
    quote! {
        impl #impl_generics ::core::default::Default for #name #ty_generics #where_clause {
            fn default() -> Self {
                <Self as #trait_name>::__oop_default_base()
            }
        }
    }
}

pub(super) fn generate_vtable_init(graph: &Graph, index: usize) -> TokenStream2 {
    let virtual_base_initializers = virtual_base_views(graph, index).into_iter().map(|view| {
        let target = view.class_index;
        let slot =
            virtual_base_slot_expr(graph, index, target, &view.actual, quote! { self }, true);
        let base_ty = &view.actual;
        let base_trait = default_base_trait_ident(&graph.names[target]);
        quote! {
            (#slot).init(<#base_ty as #base_trait>::__oop_default_subobject());
        }
    });
    let assignments = vtable_slots(graph, index).into_iter().map(|slot| {
        let vtable = vtable_factory_ident(graph, index, &slot);
        let place = place_for_path(graph, index, &slot.path);

        quote! {
            #place.__oop_vtable = #vtable();
        }
    });

    quote! {
        fn __oop_init_virtual_bases(&mut self) {
            #(#virtual_base_initializers)*
        }

        fn __oop_init_vtables(&mut self) {
            #(#assignments)*
        }
    }
}

fn place_for_path(graph: &Graph, complete: usize, path: &[usize]) -> TokenStream2 {
    let mut tokens = quote! { self };
    let mut current = complete;

    for &base in path {
        if edge_is_virtual(graph, current, base) {
            let field = virtual_base_field_ident(&graph.names[base]);
            tokens = quote! { unsafe { (#tokens).#field.assume_init_mut() } };
        } else {
            let field = base_field_ident(&graph.names[base]);
            tokens = quote! { (#tokens).#field };
        }
        current = base;
    }

    tokens
}
