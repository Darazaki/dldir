#[macro_use]
extern crate clap;

use regex::Regex;
use walkdir::WalkDir;

use std::{fs, fs::File, io, io::Write};

fn main() -> io::Result<()> {
    let matches = clap_app!(dldir =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg DOWNLOAD_URL: +takes_value
            "Downloads a dldir.txt from a root url")
        (@arg EXCLUDED_REGEX: -x --exclude +takes_value
            "Exclude a pattern of paths")
        (@arg clear: -c --clear
            "Clears the current directory before downloading")
    )
    .get_matches();

    let excluded = matches
        .value_of("EXCLUDED_REGEX")
        .map(|pat| Regex::new(pat).unwrap());

    if let Some(url) = matches.value_of("DOWNLOAD_URL") {
        if matches.is_present("clear") {
            clear_current_directory()?;
        }

        match download(sanitized_url(url), excluded) {
            Ok(()) => Ok(()),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
        }
    } else {
        generate(excluded)
    }
}

fn generate(excluded: Option<Regex>) -> io::Result<()> {
    let mut file = File::create("dldir.txt")?;

    let paths = WalkDir::new(".")
        .min_depth(1)
        .into_iter()
        .filter_entry(|entry| match &excluded {
            Some(ref pat) => match entry.file_name().to_str() {
                Some(filename) => !pat.is_match(filename),
                None => true,
            },
            None => true,
        })
        .filter_map(|entry| {
            let entry = entry.ok()?;

            let is_dir = entry.file_type().is_dir();
            let path = entry.into_path();

            if path == std::ffi::OsStr::new("./dldir.txt") {
                None
            } else {
                Some((is_dir, path))
            }
        });

    for (is_dir, path) in paths {
        let path = path.strip_prefix("./").unwrap();

        if is_dir {
            writeln!(file, "d{}", path.display())?;
        } else {
            writeln!(file, "f{}", path.display())?;
        }
    }

    Ok(())
}

fn download(url: &str, excluded: Option<Regex>) -> Result<(), reqwest::Error> {
    let text =
        reqwest::blocking::get(&(url.to_owned() + "/dldir.txt"))?.text()?;

    println!(".");

    for line in
        text.lines()
            .map(|line| line.trim())
            .filter(|line| match &excluded {
                Some(pat) => !pat.is_match(line),
                None => true,
            })
    {
        match line.chars().next() {
            None => (),
            Some('d') => {
                use io::ErrorKind::AlreadyExists;

                let path = &line[1..];
                println!("\n{}", path);
                match fs::create_dir(path) {
                    Ok(()) => (),
                    Err(err) if err.kind() == AlreadyExists => (),
                    Err(err) => Err(err).unwrap(),
                }
            },
            Some('f') => {
                let path = &line[1..];
                println!(" - {}", filename(path));
                download_file(url, path)?;
            },
            Some(_) => panic!("Unknown file type passed"),
        }
    }

    Ok(())
}

fn download_file(url: &str, filename: &str) -> Result<(), reqwest::Error> {
    let mut request =
        reqwest::blocking::get(&(url.to_owned() + "/" + filename))?;
    let mut file = File::create(filename).unwrap();

    request.copy_to(&mut file)?;

    Ok(())
}

fn clear_current_directory() -> io::Result<()> {
    std::path::Path::new(".")
        .read_dir()?
        .map(|entry| {
            let path = entry?.path();

            if path.is_dir() {
                fs::remove_dir_all(path)
            } else {
                fs::remove_file(path)
            }
        })
        .fold_results()
}

fn filename(path: &str) -> &str {
    std::path::Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
}

fn sanitized_url(url: &str) -> &str {
    url.trim()
        .trim_end_matches("/dldir.txt")
        .trim_end_matches('/')
}

trait FoldResultsExt {
    type Error;

    fn fold_results(&mut self) -> Result<(), Self::Error>;
}

impl<E, I> FoldResultsExt for I
where
    I: Iterator<Item = Result<(), E>>,
{
    type Error = E;

    fn fold_results(&mut self) -> Result<(), Self::Error> {
        for result in self {
            match result {
                Ok(()) => (),
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}
