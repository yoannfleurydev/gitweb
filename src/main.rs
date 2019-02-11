extern crate open;
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

    /// Set the remote
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

fn is_inside_working_tree() -> bool {
    get_command_output("git rev-parse --is-inside-work-tree") == "true"
}

fn get_remote_url() -> String {
    get_command_output("git config --get remote.origin.url")
}

fn print_verbose(string: &str, verbose: &bool) {
    if *verbose {
        println!("{}", string)
    }
}

fn main() {
    let opt = Opt::from_args();

    print_verbose("Verbose is ON", &opt.verbose);

    // Check that the user is in a git repository.
    if !is_inside_working_tree() {
        println!("ERROR: This is not a git directory");
        exit(NOT_A_GIT_REPOSITORY);
    }

    // Get the branch to show in the browser.
    let branch = match opt.branch {
        Some(branch) => branch,
        None => {
            print_verbose("No branch given, getting current one", &opt.verbose);

            // Get the current branch the user is on.
            get_command_output("git rev-parse --abbrev-ref HEAD")
        }
    };

    let remote = get_remote_url();

    let re = Regex::new(r".*@(.*):(.*)\.git").unwrap();
    let caps = re.captures(remote.as_str()).unwrap();

    let domain = caps.get(1).map_or("github.com", |m| m.as_str());
    let repository = caps.get(2).map_or("", |m| m.as_str());

    let url = format!(
        "https://{domain}/{repository}/tree/{branch}",
        domain = domain,
        repository = repository,
        branch = branch
    );

    // Open the browser.
    match env::var("BROWSER") {
        // If the environment variable is available, open the web browser.
        Ok(browser) => run(browser.as_str(), url.as_str()),
        Err(e) => {
            print_verbose(
                format!("BROWSER variable not available : {}", e).as_str(),
                &opt.verbose,
            );

            // If the option is available through the command line, open
            // the given web browser
            if opt.browser.is_some() {
                let option_browser = opt.browser.unwrap();

                print_verbose(
                    format!("Browser {} given as option", option_browser).as_str(),
                    &opt.verbose,
                );

                run(option_browser.as_str(), url.as_str());
            } else {
                print_verbose("Opening default browser", &opt.verbose);

                // Open the default web browser on the current system.
                open::that(url);
            }
        }
    };
}
