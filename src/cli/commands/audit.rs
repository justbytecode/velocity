//! velocity audit - Security audit command

use std::path::Path;
use clap::Args;

use crate::cli::output;
use crate::core::{VelocityResult, VelocityError, PackageJson};
use crate::security::{EcosystemAnalyzer, SupplyChainGuard, SecurityAnalysis, RiskLevel, SecurityLevel};

#[derive(Args)]
pub struct AuditArgs {
    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Only show high-risk packages
    #[arg(long)]
    pub high_only: bool,

    /// Fix vulnerabilities automatically (where possible)
    #[arg(long)]
    pub fix: bool,

    /// Include dev dependencies
    #[arg(long)]
    pub include_dev: bool,
}

pub async fn execute(args: AuditArgs, json_output: bool) -> VelocityResult<()> {
    let cwd = std::env::current_dir()?;
    
    // Load package.json
    let pkg_json_path = cwd.join("package.json");
    if !pkg_json_path.exists() {
        if json_output {
            println!(r#"{{"error": "No package.json found"}}"#);
        } else {
            output::error("No package.json found. Run 'velocity init' first.");
        }
        return Err(VelocityError::NotInitialized);
    }

    let pkg = PackageJson::load(&cwd)?;
    
    if !json_output {
        output::info("Velocity Security Audit");
        output::divider();
        println!();
    }

    let mut results = AuditResults::default();

    // Audit direct dependencies
    if !json_output {
        println!("📦 Scanning dependencies...\n");
    }

    // Collect all dependencies
    let mut deps: Vec<(String, String, bool)> = pkg.dependencies
        .iter()
        .map(|(k, v)| (k.clone(), v.clone(), false))
        .collect();

    if args.include_dev {
        deps.extend(
            pkg.dev_dependencies
                .iter()
                .map(|(k, v)| (k.clone(), v.clone(), true))
        );
    }

    for (name, version, is_dev) in &deps {
        // Supply chain analysis
        let analysis = SupplyChainGuard::analyze(name);
        
        // Ecosystem categorization
        let category = EcosystemAnalyzer::categorize(name);
        let security_level = EcosystemAnalyzer::security_level(name);
        
        // Record results
        let pkg_result = PackageAuditResult {
            name: name.clone(),
            version: version.clone(),
            is_dev: *is_dev,
            category: format!("{:?}", category),
            security_level: format!("{:?}", security_level),
            risk_level: analysis.risk_level,
            typosquat_warning: analysis.typosquat_warning.as_ref().map(|w| w.similar_to.clone()),
            recommendations: analysis.recommendations.clone(),
            requires_script_confirmation: EcosystemAnalyzer::requires_script_confirmation(name),
        };

        // Show warnings
        if !json_output {
            if let Some(ref warning) = analysis.typosquat_warning {
                results.typosquat_warnings += 1;
                println!("  🚨 {} - Possible typosquat of '{}'", 
                    name, warning.similar_to);
            }

            if analysis.risk_level == RiskLevel::High {
                results.high_risk += 1;
                if !args.high_only {
                    println!("  ⚠️  {} - High risk package", name);
                }
            } else if analysis.risk_level == RiskLevel::Medium && !args.high_only {
                results.medium_risk += 1;
                println!("  ⚡ {} - Medium risk package", name);
            }

            // Ecosystem warnings
            if let Some(warning) = EcosystemAnalyzer::security_warning(name) {
                if security_level >= SecurityLevel::Elevated {
                    println!("  {}", warning);
                }
            }
        }

        results.packages.push(pkg_result);
    }

    // Summary
    if json_output {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!();
        output::divider();
        println!();
        println!("📊 Audit Summary:");
        println!("   Total packages scanned: {}", results.packages.len());
        println!("   High risk:              {}", results.high_risk);
        println!("   Medium risk:            {}", results.medium_risk);
        println!("   Typosquat warnings:     {}", results.typosquat_warnings);
        println!();

        // Ecosystem breakdown
        let web3_count = results.packages.iter()
            .filter(|p| EcosystemAnalyzer::is_web3(&p.name))
            .count();
        let ai_count = results.packages.iter()
            .filter(|p| EcosystemAnalyzer::is_ai(&p.name))
            .count();

        if web3_count > 0 || ai_count > 0 {
            println!("🔗 Ecosystem Breakdown:");
            if web3_count > 0 {
                println!("   Web3/Blockchain packages: {}", web3_count);
            }
            if ai_count > 0 {
                println!("   AI/ML packages:           {}", ai_count);
            }
            println!();
        }

        if results.high_risk > 0 {
            output::warning(&format!(
                "{} high-risk package(s) detected. Review carefully before deployment.",
                results.high_risk
            ));
        } else if results.medium_risk > 0 {
            output::info(&format!(
                "{} medium-risk package(s). Consider reviewing.",
                results.medium_risk
            ));
        } else {
            output::success("No high-risk packages detected.");
        }
    }

    Ok(())
}

#[derive(Debug, Default, serde::Serialize)]
struct AuditResults {
    packages: Vec<PackageAuditResult>,
    high_risk: usize,
    medium_risk: usize,
    typosquat_warnings: usize,
}

#[derive(Debug, serde::Serialize)]
struct PackageAuditResult {
    name: String,
    version: String,
    is_dev: bool,
    category: String,
    security_level: String,
    risk_level: RiskLevel,
    typosquat_warning: Option<String>,
    recommendations: Vec<String>,
    requires_script_confirmation: bool,
}
