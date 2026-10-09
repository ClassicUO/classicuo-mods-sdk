//! `#[system]`: makes a cuo-mod-sdk system / observer fn an export of the mod's world.
//!
//! A mod is a component of its own world with one export per system, named like the
//! system (`snap_input` -> `export snap-input: func(...)`), its parameters the system's
//! wire parameters in order. For each marked fn this emits, next to the fn (unchanged):
//!
//! - a `wit_bindgen::generate!` of a one-export world. Every generate! leaves a
//!   component-type section; wit-component merges them all (plus the SDK's own
//!   `cuo:modding/mod` world with `setup`) into the mod's world when it links the
//!   component.
//! - the `Guest` impl of that export: it hands the typed handles to the SDK, which runs
//!   the fn through the system the App registered under that name at setup. The core
//!   export itself is emitted here (standard `cm32p2||` name mangling), not by
//!   wit-bindgen's `export!`, whose bare names clash with libc symbols.
//! - a compile-time check that each parameter's wire kind (read from its type's name,
//!   which is all a macro sees) matches the type's `SystemParam`.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Error, FnArg, GenericArgument, ItemFn, PathArguments, ReturnType, Type};

/// The tinyecs:modding package the one-export worlds `use`.
const ECS_WIT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../wit/deps/tinyecs-mod");

#[derive(Clone, Copy, PartialEq)]
enum Wire {
    None,
    Commands,
    Query,
    Res,
    Events,
}

impl Wire {
    fn wit(self) -> &'static str {
        match self {
            Wire::None => unreachable!(),
            Wire::Commands => "commands",
            Wire::Query => "query",
            Wire::Res => "res",
            Wire::Events => "events",
        }
    }
    fn variant(self) -> proc_macro2::Ident {
        proc_macro2::Ident::new(match self {
            Wire::None => "None",
            Wire::Commands => "Commands",
            Wire::Query => "Query",
            Wire::Res => "Res",
            Wire::Events => "Events",
        }, Span::call_site())
    }
}

enum Kind {
    System,
    /// First param `On<K, T>`.
    Observer,
    /// First param `Packet`, returns `Verdict`.
    Packet,
}

fn last_ident(ty: &Type) -> Option<&syn::PathSegment> {
    match ty {
        Type::Path(p) if p.qself.is_none() => p.path.segments.last(),
        Type::Paren(p) => last_ident(&p.elem),
        Type::Group(g) => last_ident(&g.elem),
        _ => None,
    }
}

fn classify(ty: &Type) -> syn::Result<Wire> {
    let unknown = || {
        Error::new(
            ty.span(),
            "#[system] can't tell this parameter's kind: it reads the type's name, so write \
             Commands / Query<..> / Res<..> / ResMut<..> / EventReader<..> / Local<..> / \
             Option<..> of those (or helpers::Items) — not an alias of them",
        )
    };
    let seg = last_ident(ty).ok_or_else(unknown)?;
    Ok(match seg.ident.to_string().as_str() {
        "Commands" => Wire::Commands,
        "Query" | "Items" => Wire::Query,
        "Res" | "ResMut" => Wire::Res,
        "EventReader" => Wire::Events,
        "Local" => Wire::None,
        "Option" => {
            let PathArguments::AngleBracketed(args) = &seg.arguments else { return Err(unknown()) };
            let Some(GenericArgument::Type(inner)) = args.args.first() else { return Err(unknown()) };
            classify(inner)?
        }
        _ => return Err(unknown()),
    })
}

/// `snap_input` -> `snap-input`, checked to be a WIT identifier.
fn kebab(ident: &syn::Ident) -> syn::Result<String> {
    let name = ident.to_string();
    let name = name.strip_prefix("r#").unwrap_or(&name).to_string();
    let words: Vec<&str> = name.split('_').collect();
    let ok = words.iter().all(|w| {
        !w.is_empty()
            && w.chars().next().is_some_and(|c| c.is_ascii_lowercase())
            && w.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    });
    if !ok {
        return Err(Error::new(
            ident.span(),
            "a system's name becomes its WIT export name (snake_case -> kebab-case): use \
             lowercase words that each start with a letter, joined by single underscores",
        ));
    }
    if name == "setup" {
        return Err(Error::new(ident.span(), "`setup` is the mod's setup export: name the system differently"));
    }
    Ok(words.join("-"))
}

/// Marks a system or observer fn: it becomes an export of the mod's world, named like
/// the fn in kebab-case (`snap_input` -> `snap-input`). Every fn you hand to
/// `add_systems` / `add_observer` / `add_packet_observer` needs it.
///
/// ```ignore
/// #[system]
/// fn low_hp(player: Query<&Hits, With<Player>>, mut cmds: Commands) { ... }
/// ```
#[proc_macro_attribute]
pub fn system(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return Error::new(Span::call_site(), "#[system] takes no arguments").to_compile_error().into();
    }
    let func = parse_macro_input!(item as ItemFn);
    match expand(&func) {
        Ok(glue) => quote!(#func #glue).into(),
        Err(e) => {
            let e = e.to_compile_error();
            quote!(#func #e).into()
        }
    }
}

fn expand(func: &ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    let sig = &func.sig;
    if !sig.generics.params.is_empty() {
        return Err(Error::new(sig.generics.span(), "a system can't be generic: it is one export of the mod"));
    }
    let name = kebab(&sig.ident)?;
    let types: Vec<&Type> = sig
        .inputs
        .iter()
        .map(|a| match a {
            FnArg::Typed(t) => Ok(&*t.ty),
            FnArg::Receiver(r) => Err(Error::new(r.span(), "a system is a free fn, not a method")),
        })
        .collect::<syn::Result<_>>()?;

    let first = types.first().and_then(|t| last_ident(t)).map(|s| s.ident.to_string());
    let kind = match first.as_deref() {
        Some("On") => Kind::Observer,
        Some("Packet") => Kind::Packet,
        _ => Kind::System,
    };
    let params = match kind {
        Kind::System => &types[..],
        _ => &types[1..],
    };
    if matches!(kind, Kind::Packet) && matches!(sig.output, ReturnType::Default) {
        return Err(Error::new(sig.span(), "a packet observer returns a Verdict"));
    }

    let mut wit_params = Vec::new();
    let mut rust_params = Vec::new();
    let mut handles = Vec::new();
    let mut checks = Vec::new();
    for ty in params {
        let wire = classify(ty)?;
        let variant = wire.variant();
        checks.push(quote! {
            const _: () = ::core::assert!(
                ::cuo_mod_sdk::__private::wire_is::<#ty>(::cuo_mod_sdk::__private::Wire::#variant),
                "#[system]: this parameter's type name doesn't match its SystemParam kind",
            );
        });
        if wire == Wire::None {
            continue;
        }
        let p = format_ident!("p{}", wit_params.len());
        wit_params.push(format!("{p}: {}", wire.wit()));
        let wty = wire.variant();
        rust_params.push(quote!(#p: ::cuo_mod_sdk::__private::ecs::#wty));
        handles.push(quote!(::cuo_mod_sdk::__private::Param::#wty(#p)));
    }

    let (lead, ret) = match kind {
        Kind::System => ("", ""),
        Kind::Observer => ("trigger: trigger-data", ""),
        Kind::Packet => ("direction: packet-direction, packet: list<u8>", " -> verdict"),
    };
    let args: Vec<String> = std::iter::once(lead.to_string()).filter(|s| !s.is_empty()).chain(wit_params).collect();
    let wit = format!(
        "package cuo-mod-system:{name};\n\
         world {name} {{\n\
         use tinyecs:modding/ecs@0.1.0.{{commands, query, res, events, trigger-data, packet-direction, verdict}};\n\
         export %{name}: func({}){ret};\n\
         }}\n",
        args.join(", ")
    );
    let world = format!("cuo-mod-system:{name}/{name}");
    let method = format_ident!("{}", name.replace('-', "_"));
    let cabi = format_ident!("_export_{}_cabi", name.replace('-', "_"));
    let post = format_ident!("__post_return_{}", name.replace('-', "_"));
    // The core export under the standard name mangling ("cm32p2||<name>"), not the
    // bare legacy one wit-bindgen's `export!` uses: a root export named like a libc
    // symbol (`open`, `read`, `close`, ...) would clash with it at link time.
    let export_name = format!("cm32p2||{name}");
    let post_name = format!("cm32p2||{name}_post");
    let handle_args: Vec<_> = (0..handles.len()).map(|i| format_ident!("h{i}")).collect();
    let module = format_ident!("__cuo_system_{}", name.replace('-', "_"));

    let body = match kind {
        Kind::System => quote! {
            fn #method(#(#rust_params),*) {
                ::cuo_mod_sdk::__private::run_system(#name, ::std::vec![#(#handles),*])
            }
        },
        Kind::Observer => quote! {
            fn #method(trigger: ::cuo_mod_sdk::__private::ecs::TriggerData, #(#rust_params),*) {
                ::cuo_mod_sdk::__private::run_observer(#name, trigger, ::std::vec![#(#handles),*])
            }
        },
        Kind::Packet => quote! {
            fn #method(
                direction: ::cuo_mod_sdk::__private::ecs::PacketDirection,
                packet: ::std::vec::Vec<u8>,
                #(#rust_params),*
            ) -> ::cuo_mod_sdk::__private::ecs::Verdict {
                ::cuo_mod_sdk::__private::run_packet_observer(#name, direction, packet, ::std::vec![#(#handles),*])
            }
        },
    };

    // The flattened core signature of the export (lead args, then one i32 per handle);
    // must match what wit-bindgen generated for `#cabi`.
    let core_export = match kind {
        Kind::System => quote! {
            #[unsafe(export_name = #export_name)]
            unsafe extern "C" fn export(#(#handle_args: i32),*) {
                unsafe { self::#cabi::<System>(#(#handle_args),*) }
            }
        },
        Kind::Observer => quote! {
            #[unsafe(export_name = #export_name)]
            unsafe extern "C" fn export(entity: i64, value: *mut u8, len: usize, #(#handle_args: i32),*) {
                unsafe { self::#cabi::<System>(entity, value, len, #(#handle_args),*) }
            }
        },
        Kind::Packet => quote! {
            #[unsafe(export_name = #export_name)]
            unsafe extern "C" fn export(direction: i32, packet: *mut u8, len: usize, #(#handle_args: i32),*) -> *mut u8 {
                unsafe { self::#cabi::<System>(direction, packet, len, #(#handle_args),*) }
            }
            #[unsafe(export_name = #post_name)]
            unsafe extern "C" fn post_return(ret: *mut u8) {
                unsafe { self::#post::<System>(ret) }
            }
        },
    };

    Ok(quote! {
        #(#checks)*

        #[cfg(target_family = "wasm")]
        #[doc(hidden)]
        #[allow(dead_code, unused, clippy::all)]
        mod #module {
            ::cuo_mod_sdk::__private::wit_bindgen::generate!({
                inline: #wit,
                path: #ECS_WIT,
                world: #world,
                with: { "tinyecs:modding/ecs@0.1.0": ::cuo_mod_sdk::__private::ecs },
                runtime_path: "::cuo_mod_sdk::__private::wit_bindgen::rt",
            });

            struct System;

            impl Guest for System {
                #body
            }

            const _: () = {
                #core_export
            };
        }
    })
}
