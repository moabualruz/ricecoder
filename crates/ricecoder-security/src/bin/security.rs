use std::path::PathBuf;

use clap::{Parser, Subcommand};
use ricecoder_security::*;
use tokio;

#[derive(Parser)]
#[command(name = "ricecoder-security")]
#[command(about = "Security testing and compliance validation for RiceCoder")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan for vulnerabilities
    Scan {
        /// Path to scan
        #[arg(short, long)]
        path: PathBuf,

        /// Scan type
        #[arg(short, long, default_value = "all")]
        scan_type: String,
    },
    /// Run penetration tests
    Pentest {
        /// Target URL
        #[arg(short, long)]
        url: String,
    },
    /// Check compliance
    Compliance {
        /// Compliance standard
        #[arg(short, long, default_value = "all")]
        standard: String,
    },
    /// Run security tests
    Test {
        /// Test type
        #[arg(short, long, default_value = "all")]
        test_type: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { path, scan_type } => {
            run_security_scan(path, scan_type).await?;
        }
        Commands::Pentest { url } => {
            run_penetration_test(url).await?;
        }
        Commands::Compliance { standard } => {
            run_compliance_check(standard).await?;
        }
        Commands::Test { test_type } => {
            run_security_tests(test_type).await?;
        }
    }

    Ok(())
}

async fn run_security_scan(path: PathBuf, scan_type: String) -> anyhow::Result<()> {
    let scanner = DefaultVulnerabilityScanner::new();

    match scan_type.as_str() {
        "dependencies" | "all" => {
            let result = scanner.scan_dependencies(&path).await?;
            println!(
                "Dependency scan completed. Found {} vulnerabilities",
                result.vulnerabilities.len()
            );
        }
        "code" | "all" => {
            let result = scanner.scan_code(&path).await?;
            println!(
                "Code scan completed. Found {} issues in {} files",
                result.issues.len(),
                result.files_scanned
            );
        }
        "config" | "all" => {
            let result = scanner.scan_config(&path).await?;
            println!(
                "Config scan completed. Found {} issues in {} files",
                result.issues.len(),
                result.files_scanned
            );
        }
        "licenses" | "all" => {
            let result = scanner.scan_licenses(&path).await?;
            println!(
                "License scan completed. Found {} incompatible licenses",
                result.incompatible_licenses.len()
            );
        }
        _ => {
            println!("Unknown scan type: {}", scan_type);
        }
    }

    Ok(())
}

async fn run_penetration_test(url: String) -> anyhow::Result<()> {
    let tester = DefaultPenetrationTester::new();

    let results = tester.run_full_penetration_test(&url).await?;

    println!(
        "Penetration test completed. Found {} vulnerabilities",
        results
            .iter()
            .map(|r| r.vulnerabilities_found.len())
            .sum::<usize>()
    );

    for result in results {
        if !result.vulnerabilities_found.is_empty() {
            println!(
                "Test: {} - Found {} vulnerabilities",
                result.test_name,
                result.vulnerabilities_found.len()
            );
        }
    }

    Ok(())
}

async fn run_compliance_check(standard: String) -> anyhow::Result<()> {
    let checker = DefaultComplianceChecker::new();

    match standard.as_str() {
        "soc2" | "all" => {
            let result = checker.check_soc2_compliance().await?;
            println!(
                "SOC 2 compliance: {:.1}% ({})",
                result.score,
                if result.passed { "PASSED" } else { "FAILED" }
            );
        }
        "gdpr" | "all" => {
            let result = checker.check_gdpr_compliance().await?;
            println!(
                "GDPR compliance: {:.1}% ({})",
                result.score,
                if result.passed { "PASSED" } else { "FAILED" }
            );
        }
        "hipaa" | "all" => {
            let result = checker.check_hipaa_compliance().await?;
            println!(
                "HIPAA compliance: {:.1}% ({})",
                result.score,
                if result.passed { "PASSED" } else { "FAILED" }
            );
        }
        "owasp" | "all" => {
            let result = checker.check_owasp_compliance().await?;
            println!(
                "OWASP compliance: {:.1}% ({})",
                result.score,
                if result.passed { "PASSED" } else { "FAILED" }
            );
        }
        _ => {
            println!("Unknown compliance standard: {}", standard);
        }
    }

    Ok(())
}

async fn run_security_tests(test_type: String) -> anyhow::Result<()> {
    let validator = DefaultSecurityValidator::new();

    match test_type.as_str() {
        "input" | "all" => {
            let malicious_inputs = vec![
                "<script>alert('xss')</script>",
                "../../../etc/passwd",
                "' OR '1'='1",
            ];

            for input in malicious_inputs {
                let result = validator.validate_input(input).await?;
                println!(
                    "Input validation for '{}': {}",
                    input,
                    if result.is_safe { "SAFE" } else { "BLOCKED" }
                );
            }
        }
        "auth" | "all" => {
            let result = validator
                .validate_authentication("user", "password")
                .await?;
            println!(
                "Authentication test: {}",
                if result.success { "PASSED" } else { "FAILED" }
            );
        }
        "encryption" | "all" => {
            let result = validator.encrypt_data("sensitive data").await?;
            println!(
                "Encryption test: {}",
                if result.success { "PASSED" } else { "FAILED" }
            );
        }
        _ => {
            println!("Unknown test type: {}", test_type);
        }
    }

    Ok(())
}
