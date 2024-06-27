use aws_config::Region;
use aws_sdk_s3::{primitives::ByteStream, Client};
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, State},
    http::StatusCode,
    response::Response,
    routing::post,
    Router,
};
use futures::TryStreamExt;
use std::{io, pin::Pin};
use tokio::{
    fs::File,
    io::{AsyncRead, BufWriter},
};
use tokio_util::io::StreamReader;
use tower_http::limit::RequestBodyLimitLayer;

mod errors;

const UPLOADS_DIRECTORY: &str = "uploads";
#[derive(Clone, Debug)]
pub struct AppState {
    s3: Client,
}

#[tokio::main]
async fn main() {
    // uses the default aws credentials provider chain
    // AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, and AWS_SESSION_TOKEN
    let cfg = aws_config::from_env()
        .endpoint_url("https://sgp1.digitaloceanspaces.com")
        .region(Region::new("us-east-1"))
        .load()
        .await;
    let s3 = Client::new(&cfg);

    let state = AppState { s3 };

    let router = Router::new()
        .route("/upload", post(upload))
        .with_state(state)
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            1024 * 1024 * 1024 * 10, /* 10GB */
        ));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router)
        .tcp_nodelay(true)
        .await
        .unwrap();
}

pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Response, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let filename = if let Some(filename) = field.file_name() {
            filename.to_string()
        } else {
            continue;
        };

        let body_with_io_error = field.map_err(|err| io::Error::new(io::ErrorKind::Other, err));

        let body_reader = StreamReader::new(body_with_io_error);

        futures::pin_mut!(body_reader);

        put_file(state.s3, &filename, body_reader).await.unwrap();

        return Ok(Response::builder()
            .status(StatusCode::CREATED)
            .body(Body::from("OK".to_string()))
            .unwrap());
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn put_file(
    s3: Client,
    filename: &str,
    mut reader: Pin<&mut (dyn AsyncRead + Send)>,
) -> Result<(), (StatusCode, String)> {
    async {
        let path = std::path::Path::new(UPLOADS_DIRECTORY).join(filename);
        let cloned_path = path.clone();
        // Create the file. `File` implements `AsyncWrite`.
        let mut file = BufWriter::with_capacity(1024 * 1024 * 250, File::create(path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut reader, &mut file).await?;

        let file = ByteStream::from_path(cloned_path).await;
        s3.put_object()
            .bucket("transitfiles")
            .body(file.unwrap())
            .key(filename)
            .send()
            .await
            .map_err(|err| {
                println!("Error: {:?}", err);
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("S3 error: {}", err.to_string()),
                )
            })?;

        tokio::fs::remove_file(std::path::Path::new(UPLOADS_DIRECTORY).join(filename)).await?;
        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))

    // let _res = s3
    //     .put_object()
    //     .bucket("transitfiles")
    //     .key(filename)
    //     .body(new_vec.into())
    //     .send()
    //     .await?;
}
