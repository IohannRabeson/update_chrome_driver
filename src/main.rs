///! Downloads the correct version of chromedriver regarding the version of the local Chrome.
///! Basically the rust implementation of https://chromedriver.chromium.org/downloads/version-selection.

use clap::Parser;
use std::fmt::{Display, Formatter};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::ffi::{OsStr};

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let platform = Platform::default();
    let chrome_version = get_local_browser_version(&cli.chrome_browser_path)?;
    let required_chrome_driver_version = get_required_driver_version(&chrome_version)?;
    let local_driver_version = get_local_driver_version(&cli.output_directory, platform)?;
    let require_update = must_update(&local_driver_version, &required_chrome_driver_version);

    println!("Required version: {}", required_chrome_driver_version);
    println!("Current version: {}", local_driver_version.as_ref().map(ToString::to_string).unwrap_or_else(||String::from("None")));
    println!("Require update: {}", require_update);

    if must_update(&local_driver_version, &required_chrome_driver_version) {
        let download_url = get_download_url(&required_chrome_driver_version, platform);

        println!("Download: {}", download_url);

        download_and_extract(&download_url, &cli.output_directory)?;
    }

    Ok(())
}

#[derive(Parser)]
struct Cli {
    /// The location of the local Google Chrome executable.
    pub chrome_browser_path: PathBuf,

    /// The location of the output directory where the Google Driver executable will
    /// be extracted.
    pub output_directory: PathBuf,
}

/// Version
///
/// https://www.chromium.org/developers/version-numbers/
#[derive(PartialEq, Eq, Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, build: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            build,
            patch,
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.build, self.patch
        )
    }
}

mod parsers;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Program '{0}' does not exist")]
    ProgramDoesNotExist(PathBuf),

    #[error("Can't run '{0}': {1}")]
    CantRunProgram(PathBuf, String),

    #[error("Failed to read output: {0}")]
    FailedToReadOutput(#[from] std::io::Error),

    #[error("Failed to parse version: {0}")]
    ParsingVersionFailed(String),

    #[error(transparent)]
    RequestFailed(#[from] reqwest::Error),

    #[error(transparent)]
    ZipExtractionFailed(#[from]zip::result::ZipError),
}

fn must_update(current_version: &Option<Version>, new_version: &Version) -> bool {
    if let Some(current_version) = current_version {
        return current_version.major < new_version.major
            || current_version.minor < new_version.minor
            || current_version.build < new_version.build
            || current_version.patch < new_version.patch
    }

    true
}

fn download_and_extract(url: &str, output_directory: &Path) -> Result<(), Error> {
    let response = Cursor::new(reqwest::blocking::get(url)?.bytes()?);
    let mut archive = zip::read::ZipArchive::new(response)?;

    archive.extract(output_directory)?;

    Ok(())
}

fn get_download_url(required_version: &Version, platform: Platform) -> String {
    format!("https://chromedriver.storage.googleapis.com/{}.{}.{}.{}/chromedriver_{}.zip",
            required_version.major, required_version.minor, required_version.build, required_version.patch,
            platform.get_key())
}


fn run_program<I, S>(program_path: &Path, arguments: I) -> Result<String, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    use std::process::Command;

    if !program_path.exists() {
        return Err(Error::ProgramDoesNotExist(program_path.to_path_buf()));
    }

    let output = Command::new(program_path)
        .args(arguments)
        .output()
        .map_err(|error| Error::CantRunProgram(program_path.to_path_buf(), error.to_string()))?;

    Ok(String::from_utf8_lossy(output.stdout.as_slice()).to_string())
}

#[cfg(not(target_os = "windows"))]
fn get_local_browser_version(program_path: &Path) -> Result<Version, Error> {
    let stdout = run_program(program_path, ["--version"])?;

    parsers::parse_chromium_version_output(&stdout)
        .map_err(|error| Error::ParsingVersionFailed(error.to_string()))
        .map(|(_, version)| version)
}

#[cfg(target_os = "windows")]
fn get_local_browser_version(program_path: &Path) -> Result<Version, Error> {
    let stdout = run_program(Path::new("C:\\Windows\\System32\\wbem\\WMIC.exe"), [
        "datafile", "where", &format!("name={:?}", program_path.display()),
        "get", "Version", "/value"
    ])?;

    parsers::parse_wmic_version(&stdout)
        .map_err(|error| Error::ParsingVersionFailed(error.to_string()))
        .map(|(_, version)| version)
}

// On Windows Chrome.exe seems to ignore all the arguments passed to the command line.
// Found this hackish way on stackoverflow..
// https://stackoverflow.com/questions/50880917/how-to-get-chrome-version-using-command-prompt-in-windows

fn get_required_driver_version(chrome_version: &Version) -> Result<Version, Error> {
    let url = format!(
        "https://chromedriver.storage.googleapis.com/LATEST_RELEASE_{}.{}.{}",
        chrome_version.major, chrome_version.minor, chrome_version.build
    );
    let response = reqwest::blocking::get(url)?.text()?;

    parsers::parse_version_numbers(&response)
        .map_err(|error| Error::ParsingVersionFailed(error.to_string()))
        .map(|(_, version)| version)
}

fn get_local_driver_version(driver_directory: &Path, platform: Platform) -> Result<Option<Version>, Error> {
    let program_path = driver_directory.join(platform.get_chromedriver_executable_name());

    if !program_path.exists() {
        return Ok(None)
    }

    let stdout = run_program(&program_path, ["--version"])?;

    parsers::parse_chromedriver_version_output(&stdout)
        .map_err(|error| Error::ParsingVersionFailed(error.to_string()))
        .map(|(_, version)| Some(version))
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum Platform {
    Windows,
    MacOs,
    Linux,
}

impl Platform {
    pub fn get_key(self) -> &'static str {
        match self {
            Platform::Windows => "win32",
            Platform::MacOs => "mac64",
            Platform::Linux => "linux64",
        }
    }

    pub fn get_chromedriver_executable_name(self) -> &'static str {
        match self {
            Platform::Windows => "chromedriver.exe",
            Platform::MacOs => "chromedriver",
            Platform::Linux => "chromedriver",
        }
    }
}

impl Default for Platform {
    fn default() -> Platform {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else {
            panic!("Unsupported platform")
        }
    }
}
