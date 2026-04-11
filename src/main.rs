use std::collections::HashMap;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use color_eyre::eyre::Result;
use gitversion::agents::detect_build_agent;
use gitversion::calculation::next_version::NextVersionCalculator;
use gitversion::config::provider::ConfigurationProvider;
use gitversion::context::GitVersionContext;
use gitversion::git::git2_impl::repository::Git2Repository;
use gitversion::output::dotenv::write_dotenv;
use gitversion::output::file::write_output_file;
use gitversion::output::json::to_json;
use gitversion::output::variable_provider::VariableProvider;
use gitversion::prepare::{GitPrepareOptions, GitPreparer};

#[derive(Debug, Clone, ValueEnum)]
enum OutputType {
    Json,
    File,
    Buildserver,
    Dotenv,
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
    env_logger::init();

    let cli = Cli::parse();
    let mut repo = Git2Repository::open(&cli.target_path)?;

    let prepare = GitPreparer::default();
    prepare.prepare(
        GitPrepareOptions {
            no_fetch: cli.no_fetch,
            no_normalize: cli.no_normalize,
            allow_shallow: cli.allow_shallow,
        },
        &mut repo,
    )?;

    let provider = ConfigurationProvider::default();
    let config = provider.provide(
        &cli.target_path,
        cli.config.as_deref(),
        parse_override(&cli.override_config),
    )?;

    let ctx = GitVersionContext::from_repository(repo, config)?;
    let calculator = NextVersionCalculator::default();
    let semver = calculator.find_version(&ctx)?;

    let vars = VariableProvider::default().get_variables_for(&semver, &ctx.configuration, 0);

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
