use std::{fs::File, io::Write, ops::RangeTo, path::Path};

use axum::{
    body::StreamBody,
    extract::{Multipart, Query},
    http::StatusCode,
    response::IntoResponse,
};
use ffmpeg_cli::FfmpegBuilder;
use infer::{MatcherType, Type};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio_util::io::ReaderStream;
use tracing::{error, log::warn};

#[derive(Debug, Serialize, Deserialize)]
enum AudioVisualFormats {}

#[derive(Debug, Deserialize)]
pub struct AudioVisualQuery {
    format: Option<AudioVisualFormats>,
    /// By default, the value is ImageMagick's [default convert -quality](https://imagemagick.org/script/command-line-options.php#quality)
    quality: Option<u8>,
}

impl Default for AudioVisualQuery {
    fn default() -> Self {
        Self {
            format: Default::default(),
            quality: Some(85),
        }
    }
}

pub async fn root(
    Query(AudioVisualQuery { format, quality }): Query<AudioVisualQuery>,
    mut multipart: Multipart,
) -> Result<StreamBody<ReaderStream<tokio::fs::File>>, StatusCode> {
    let (tmp_file, matcher_type) = get_file(multipart).await?;

    if !matches!(matcher_type, MatcherType::Audio | MatcherType::Video) {
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    let Some(tmp_file_path) = tmp_file.path().to_str() else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Ok(output) = tempfile::NamedTempFile::new() else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Some(output_path) = output.path().to_str() else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    // g(tmp_file.path());

    // let d = lower_bitrate(tmp_file.path()).await;
    // warn!("{:?}", d);

    // let ffmpeg_builder = FfmpegBuilder::new()
    //     .input(ffmpeg_cli::File::new(tmp_file_path))
    //     .output(
    //         ffmpeg_cli::File::new(output_path), // .option(Parameter::KeyValue("vcodec", "libx265"))
    //                                             // .option(Parameter::KeyValue("crf", "28")),
    //     );

    // let Ok(ffmpeg) = ffmpeg_builder.run().await else {
    //     return Err(StatusCode::UNPROCESSABLE_ENTITY);
    // };

    // let Ok(f) = ffmpeg.process.wait_with_output() else {
    //     return Err(StatusCode::UNPROCESSABLE_ENTITY);
    // };

    // warn!("haha");

    // if !f.status.success() {
    //     return Err(StatusCode::UNPROCESSABLE_ENTITY);
    // }

    // warn!("{}", f.status);

    get_stream(tmp_file.path()).await
}

async fn get_file(
    mut multipart: Multipart,
) -> Result<(tempfile::NamedTempFile, MatcherType), StatusCode> {
    let Ok(Some(field)) = multipart.next_field().await else {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    };

    let Ok(mut tmp_file) = tempfile::NamedTempFile::new() else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if tmp_file.write_all(&field.bytes().await.unwrap()).is_err() {
        return Err(StatusCode::BAD_REQUEST);
    };

    let file_type = infer::get_from_path(tmp_file.path()).unwrap().unwrap();

    let file = tempfile::Builder::new()
        .suffix(&format!(".{}", file_type.extension()))
        .tempfile();

    let Ok(f) = file else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if fs::copy(tmp_file.path(), f.path()).await.is_err() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok((f, file_type.matcher_type()))
}

async fn get_stream(path: &Path) -> Result<StreamBody<ReaderStream<tokio::fs::File>>, StatusCode> {
    // `File` implements `AsyncRead`
    let Ok(file) = tokio::fs::File::open(path).await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    Ok(StreamBody::new(stream))
}
