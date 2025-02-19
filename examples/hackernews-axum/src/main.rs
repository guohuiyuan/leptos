use cfg_if::cfg_if;
use leptos::*;

// boilerplate to run in different modes
cfg_if! {
if #[cfg(feature = "ssr")] {
    use axum::{
        routing::{get},
        Router,
        error_handling::HandleError,
    };
    use http::StatusCode;
    use std::net::SocketAddr;
    use tower_http::services::ServeDir;
    use std::env;

    #[tokio::main]
    async fn main() {
        use leptos_hackernews_axum::*;
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

        log::debug!("serving at {addr}");

        simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");

        // These are Tower Services that will serve files from the static and pkg repos.
        // HandleError is needed as Axum requires services to implement Infallible Errors
        // because all Errors are converted into Responses
        let static_service = HandleError::new( ServeDir::new("./static"), handle_file_error);
        let pkg_service =HandleError::new( ServeDir::new("./pkg"), handle_file_error);

        /// Convert the Errors from ServeDir to a type that implements IntoResponse
        async fn handle_file_error(err: std::io::Error) -> (StatusCode, String) {
            (
                StatusCode::NOT_FOUND,
                format!("File Not Found: {}", err),
            )
        }

        let render_options: RenderOptions = RenderOptions::builder().pkg_path("/pkg/leptos_hackernews_axum").socket_address(addr).reload_port(3001).environment(&env::var("RUST_ENV")).build();
        render_options.write_to_file();
        // build our application with a route
        let app = Router::new()
        // `GET /` goes to `root`
        .nest_service("/pkg", pkg_service)
        .nest_service("/static", static_service)
        .fallback(leptos_axum::render_app_to_stream(render_options, |cx| view! { cx, <App/> }));

        // run our app with hyper
        // `axum::Server` is a re-export of `hyper::Server`
        log!("listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

    // client-only stuff for Trunk
    else {
        use leptos_hackernews_axum::*;

        pub fn main() {
            console_error_panic_hook::set_once();
            _ = console_log::init_with_level(log::Level::Debug);
            console_error_panic_hook::set_once();
            mount_to_body(|cx| {
                view! { cx, <App/> }
            });
        }
    }
}
