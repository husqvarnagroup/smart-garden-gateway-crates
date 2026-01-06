//! Create a task that can be talked to via an async API.
//!
//! This crate implements Strategy 2 from the [tokio tutorial](https://tokio.rs/tokio/tutorial/shared-state#strategies).
//! But instead of hand-writing that boilerplate for every function, this crate
//! provides a macro that does that for you.

use proc_macro_error2::abort;
use quote::quote;
use syn::spanned::Spanned as _;

/// parsed `tokio_task_rpc` proc macro arguments
#[derive(Default)]
struct MacroArgs {
    handle_name: Option<String>,
    enum_visibility: Option<proc_macro2::TokenStream>,
    handle_visibility: Option<proc_macro2::TokenStream>,
    handlefns_visibility: Option<proc_macro2::TokenStream>,
    receiver_visibility: Option<proc_macro2::TokenStream>,
}

/// parsed `tokio_task_rpc` method arguments
struct MethodArgs {
    wait: bool,
}

fn parse_visibility_str(lit: &syn::Lit, destination: &mut Option<proc_macro2::TokenStream>) {
    let value = match lit {
        syn::Lit::Str(s) => s.value(),
        _ => abort!(lit.span(), "unsupported attribute value type"),
    };

    if destination.is_some() {
        abort!(lit.span(), "duplicate attribute");
    }

    *destination = Some(match value.as_str() {
        "pub" => quote! {pub},
        "pub(super)" => quote! {pub(super)},
        other => abort!("unsupported visibility: {}", other),
    });
}

/// parse the channel names which may be present in the invocation arguments of
/// this crate's main macro.
fn parse_args(synargs: &[syn::NestedMeta]) -> MacroArgs {
    let mut args = MacroArgs::default();

    for arg in synargs {
        let meta = match arg {
            syn::NestedMeta::Meta(m) => m,
            _ => abort!(arg.span(), "expected meta arg"),
        };
        let namevalue = match meta {
            syn::Meta::NameValue(nv) => nv,
            _ => abort!(meta.span(), "expected namevalue arg"),
        };

        let ident = namevalue
            .path
            .get_ident()
            .unwrap_or_else(|| abort!(namevalue.path.span(), "missing ident"));
        match ident.to_string().as_str() {
            "handle_name" => {
                let value = match &namevalue.lit {
                    syn::Lit::Str(s) => s.value(),
                    _ => abort!(namevalue.lit.span(), "unsupported attribute value type"),
                };

                if args.handle_name.is_some() {
                    abort!(ident.span(), "duplicate handle_name");
                }

                args.handle_name = Some(value);
            }
            "enum_visibility" => {
                parse_visibility_str(&namevalue.lit, &mut args.enum_visibility);
            }
            "handle_visibility" => {
                parse_visibility_str(&namevalue.lit, &mut args.handle_visibility);
            }
            "handlefns_visibility" => {
                parse_visibility_str(&namevalue.lit, &mut args.handlefns_visibility);
            }
            "receiver_visibility" => {
                parse_visibility_str(&namevalue.lit, &mut args.receiver_visibility);
            }
            _ => abort!(ident.span(), "unsupported attribute"),
        }
    }

    args
}

/// this parses the `tokio_task_rpc` arguments and removes them from the
/// token tree so you won't get a compiler error about unsupported macros.
fn extract_method_args(method: &mut syn::ImplItemMethod) -> MethodArgs {
    let mut args = MethodArgs { wait: true };

    let mut i = 0;
    while i < method.attrs.len() {
        let attr = &method.attrs[i];
        if !attr.path.is_ident("tokio_task_rpc") {
            i += 1;
            continue;
        }

        let attr = method.attrs.remove(i);
        let meta = attr.parse_meta().unwrap();
        let metalist = match &meta {
            syn::Meta::List(l) => l,
            other => abort!(meta.span(), "expected MetaList, got: {:#?}", other),
        };

        for nestedmeta in &metalist.nested {
            match &nestedmeta {
                syn::NestedMeta::Meta(syn::Meta::Path(path)) if path.is_ident("nowait") => {
                    args.wait = false;
                }
                _ => abort!(nestedmeta.span(), "unsupported key"),
            };
        }
    }

    args
}

/// This crate's main macro
///
/// It generates the following types:
/// - `{structname}Receiver` which can receive messages from an mpsc channel
/// - `{structname}Handle` which implements all RPC methods. Internally this uses
///   tokio channels to execute the actual implementations inside the receiver
///   task. The handle can be cloned to do RPC calls form multiple tasks at the
///   same time.
/// - `{structname}Request{channelname}` internal enums which hold arguments and
///   an optional result channel for all methods.
///
/// it also implements these methods in `{structname}`
/// - `handle_requests` this processes requests on all channels until all handles
///   were dropped.
/// - `handle_requests_idlefn` same as `handle_requests` but runs the provided
///   callback before waiting for new requests if the queue is empty.
/// - `handle_one_{channelname}_request` This handles a single request on
///   {channelname} and returns. If any of the channel lost all it's senders,
///   this function will return an error.
///
/// The code generated by these macros expects the following fields in
/// `{structname}`:
/// - `receiver` An instance of `{structname}Receiver`
///
/// It also expects an `Error` type:
/// - it needs a `From` implementation for `tokio::sync::oneshot::error::RecvError`
/// - it needs a `MpscReceiverClosed` variant
///
/// supported `tokio_task_rpc::interface` attributes:
/// - `extra_channels` a string with a comma-separated list of channels.
///   The channel `default` does always exist and must not be part of that list.
///
/// You can also add a `tokio_task_rpc` to each method.
/// It supports the following keys:
/// - `channel` Which channel to use for executing this method.
///   Default value: `default`.
/// - `nowait` if this is present, calling this method on the handle will just
///   trigger the execution but not wait for it to complete.
///   The return value must be `Result<(), Error>` in that case.
///
/// # Examples
/// - minimal working code:
/// ```
/// use anyhow::Error;
///
/// struct Device {
///     receiver: DeviceReceiver,
/// }
///
/// #[tokio_task_rpc::interface]
/// impl Device {
///     async fn doit(&self) -> anyhow::Result<()> {
///         Ok(())
///     }
/// }
/// ```
/// - an example using all features:
/// ```
/// # use anyhow::Error;
/// #
/// # struct Device {
/// #     receiver: DeviceReceiver,
/// # }
/// #
/// #[tokio_task_rpc::interface]
/// impl Device {
///     async fn doit(&self) -> anyhow::Result<u32> {
///         Ok(42)
///     }
///
///     #[tokio_task_rpc()]
///     async fn doit2(&self) -> anyhow::Result<()> {
///         Ok(())
///     }
///
///     #[tokio_task_rpc(nowait)]
///     async fn doit3(&self) -> anyhow::Result<()> {
///         Ok(())
///     }
/// }
/// ```
#[proc_macro_error2::proc_macro_error]
#[proc_macro_attribute]
pub fn interface(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let args = parse_args(&args);

    let mut root_impl = syn::parse_macro_input!(input as syn::ItemImpl);
    let root_span = root_impl.span();
    let root_ident = match &*root_impl.self_ty {
        syn::Type::Path(path) => {
            &path
                .path
                .segments
                .last()
                .unwrap_or_else(|| abort!(root_impl, "no segments in impl self type path"))
                .ident
        }
        _ => abort!(root_impl, "self type of the impl must be a Path"),
    };

    let enum_visibility = &args.enum_visibility;
    let handle_visibility = &args.handle_visibility;
    let handlefns_visibility = &args.handlefns_visibility;
    let receiver_visibility = &args.receiver_visibility;

    let enum_ident = proc_macro2::Ident::new(&format!("{root_ident}Request"), root_ident.span());
    let handle_ident = proc_macro2::Ident::new(
        &args.handle_name.unwrap_or(format!("{root_ident}Handle")),
        root_ident.span(),
    );
    let receiver_ident =
        proc_macro2::Ident::new(&format!("{root_ident}Receiver"), root_ident.span());
    let handleone_ret_ident =
        proc_macro2::Ident::new(&format!("{root_ident}HandleOneValueRef"), root_ident.span());
    let mut enum_variants = Vec::new();
    let mut handle_matches = Vec::new();
    let mut handle_functions = Vec::new();
    let mut receiver_functions = Vec::new();
    let mut extra_root_impls = Vec::new();

    for item in &mut root_impl.items {
        let method_orig = match item {
            syn::ImplItem::Method(m) => m,
            _ => continue,
        };
        let method_args = extract_method_args(method_orig);
        let mut method = method_orig.clone();
        let method_ident = &method.sig.ident;
        let method_ident_str = proc_macro2::Literal::string(&method_ident.to_string());
        let method_returntype = match &method.sig.output {
            syn::ReturnType::Type(_, t) => t,
            _ => abort!(method_ident.span(), "method needs an explicit return type"),
        };
        let method_await = if method.sig.asyncness.is_some() {
            quote! { .await }
        } else {
            quote! {}
        };

        let rpc_returntype: proc_macro::TokenStream = if method_args.wait {
            quote! {Result<#method_returntype, ::tokio_task_rpc_util::Error>}
        } else {
            quote! {Result<(), ::tokio_task_rpc_util::Error>}
        }
        .into();
        let rpc_returntype = syn::parse_macro_input!(rpc_returntype as syn::Type);

        let mut variant_args = Vec::new();
        let mut variant_initializers = Vec::new();
        let mut variant_arg_names = Vec::new();
        let mut predicate_types = Vec::new();
        for fnarg in &method.sig.inputs {
            let pattype = match fnarg {
                syn::FnArg::Typed(o) => o,
                _ => continue,
            };
            let patident = match &*pattype.pat {
                syn::Pat::Ident(i) => i,
                _ => abort!(pattype, "typed args must have identifiers as `pat`"),
            };
            let ident = &patident.ident;
            let ty = &pattype.ty;

            let arg_ident = patident.ident.to_string();
            let arg_ident = proc_macro2::Ident::new(
                &format!(
                    "arg_{}",
                    if let Some(s) = arg_ident.strip_prefix('_') {
                        s
                    } else {
                        &arg_ident
                    }
                ),
                patident.ident.span(),
            );
            variant_args.push(quote! {
                #arg_ident: #ty,
            });

            variant_initializers.push(quote! {
                #arg_ident: #ident,
            });

            variant_arg_names.push(arg_ident);
            predicate_types.push(ty);
        }

        let response_field = if method_args.wait {
            quote! { response: ::tokio::sync::oneshot::Sender<#method_returntype>, }
        } else {
            quote! {}
        };
        enum_variants.push(quote! {
            #method_ident {
                #(#variant_args)*
                #response_field
            },
        });

        let response_init = if method_args.wait {
            quote! { response: resp_tx, }
        } else {
            quote! {}
        };
        let response_wait = if method_args.wait {
            // technically the receiver could have closed while or after
            // processing the call and we'd want to be able to handle that
            // differently. Unfortunately it's not that easy to find the reason
            // and in reality this shouldn't happen unless the thread panicked.
            quote! { resp_rx.await.map_err(|_| tokio_task_rpc_util::Error::ReceiverClosed) }
        } else {
            quote! { Ok(()) }
        };
        let method_block = proc_macro::TokenStream::from(quote! {{
            let (resp_tx, resp_rx) = ::tokio::sync::oneshot::channel::<#method_returntype>();

            self.tx
                .send((std::time::Instant::now(), #enum_ident::#method_ident {
                    #(#variant_initializers)*
                    #response_init
                })).map_err(|e| match e {
                    rpc_mpsc::Error::ReceiverClosed => tokio_task_rpc_util::Error::ReceiverClosed,
                })?;

            #response_wait
        }});
        method.block = syn::parse_macro_input!(method_block as syn::Block);
        method.sig.output =
            syn::ReturnType::Type(syn::Token![->](root_span), Box::new(rpc_returntype));

        if !method_args.wait {
            method.sig.asyncness = None;
        }

        handle_functions.push(quote! {
            #[allow(unused_mut)]
            #[automatically_derived]
            #method
        });

        let response_send = if method_args.wait {
            quote! {
                if let Err(_) = response.send(res) {
                    ::tracing::error!("Can't send result for method `{}`", #method_ident_str);
                }
            }
        } else {
            quote! {
                if let Err(e) = res {
                    ::tracing::error!("Method `{}` failed: {:?}", #method_ident_str, e);
                }
            }
        };
        let variant_response_arg = if method_args.wait {
            quote! { response, }
        } else {
            quote! {}
        };
        handle_matches.push(quote! {
            #enum_ident::#method_ident{ #variant_response_arg #(#variant_arg_names,)* } => {
                let res = self.#method_ident(#(#variant_arg_names,)*)#method_await;
                #response_send
            },
        });

        let ident_fn =
            proc_macro2::Ident::new(&format!("handle_one_{method_ident}_request"), root_span);
        let tokens = proc_macro::TokenStream::from(quote! {
            #[automatically_derived]
            #[allow(clippy::unused_unit)]
            #handlefns_visibility async fn #ident_fn<P>(&mut self, mut predicate: P) -> Option<impl #handleone_ret_ident<'_, (#(&#predicate_types,)*)>>
            where
                P: FnMut(std::time::Instant, #(&#predicate_types,)*) -> bool,
            {
                let value_ref = self.receiver.rx.wait_for(|(instant, item)| match item {
                    #enum_ident::#method_ident{ #variant_response_arg #(#variant_arg_names,)* } => {
                        predicate(*instant, #(#variant_arg_names,)*)
                    }
                    _ => false,
                }).await?;

                struct Ret<'a> {
                    value_ref: ::rpc_mpsc::ValueRef<'a, (std::time::Instant, #enum_ident)>,
                }

                impl<'a> #handleone_ret_ident<'a, (#(&'a #predicate_types,)*)> for Ret<'a> {
                    fn args(&'a self) -> (#(&'a #predicate_types,)*) {
                        use std::ops::Deref;

                        match &self.value_ref.deref().1 {
                            #enum_ident::#method_ident{ #variant_response_arg #(#variant_arg_names,)* } => {
                                (#(&#variant_arg_names,)*)
                            },
                            _ => unreachable!(),
                        }
                    }
                }

                Some(Ret {value_ref})
            }
        });
        let method = syn::parse_macro_input!(tokens as syn::ImplItemMethod);
        extra_root_impls.push(syn::ImplItem::Method(method));

        // if the caller waits for an answer, removing the request without
        // processing it is not an easy thing to do. Since we currently don't
        // need that, let's just not implement it for now.
        if !method_args.wait {
            let ident_fn =
                proc_macro2::Ident::new(&format!("remove_one_{method_ident}_request"), root_span);
            let tokens = proc_macro::TokenStream::from(quote! {
                #[automatically_derived]
                #[allow(clippy::unused_unit)]
                pub async fn #ident_fn<P>(&mut self, mut predicate: P) -> Option<(#(#predicate_types,)*)>
                where
                    P: FnMut(std::time::Instant, #(&#predicate_types,)*) -> bool,
                {
                    let item = self.rx.wait_for_remove(|(instant, item)| match item {
                        #enum_ident::#method_ident{ #variant_response_arg #(#variant_arg_names,)* } => {
                            predicate(*instant, #(#variant_arg_names,)*)
                        }
                        _ => false,
                    }).await?;

                    match item.1 {
                        #enum_ident::#method_ident{ #variant_response_arg #(#variant_arg_names,)* } => {
                            Some((#(#variant_arg_names,)*))
                        },
                        _ => unreachable!(),
                    }
                }
            });
            let method = syn::parse_macro_input!(tokens as syn::ImplItemMethod);
            receiver_functions.push(syn::ImplItem::Method(method));
        }
    }

    let tokens = proc_macro::TokenStream::from(quote! {
        #[automatically_derived]
        #handlefns_visibility async fn handle_requests(&mut self) {
            use ::futures_util::FutureExt;

            while self.receiver.is_enabled() {
                match self.receiver.rx.recv().await {
                    Some((_instant, cmd)) => match cmd {
                        #(#handle_matches)*
                    },
                    None => return,
                }
            }
        }
    });
    let method = syn::parse_macro_input!(tokens as syn::ImplItemMethod);
    root_impl.items.push(syn::ImplItem::Method(method));

    let tokens = proc_macro::TokenStream::from(quote! {
        #[automatically_derived]
        #handlefns_visibility async fn handle_requests_idlefn<F>(&mut self, on_idle: F)
        where
            for<'a> F: ::tokio_task_rpc_util::FnHelper<'a, Self, Result<bool, Error>>,
        {
            use ::futures_util::FutureExt;

            while self.receiver.is_enabled() {
                if self.receiver.rx.is_empty() {
                    match on_idle.call(self).await {
                        Err(e) => ::tracing::error!("Idle function failed: {:?}", e),
                        Ok(call_recv) => if !call_recv {
                            continue;
                        }
                    }
                }

                match self.receiver.rx.recv().await {
                    Some((_instant, cmd)) => match cmd {
                        #(#handle_matches)*
                    },
                    None => return,
                }
            }
        }
    });
    let method = syn::parse_macro_input!(tokens as syn::ImplItemMethod);
    root_impl.items.push(syn::ImplItem::Method(method));

    for item in extra_root_impls {
        root_impl.items.push(item);
    }

    proc_macro::TokenStream::from(quote! {
        #[automatically_derived]
        #[allow(non_camel_case_types)]
        #[derive(Debug)]
        #enum_visibility enum #enum_ident {
            #(#enum_variants)*
        }

        #[derive(Clone)]
        #[automatically_derived]
        #handle_visibility struct #handle_ident {
            tx: ::rpc_mpsc::Sender<(std::time::Instant, #enum_ident)>,
        }

        #[automatically_derived]
        #receiver_visibility struct #receiver_ident {
            pub rx: ::rpc_mpsc::Receiver<(std::time::Instant, #enum_ident)>,
            enabled: bool
        }

        impl #receiver_ident {
            pub fn is_enabled(&self) -> bool {
                self.enabled
            }

            pub fn set_enabled(&mut self, enabled: bool) {
                self.enabled = enabled;
            }

            #(#receiver_functions)*
        }

        #[automatically_derived]
        impl #handle_ident {
            fn new() -> (Self, #receiver_ident) {
                let (tx, rx) = ::rpc_mpsc::channel();

                let handle = Self {
                    tx,
                };
                let receiver = #receiver_ident {
                    rx,
                    enabled: true
                };

                (handle, receiver)
            }

            #[automatically_derived]
            pub fn is_closed(&self) -> bool {
                self.tx.is_closed()
            }

            #[automatically_derived]
            pub fn clone_uncounted(&self) -> Self {
                Self {
                    tx: self.tx.clone_uncounted(),
                }
            }

            #(#handle_functions)*
        }

        #[automatically_derived]
        #handlefns_visibility trait #handleone_ret_ident<'a, T> {
            fn args(&'a self) -> T;
        }

        #root_impl
    })
}
