extern crate regex;
extern crate structopt;

use regex::Regex;
use std::env;
use std::process::exit;
use std::process::Command;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
// Rename all will use the name of the field
#[structopt(rename_all = "kebab-case")]
pub struct Opt {
    /// Set the branch
    #[structopt(short, long)]
    branch: Option<String>,

    /// Set the browser
    #[structopt(short = "-B", long)]
    browser: Option<String>,

    /// Set the remote to use
    #[structopt(short, long)]
    remote: Option<String>,

    /// Set the verbosity of the command
    #[structopt(short, long)]
    verbose: bool,
}

/**
 * Error list. I do not know if it is the best for Rust to declare constants and
 * use it after.
 */
const NOT_A_GIT_REPOSITORY: i32 = 1;

#[cfg(target_os = "linux")]
fn get_command_output(command: &str) -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");

    return String::from(
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .trim_start(),
    );
}

#[cfg(target_os = "linux")]
fn run(browser: &str, url: &str) {
    Command::new(browser)
        .arg(url)
        .output()
        .expect("failed to execute process");
}

#[cfg(target_os = "linux")]
fn get_default_browser() -> String {
    String::from("firefox")
}

#[cfg(target_os = "windows")]
fn run(browser: &str, url: &str) {
    Command::new("cmd")
        .arg("/C")
        .arg(browser)
        .arg(url)
        .output()
        .expect("failed to execute process");
}

#[cfg(target_os = "windows")]
fn get_command_output(command: &str) -> String {
    let output = Command::new("cmd")
        .arg("/C")
        .arg(command)
        .output()
        .expect("failed to execute process");

    return String::from(
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .trim_start(),
    );
}

#[cfg(target_os = "windows")]
fn get_default_browser() -> String {
    String::from("C:\\Program Files\\Mozilla Firefox\\firefox.exe")
}

fn is_inside_working_tree() -> bool {
    get_command_output("git rev-parse --is-inside-work-tree") == "true"
}

fn get_remote_url() -> String {
    get_command_output("git config --get remote.origin.url")
}

fn main() {
    let opt = Opt::from_args();

    if opt.verbose {
        println!("Verbose is ON");
    }

    println!(
        "{:?}",
        get_command_output("git rev-parse --abbrev-ref HEAD")
    );

    // Check that the user is in a git repository.
    if !is_inside_working_tree() {
        println!("ERROR: This is not a git directory");
        exit(NOT_A_GIT_REPOSITORY);
    }

    let remote = get_remote_url();

    let re = Regex::new(r".*@(.*):(.*)\.git").unwrap();
    let caps = re.captures(remote.as_str()).unwrap();

    let domain = caps.get(1).map_or("github.com", |m| m.as_str());
    let repository = caps.get(2).map_or("", |m| m.as_str());

    let url = format!(
        "https://{domain}/{repository}",
        domain = domain,
        repository = repository
    );

    let key: &str = "BROWSER";

    let browser = match env::var(key) {
        Ok(browser) => browser,
        Err(e) => {
            if opt.verbose {
                println!("{:?}", e)
            }

            opt.browser.unwrap_or(get_default_browser())
        }
    };

    run(browser.as_str(), url.as_str());
}
