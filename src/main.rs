use std::{
    env,
    io::{BufWriter, Cursor},
    net::SocketAddr,
};

use axum::{extract::Query, response::IntoResponse, routing::get, Router};
use image::ImageFormat;
use request_context::RequestContext;
use reqwest::{header, StatusCode};
mod request_context;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handle));

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
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: check if domain allowed

    // get image format
    let extension_string = &ctx.format.unwrap_or("png".into());
    let format = ImageFormat::from_extension(extension_string).unwrap_or(ImageFormat::Png);
    let mime_type = mime_guess::from_ext(extension_string).first_or("image/png".parse().unwrap());

    // download imgage
    let res = reqwest::get(ctx.url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if res.error_for_status_ref().is_err() {
        let error = &res.error_for_status_ref().unwrap_err();
        return Err((res.status(), error.to_string()));
    }
    let bytes = res
        .bytes()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let image = image::load_from_memory(&bytes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // resize
    let image = image.resize(
        ctx.width.unwrap_or(image.width()),
        ctx.height.unwrap_or(image.height()),
        image::imageops::FilterType::Nearest,
    );

    // convert
    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    image
        .write_to(&mut buffer, format)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let bytes: Vec<u8> = buffer
        .into_inner()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_inner();

    return Ok((
        [(header::CONTENT_TYPE, mime_type.to_string())],
        [(
            header::CACHE_CONTROL,
            format!("max-age={}", ctx.cache_max_age.unwrap_or(31536000)),
        )],
        bytes,
    ));
}
