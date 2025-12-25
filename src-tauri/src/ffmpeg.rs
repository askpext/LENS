use serde_json::{json, Value};
use ffmpeg_sidecar::{command::FfmpegCommand, download::auto_download, paths::sidecar_dir};

pub async fn check_and_install() -> Result<String, String> {
    // Try to create an FFmpeg command to check if it exists
    match FfmpegCommand::new().spawn() {
        Ok(_) => return Ok("ready".to_string()),
        Err(_) => {
            // FFmpeg not found, try to download it
            match auto_download() {
                Ok(_) => Ok("installed".to_string()),
                Err(e) => Err(format!("Failed to download FFmpeg: {}", e)),
            }
        }
    }
}

pub async fn get_video_info(path: &str) -> Result<Value, String> {
    // Ensure FFmpeg is available
    check_and_install().await?;

    // Get the ffprobe path
    let sidecar_path = sidecar_dir()
        .map_err(|e| format!("Failed to get sidecar directory: {}", e))?;
    
    let ffprobe_path = if cfg!(windows) {
        sidecar_path.join("ffprobe.exe")
    } else {
        sidecar_path.join("ffprobe")
    };

    let output = std::process::Command::new(ffprobe_path)
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            "-show_chapters",
            path,
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffprobe: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFprobe error: {}", stderr));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let data: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(parse_video_data(data))
}

fn parse_video_data(data: Value) -> Value {
    let empty_vec = vec![];
    let streams = data["streams"].as_array().unwrap_or(&empty_vec);
    let format = &data["format"];
    let chapters = data["chapters"].as_array().unwrap_or(&empty_vec);

    let video_stream = streams.iter().find(|s| s["codec_type"] == "video");
    let audio_streams: Vec<&Value> = streams.iter().filter(|s| s["codec_type"] == "audio").collect();
    let subtitle_streams: Vec<&Value> = streams.iter().filter(|s| s["codec_type"] == "subtitle").collect();
    let data_streams: Vec<&Value> = streams.iter().filter(|s| s["codec_type"] == "data").collect();

    // Calculate total frames if possible
    let total_frames = video_stream.and_then(|v| {
        v["nb_frames"].as_str().and_then(|f| f.parse::<u64>().ok())
            .or_else(|| {
                // Calculate from duration and fps
                let duration = format["duration"].as_str()?.parse::<f64>().ok()?;
                let fps_str = v["r_frame_rate"].as_str()?;
                let fps_parts: Vec<&str> = fps_str.split('/').collect();
                if fps_parts.len() == 2 {
                    let num: f64 = fps_parts[0].parse().ok()?;
                    let den: f64 = fps_parts[1].parse().ok()?;
                    Some((duration * (num / den)) as u64)
                } else {
                    None
                }
            })
    });

    json!({
        "format": {
            "filename": format["filename"],
            "format_name": format["format_name"],
            "format_long_name": format["format_long_name"],
            "duration": format["duration"].as_str().and_then(|d| d.parse::<f64>().ok()).unwrap_or(0.0),
            "size": format["size"].as_str().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0),
            "bit_rate": format["bit_rate"].as_str().and_then(|b| b.parse::<u64>().ok()).unwrap_or(0),
            "nb_streams": format["nb_streams"],
            "nb_programs": format["nb_programs"],
            "probe_score": format["probe_score"],
            "tags": format.get("tags").cloned().unwrap_or(json!({})),
        },
        "video": video_stream.map(|v| {
            let width = v["width"].as_u64().unwrap_or(0);
            let height = v["height"].as_u64().unwrap_or(0);
            let aspect_ratio = if height > 0 {
                format!("{:.2}", width as f64 / height as f64)
            } else {
                "N/A".to_string()
            };

            json!({
                "index": v["index"],
                "codec": v["codec_name"],
                "codec_long": v["codec_long_name"],
                "codec_tag": v.get("codec_tag_string").cloned().unwrap_or(json!("")),
                "profile": v.get("profile").cloned().unwrap_or(json!("")),
                "level": v.get("level").cloned().unwrap_or(json!("")),
                "width": width,
                "height": height,
                "aspect_ratio": aspect_ratio,
                "display_aspect_ratio": v.get("display_aspect_ratio").cloned().unwrap_or(json!("")),
                "sample_aspect_ratio": v.get("sample_aspect_ratio").cloned().unwrap_or(json!("")),
                "fps": v["r_frame_rate"],
                "avg_fps": v.get("avg_frame_rate").cloned().unwrap_or(json!("")),
                "time_base": v.get("time_base").cloned().unwrap_or(json!("")),
                "bit_rate": v["bit_rate"].as_str().and_then(|b| b.parse::<u64>().ok()).unwrap_or(0),
                "max_bit_rate": v.get("max_bit_rate").cloned().unwrap_or(json!("")),
                "pix_fmt": v["pix_fmt"],
                "color_space": v.get("color_space").cloned().unwrap_or(json!("")),
                "color_transfer": v.get("color_transfer").cloned().unwrap_or(json!("")),
                "color_primaries": v.get("color_primaries").cloned().unwrap_or(json!("")),
                "color_range": v.get("color_range").cloned().unwrap_or(json!("")),
                "chroma_location": v.get("chroma_location").cloned().unwrap_or(json!("")),
                "field_order": v.get("field_order").cloned().unwrap_or(json!("")),
                "refs": v.get("refs").cloned().unwrap_or(json!("")),
                "has_b_frames": v.get("has_b_frames").cloned().unwrap_or(json!(0)),
                "bits_per_raw_sample": v.get("bits_per_raw_sample").cloned().unwrap_or(json!("")),
                "start_time": v.get("start_time").cloned().unwrap_or(json!("")),
                "duration": v.get("duration").cloned().unwrap_or(json!("")),
                "nb_frames": total_frames,
                "rotation": v.get("tags").and_then(|t| t.get("rotate")).cloned().unwrap_or(json!("")),
                "tags": v.get("tags").cloned().unwrap_or(json!({})),
            })
        }),
        "audio": audio_streams.iter().enumerate().map(|(i, a)| json!({
            "index": a["index"],
            "track_number": i + 1,
            "codec": a["codec_name"],
            "codec_long": a["codec_long_name"],
            "codec_tag": a.get("codec_tag_string").cloned().unwrap_or(json!("")),
            "profile": a.get("profile").cloned().unwrap_or(json!("")),
            "sample_rate": a["sample_rate"],
            "channels": a["channels"],
            "channel_layout": a.get("channel_layout").cloned().unwrap_or(json!("")),
            "bits_per_sample": a.get("bits_per_sample").cloned().unwrap_or(json!("")),
            "bit_rate": a["bit_rate"].as_str().and_then(|b| b.parse::<u64>().ok()).unwrap_or(0),
            "max_bit_rate": a.get("max_bit_rate").cloned().unwrap_or(json!("")),
            "sample_fmt": a.get("sample_fmt").cloned().unwrap_or(json!("")),
            "time_base": a.get("time_base").cloned().unwrap_or(json!("")),
            "start_time": a.get("start_time").cloned().unwrap_or(json!("")),
            "duration": a.get("duration").cloned().unwrap_or(json!("")),
            "nb_frames": a.get("nb_frames").cloned().unwrap_or(json!("")),
            "language": a.get("tags").and_then(|t| t.get("language")).cloned().unwrap_or(json!("")),
            "title": a.get("tags").and_then(|t| t.get("title")).cloned().unwrap_or(json!("")),
            "tags": a.get("tags").cloned().unwrap_or(json!({})),
        })).collect::<Vec<_>>(),
        "subtitles": subtitle_streams.iter().enumerate().map(|(i, s)| json!({
            "index": s["index"],
            "track_number": i + 1,
            "codec": s["codec_name"],
            "codec_long": s.get("codec_long_name").cloned().unwrap_or(json!("")),
            "codec_tag": s.get("codec_tag_string").cloned().unwrap_or(json!("")),
            "language": s.get("tags").and_then(|t| t.get("language")).cloned().unwrap_or(json!("")),
            "title": s.get("tags").and_then(|t| t.get("title")).cloned().unwrap_or(json!("")),
            "forced": s.get("disposition").and_then(|d| d.get("forced")).cloned().unwrap_or(json!(0)),
            "default": s.get("disposition").and_then(|d| d.get("default")).cloned().unwrap_or(json!(0)),
            "tags": s.get("tags").cloned().unwrap_or(json!({})),
        })).collect::<Vec<_>>(),
        "data_streams": data_streams.iter().map(|d| json!({
            "index": d["index"],
            "codec": d.get("codec_name").cloned().unwrap_or(json!("")),
            "codec_long": d.get("codec_long_name").cloned().unwrap_or(json!("")),
            "tags": d.get("tags").cloned().unwrap_or(json!({})),
        })).collect::<Vec<_>>(),
        "chapters": chapters.iter().map(|c| json!({
            "id": c.get("id").cloned().unwrap_or(json!(0)),
            "time_base": c.get("time_base").cloned().unwrap_or(json!("")),
            "start": c.get("start").cloned().unwrap_or(json!(0)),
            "end": c.get("end").cloned().unwrap_or(json!(0)),
            "start_time": c.get("start_time").cloned().unwrap_or(json!("")),
            "end_time": c.get("end_time").cloned().unwrap_or(json!("")),
            "tags": c.get("tags").cloned().unwrap_or(json!({})),
        })).collect::<Vec<_>>(),
    })
}