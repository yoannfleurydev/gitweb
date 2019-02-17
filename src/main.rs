extern crate git2;
extern crate open;
extern crate regex;
extern crate structopt;

use git2::{ErrorCode, Repository};
use regex::Regex;
use std::env;
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

#[cfg(target_os = "linux")]
fn open_browser(browser: &String, url: &String) {
    Command::new(browser)
        .arg(url)
        .output()
        .expect("failed to execute process");
}

#[cfg(target_os = "windows")]
fn open_browser(browser: &String, url: &String) {
    Command::new(browser)
        .arg(url)
        .output()
        .expect("failed to execute process");
}

fn get_repo() -> Repository {
    return match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
}

fn get_branch(repo: &Repository, verbose: &bool) -> String {
    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => {
            None
        }
        Err(e) => panic!("failed to get head ref {}", e),
    };

    let head = head.as_ref().and_then(|h| h.shorthand());
    print_verbose(
        format!(
            "# On branch {}",
            head.unwrap_or("Not currently on any branch")
        )
        .as_str(),
        verbose,
    );

    String::from(head.unwrap_or("master"))
}

fn print_verbose(string: &str, verbose: &bool) {
    if *verbose {
        println!("{}", string)
    }
}

fn main() {
    // Get the command line options
    let opt = Opt::from_args();

    print_verbose("Verbose is active", &opt.verbose);

    // Check that the user is in a git repository.
    let repo = get_repo();

    // Get the branch to show in the browser.
    let branch = match opt.branch {
        Some(branch) => branch,
        None => {
            print_verbose("No branch given, getting current one", &opt.verbose);

            get_branch(&repo, &opt.verbose)
        }
    };

    let remote_name = &opt.remote.unwrap_or("origin".to_string());

    print_verbose(
        format!("Getting remote for {}", remote_name).as_str(),
        &opt.verbose,
    );

    let optional_remote = match repo.find_remote(remote_name) {
        Ok(remote) => remote,
        Err(e) => panic!("failed to get remote {}", e),
    };

    let remote_url = match optional_remote.url() {
        Some(remote) => remote,
        None => panic!("no remote available"),
    };

    let re = Regex::new(r".*@(.*):(.*)\.git").unwrap();
    let caps = re.captures(remote_url).unwrap();

    let domain = caps.get(1).map_or("github.com", |m| m.as_str());
    let repository = caps.get(2).map_or("", |m| m.as_str());

    let url = format!(
        "https://{domain}/{repository}/tree/{branch}",
        domain = domain,
        repository = repository,
        branch = branch
    );

    // Open the browser.
    // If the option is available through the command line, open
    // the given web browser
    if opt.browser.is_some() {
        let option_browser = opt.browser.unwrap();

        print_verbose(
            format!("Browser {} given as option", option_browser).as_str(),
            &opt.verbose,
        );

        open_browser(&option_browser, &url);
    } else {
        match env::var("BROWSER") {
            // If the environment variable is available, open the web browser.
            Ok(browser) => open_browser(&browser, &url),
            Err(e) => {
                print_verbose(
                    format!("BROWSER variable not available : {}", e).as_str(),
                    &opt.verbose,
                );

                print_verbose("Opening default browser", &opt.verbose);

                // Open the default web browser on the current system.
                match open::that(url) {
                    Ok(res) => println!("{:?}", res),
                    Err(err) => panic!("failed to open the browser : {}", err),
                }
            }
        }
    };
}
