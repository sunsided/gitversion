use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use color_eyre::eyre::Result;
use env_logger::Target;
use gitversion::agents::detect_build_agent;
use gitversion::calculation::next_version::NextVersionCalculator;
use gitversion::config::provider::ConfigurationProvider;
use gitversion::context::GitVersionContext;
use gitversion::output::dotenv::write_dotenv;
use gitversion::output::file::write_output_file;
use gitversion::output::json::to_json;
use gitversion::output::variable_provider::VariableProvider;
use gitversion::prepare::{GitPrepareOptions, GitPreparer, GitRemoteAuth};
use log::LevelFilter;

#[derive(Debug, Clone, ValueEnum)]
enum OutputType {
    Json,
    File,
    Buildserver,
    Dotenv,
}

#[derive(Debug, Clone, ValueEnum)]
enum Verbosity {
    Quiet,
    Minimal,
    Normal,
    Verbose,
    Diagnostic,
}

#[derive(Parser, Debug)]
#[command(name = "gitversion")]
struct Cli {
    #[arg(default_value = ".")]
    target_path: PathBuf,
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long = "overrideconfig", num_args = 1..)]
    override_config: Vec<String>,
    #[arg(short = 'v', long = "showvariable")]
    show_variable: Option<String>,
    #[arg(short = 'l', long = "log-file")]
    log_file: Option<PathBuf>,
    #[arg(long)]
    url: Option<String>,
    #[arg(short = 'u', long)]
    username: Option<String>,
    #[arg(short = 'p', long)]
    password: Option<String>,
    #[arg(short = 'c', long = "commit")]
    commit: Option<String>,
    #[arg(short = 'b', long = "branch")]
    branch: Option<String>,
    #[arg(long = "no-cache")]
    no_cache: bool,
    #[arg(long = "showConfig", alias = "show-config")]
    show_config: bool,
    #[arg(long, value_enum)]
    verbosity: Option<Verbosity>,
    #[arg(long)]
    diag: bool,
    #[arg(long)]
    format: Option<String>,
    #[arg(long, value_delimiter = ' ')]
    output: Vec<OutputType>,
    #[arg(long = "outputfile")]
    output_file: Option<PathBuf>,
    #[arg(long)]
    no_fetch: bool,
    #[arg(long)]
    no_normalize: bool,
    #[arg(long)]
    allow_shallow: bool,
}

fn parse_override(items: &[String]) -> HashMap<String, String> {
    items
        .iter()
        .filter_map(|v| {
            v.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
        })
        .collect()
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    init_logger(&cli)?;

    let prepare = GitPreparer;
    let auth = GitRemoteAuth {
        username: cli.username.clone(),
        password: cli.password.clone(),
    };
    let mut repo = prepare.open_or_clone_repository(&cli.target_path, cli.url.as_deref(), &auth)?;

    prepare.prepare(
        GitPrepareOptions {
            no_fetch: cli.no_fetch,
            no_normalize: cli.no_normalize,
            allow_shallow: cli.allow_shallow,
            target_branch: cli.branch.clone(),
            target_commit: cli.commit.clone(),
            auth,
        },
        &mut repo,
    )?;

    let provider = ConfigurationProvider;
    let config = provider.provide(
        &cli.target_path,
        cli.config.as_deref(),
        parse_override(&cli.override_config),
    )?;

    if cli.show_config {
        println!("{}", serde_json::to_string_pretty(&config)?);
        return Ok(());
    }

    let ctx = GitVersionContext::from_repository(repo, config)?;
    let calculator = NextVersionCalculator;
    let semver = calculator.find_version(&ctx)?;

    let vars = VariableProvider.get_variables_for(&semver, &ctx.configuration, 0);

    if let Some(name) = cli.show_variable.as_deref() {
        if let Some(value) = vars.get(name) {
            println!("{value}");
        }
        return Ok(());
    }

    if let Some(format) = cli.format.as_deref() {
        println!(
            "{}",
            gitversion::extensions::format_with(format, &vars, &|k| std::env::var(k).ok())
        );
        return Ok(());
    }

    let outputs = if cli.output.is_empty() {
        vec![OutputType::Json]
    } else {
        cli.output
    };
    for output in outputs {
        match output {
            OutputType::Json => println!("{}", to_json(&vars)?),
            OutputType::File => {
                if let Some(path) = &cli.output_file {
                    write_output_file(path, &vars)?;
                }
            }
            OutputType::Dotenv => {
                if let Some(path) = &cli.output_file {
                    write_dotenv(path, &vars)?;
                }
            }
            OutputType::Buildserver => {
                let agent = detect_build_agent();
                agent.write_integration(
                    &mut |line| {
                        if let Some(line) = line {
                            println!("{line}");
                        }
                    },
                    &vars,
                    ctx.configuration.update_build_number,
                );
            }
        }
    }

    Ok(())
}

fn init_logger(cli: &Cli) -> Result<()> {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(match (cli.diag, cli.verbosity.as_ref()) {
        (true, _) => LevelFilter::Trace,
        (_, Some(Verbosity::Quiet)) => LevelFilter::Error,
        (_, Some(Verbosity::Minimal)) => LevelFilter::Warn,
        (_, Some(Verbosity::Normal)) => LevelFilter::Info,
        (_, Some(Verbosity::Verbose)) => LevelFilter::Debug,
        (_, Some(Verbosity::Diagnostic)) => LevelFilter::Trace,
        _ => LevelFilter::Info,
    });

    if let Some(path) = &cli.log_file {
        let file = File::create(path)?;
        builder.target(Target::Pipe(Box::new(LogWriter(file))));
    }

    builder.init();
    Ok(())
}

struct LogWriter(File);

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;

    #[test]
    fn cli_accepts_new_compatibility_flags() {
        let cli = Cli::try_parse_from([
            "gitversion",
            ".",
            "--url",
            "https://example.com/repo.git",
            "-u",
            "user",
            "-p",
            "secret",
            "-c",
            "abcdef1234567890",
            "-b",
            "main",
            "--no-cache",
            "--showConfig",
            "--verbosity",
            "diagnostic",
            "--diag",
            "--log-file",
            "gitversion.log",
        ])
        .expect("parse cli");

        assert_eq!(cli.url.as_deref(), Some("https://example.com/repo.git"));
        assert_eq!(cli.username.as_deref(), Some("user"));
        assert_eq!(cli.password.as_deref(), Some("secret"));
        assert_eq!(cli.commit.as_deref(), Some("abcdef1234567890"));
        assert_eq!(cli.branch.as_deref(), Some("main"));
        assert!(cli.no_cache);
        assert!(cli.show_config);
        assert!(cli.diag);
        assert_eq!(
            cli.log_file
                .as_ref()
                .map(|path| path.to_string_lossy().to_string())
                .as_deref(),
            Some("gitversion.log")
        );
    }
}
