use crate::app_state::AppState;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use fetch::fetch_data;
use image::{
    guess_format,
    io::{Limits, Reader},
    ImageFormat,
};
use request_context::RequestContext;
use reqwest::{header, StatusCode};
use std::{
    env,
    io::{BufWriter, Cursor},
    net::SocketAddr,
};

mod app_state;
mod fetch;
mod request_context;

#[tokio::main]
async fn main() {
    let app_state = AppState::new().unwrap();
    let app = Router::new().route("/", get(handle)).with_state(app_state);

    let addr = SocketAddr::new(
        "0.0.0.0".parse().unwrap(),
        env::var("PORT").unwrap_or("3080".into()).parse().unwrap(),
    );
    println!("Image Scaling Service starting on {:?}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[axum_macros::debug_handler]
async fn handle(
    Query(ctx): Query<RequestContext>,
    State(state): State<AppState>,
) -> Result<Response, (StatusCode, String)> {
    // check if domain is allowed
    if !state.is_allowed(&ctx.url) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Domain is not whitelisted".to_string(),
        ));
    }

    // get image format
    let requested_extension_string = &ctx.format.clone().unwrap_or("webp".into());
    let requested_format = ImageFormat::from_extension(requested_extension_string);
    let requested_mime_type = mime_guess::from_ext(&requested_extension_string).first();

    if requested_mime_type.is_none() || requested_format.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Format {} not supported", &requested_extension_string),
        ));
    }

    let requested_mime_type = requested_mime_type.unwrap();
    let requested_format = requested_format.unwrap();

    // redirect to source if the provided format is svg
    let provided_mime_type =
        mime_guess::from_path(&ctx.url).first_or("image/webp".parse().unwrap());
    if provided_mime_type.subtype().to_string() == "svg" {
        return Ok(Redirect::temporary(&ctx.url).into_response());
    }
    // download imgage
    let bytes = fetch_data(&ctx.url).await.map_err(|e| e.to_http_error())?;

    // determine image format
    let fetched_format = guess_format(&bytes).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to determine Image format".to_string(),
        )
    })?;

    // decode image
    let mut decoder = Reader::new(Cursor::new(&bytes));
    decoder.set_format(fetched_format);
    let mut limits = Limits::no_limits();
    // set allocator limit to 1gb
    limits.max_alloc = Some(1024 * 1024 * 1024);
    decoder.limits(limits);

    let image = decoder.decode();

    if image.is_err() {
        let err = image.unwrap_err();
        return match err {
            e => {
                // since not all decoders use the correct allocator limits, specifically the png one
                // as a temprorary workarround until proper limits are used here
                // https://github.com/image-rs/image-png/blob/2f53fc40b91a5e1d0ad1801b746c04a7fe1d8603/src/decoder/stream.rs#L751
                // we redirect to the original data instead of trying to convert it
                // TODO: remove temporary workarround
                println!("{:?} on {:?} for format {:?}", e, &ctx.url, &fetched_format);
                Ok(Redirect::temporary(&ctx.url).into_response())
            }
        };
    }

    let image = image.unwrap();

    // resize
    let image = image.resize(
        ctx.width.unwrap_or(image.width()),
        ctx.height.unwrap_or(image.height()),
        image::imageops::FilterType::Nearest,
    );

    // convert to rgba8 here since webp only supports that
    let image = image.to_rgba8();
    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    image.write_to(&mut buffer, requested_format).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error Converting to requested format".to_string(),
        )
    })?;
    let bytes: Vec<u8> = buffer
        .into_inner()
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error Converting to requested format".to_string(),
            )
        })?
        .into_inner();

    return Ok((
        [(header::CONTENT_TYPE, requested_mime_type.to_string())],
        [(
            header::CACHE_CONTROL,
            format!("max-age={}", ctx.cache_max_age.unwrap_or(31536000)),
        )],
        bytes,
    )
        .into_response());
}
