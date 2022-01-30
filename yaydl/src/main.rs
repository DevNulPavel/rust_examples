/*
 * The contents of this file are subject to the terms of the
 * Common Development and Distribution License, Version 1.0 only
 * (the "License").  You may not use this file except in compliance
 * with the License.
 *
 * See the file LICENSE in this distribution for details.
 * A copy of the CDDL is also available via the Internet at
 * http://www.opensource.org/licenses/cddl1.txt
 *
 * When distributing Covered Code, include this CDDL HEADER in each
 * file and include the contents of the LICENSE file from this
 * distribution.
 */

// Yet Another Youtube Down Loader
// - main.rs file -

use anyhow::Result;
use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    borrow::Cow,
    fs,
    io::{self, copy, Read},
    path::{Path, PathBuf},
};
use url::Url;

mod definitions;
mod ffmpeg;
mod handlers;

struct DownloadProgress<R> {
    inner: R,
    progress_bar: ProgressBar,
}

impl<R: Read> Read for DownloadProgress<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_bar.inc(n as u64);
            n
        })
    }
}

fn download(url: &str, filename: &str) -> Result<()> {
    let url = Url::parse(url)?;
    let resp = ureq::get(url.as_str()).call()?;

    // Find the video size:
    let total_size = resp
        .header("Content-Length")
        .unwrap_or("0")
        .parse::<u64>()?;

    let mut request = ureq::get(url.as_str());

    // Display a progress bar:
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
                 .template("{spinner:.green} [{elapsed_precise}] [{bar:40.green/blue}] {bytes}/{total_bytes} ({eta})")
                 .progress_chars("#>-"));

    let file = Path::new(filename);

    if file.exists() {
        // Continue the file:
        let size = file.metadata()?.len() - 1;
        // Override the range:
        request = ureq::get(url.as_str()).set("Range", &format!("bytes={}-", size));
        pb.inc(size);
    }

    let resp = request.call()?;
    let mut source = DownloadProgress {
        progress_bar: pb,
        inner: resp.into_reader(),
    };

    let mut dest = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)?;

    let _ = copy(&mut source, &mut dest)?;

    Ok(())
}

fn main() -> Result<()> {
    // Argument parsing:
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let args = App::new("yaydl")
        .version(VERSION)
        .about("Yet Another Youtube Down Loader")
        .arg(Arg::new("onlyaudio")
             .about("Only keeps the audio stream")
             .short('x')
             .long("only-audio"))
        .arg(Arg::new("verbose")
             .about("Talks more while the URL is processed")
             .short('v')
             .long("verbose"))
        .arg(Arg::new("audioformat")
             .about("Sets the target audio format (only if --only-audio is used).\nSpecify the file extension here (defaults to \"mp3\").")
             .short('f')
             .long("audio-format")
             .takes_value(true)
             .value_name("AUDIO"))
        .arg(Arg::new("outputfile")
             .about("Sets the output file name")
             .short('o')
             .long("output")
             .takes_value(true)
             .value_name("OUTPUTFILE"))
        .arg(Arg::new("URL")
             .about("Sets the input URL to use")
             .required(true)
             .index(1))
        .get_matches();

    if let Some(in_url) = args.value_of("URL") {
        // Регистрируем все структуры, которые реализуют definitions::SiteDefinition
        inventory::collect!(&'static dyn definitions::SiteDefinition);
        let mut site_def_found = false;

        for handler in inventory::iter::<&dyn definitions::SiteDefinition> {
            // "15:15 And he found a pair of eyes, scanning the directories for files."
            // https://kingjamesprogramming.tumblr.com/post/123368869357/1515-and-he-found-a-pair-of-eyes-scanning-the
            // ------------------------------------

            // Может ли данный обработчик обработать url?
            if !handler.can_handle_url(in_url) {
                continue;
            }

            // Данный обработчик может обработать
            site_def_found = true;
            println!("Fetching from {}.", handler.display_name());

            // Есть ли видео такое?
            let video_exists = handler.does_video_exist(in_url)?;
            if !video_exists {
                println!("The video could not be found. Invalid link?");
            } else {
                if args.is_present("verbose") {
                    println!("The requested video was found. Processing...");
                }

                // Получаем имя нашего видео
                let video_title = handler.find_video_title(in_url);
                let vt = match video_title {
                    Err(_e) => "".to_string(),
                    Ok(title) => title,
                };

                // Usually, we already find errors here.

                if vt.is_empty() {
                    println!("The video title could not be extracted. Invalid link?");
                } else {
                    if args.is_present("verbose") {
                        println!("Title: {}", vt);
                    }

                    // Находим прямую ссылку на скачивание видео
                    let url =
                        handler.find_video_direct_url(in_url, args.is_present("onlyaudio"))?;
                    // Определяем расширение для файлика
                    let ext =
                        handler.find_video_file_extension(in_url, args.is_present("onlyaudio"))?;

                    // Начинаем загрузку файлика
                    let targetfile = if let Some(in_targetfile) = args.value_of("outputfile") {
                        in_targetfile.to_string()
                    } else {
                        // Обрезаем заголовок файлика, заменяем всякие странные символы на пустую строку
                        let fixed_title = vt
                            .trim()
                            .replace(['|', '\'', '\"', ':', '\'', '\\', '/'].as_slice(), r#""#);
                        format!("{}.{}", fixed_title, ext)
                    };

                    if args.is_present("verbose") {
                        println!("Starting the download.");
                    }

                    // Стартуем загрузку
                    download(&url, &targetfile)?;

                    if args.is_present("onlyaudio") {
                        if args.is_present("verbose") {
                            println!("Post-processing.");
                        }

                        // Convert the file if needed.
                        let outputext = if let Some(in_outputext) = args.value_of("audioformat") {
                            in_outputext
                        } else {
                            "mp3"
                        };

                        // Заменяем расширение файлика
                        let outpath = {
                            let inpath = Path::new(&targetfile);
                            if outputext != ext {
                                let mut outpathbuf = PathBuf::from(&targetfile);
                                outpathbuf.set_extension(outputext);
                                // Перегоняем в аудио
                                ffmpeg::to_audio(inpath, outpathbuf.as_path());
                                // Удаляем старый файлик
                                fs::remove_file(&targetfile)?;
                                Cow::Owned(outpathbuf)
                            } else {
                                // Перегоняем в аудио
                                ffmpeg::to_audio(inpath, inpath);
                                Cow::Borrowed(inpath)
                            }
                        };

                        // Success!
                        println!("\"{}\" successfully downloaded.", outpath.display());
                    } else {
                        // ... just success!
                        println!("\"{}\" successfully downloaded.", &targetfile);
                    }
                }

                // Stop looking for other handlers:
                break;
            }
        }

        if !site_def_found {
            println!(
                "yaydl could not find a site definition that would satisfy {}. Exiting.",
                in_url
            );
        }
    }

    Ok(())
}
