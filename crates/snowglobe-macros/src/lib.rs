use humantime::Duration;
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn scene(args: TokenStream, item: TokenStream) -> TokenStream {
    let args: SceneArgs = match syn::parse(args) {
        Ok(args) => args,
        Err(error) => return error.to_compile_error().into(),
    };
    let func = parse_macro_input!(item as ItemFn);
    let name_ident = &func.sig.ident;

    let start_time = quote_option(args.start_time.map(|arg| arg.0));
    let simulation_duration = quote_option(args.simulation_duration);
    let tick_duration = quote_option(args.tick_duration);
    let min_message_latency = quote_option(args.min_message_latency);
    let max_message_latency = quote_option(args.max_message_latency);
    let fail_rate = quote_option(args.fail_rate);
    let repair_rate = quote_option(args.repair_rate);
    let tcp_capacity = quote_option(args.tcp_capacity);
    let udp_capacity = quote_option(args.udp_capacity);
    let fs_corruption_probability = quote_option(args.fs_corruption_probability);

    let expanded = quote! {
        #func

        const _: () = {
            use ::snowglobe::__private::*;

            #[linkme::distributed_slice(SCENES)]
            #[linkme(crate = linkme)]
            static SCENE: Scene = Scene {
                module: module_path!(),
                name: stringify!(#name_ident),
                func: #name_ident,
                config: SceneConfig {
                    start_time: #start_time,
                    simulation_duration: #simulation_duration,
                    tick_duration: #tick_duration,
                    min_message_latency: #min_message_latency,
                    max_message_latency: #max_message_latency,
                    fail_rate: #fail_rate,
                    repair_rate: #repair_rate,
                    tcp_capacity: #tcp_capacity,
                    udp_capacity: #udp_capacity,
                    fs_corruption_probability: #fs_corruption_probability,
                },
            };
        };
    };

    expanded.into()
}

fn quote_option<T: quote::ToTokens>(opt: Option<T>) -> proc_macro2::TokenStream {
    match opt {
        Some(v) => quote! { ::std::option::Option::Some(#v) },
        None => quote! { ::std::option::Option::None },
    }
}

#[derive(Debug, darling::FromMeta)]
#[darling(derive_syn_parse)]
struct SceneArgs {
    start_time: Option<StartTimeArg>,
    simulation_duration: Option<DurationArg>,
    tick_duration: Option<DurationArg>,
    min_message_latency: Option<DurationArg>,
    max_message_latency: Option<DurationArg>,
    fail_rate: Option<f64>,
    repair_rate: Option<f64>,
    tcp_capacity: Option<usize>,
    udp_capacity: Option<usize>,
    fs_corruption_probability: Option<f64>,
}

#[derive(Debug)]
struct StartTimeArg(DurationArg);

impl darling::FromMeta for StartTimeArg {
    fn from_string(s: &str) -> darling::Result<Self> {
        let start_time = humantime::parse_rfc3339(s).map_err(darling::Error::custom)?;
        let since_unix_epoch = start_time
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(darling::Error::custom)?;
        Ok(Self(DurationArg(since_unix_epoch.into())))
    }
}

#[derive(Debug)]
struct DurationArg(Duration);

impl darling::FromMeta for DurationArg {
    fn from_string(s: &str) -> darling::Result<Self> {
        s.parse::<Duration>()
            .map(DurationArg)
            .map_err(darling::Error::custom)
    }
}

impl quote::ToTokens for DurationArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let secs = self.0.as_secs();
        let nanos = self.0.subsec_nanos();
        tokens.extend(quote! {
            ::std::time::Duration::new(#secs, #nanos)
        });
    }
}
