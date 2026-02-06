use rensa_core::ScanReport;

pub fn print_report(report: &ScanReport) {
    println!("\n{}", "=".repeat(60));
    println!("Rensa Scan Report");
    println!("{}", "=".repeat(60));
    println!("Path: {}", report.scanned_path.display());
    println!("Duration: {}ms", report.elapsed);
    println!();

    println!("Summary:");
    println!("  Dependency files: {}", report.total_dependency_files);
    println!("  Dependencies: {}", report.total_dependencies);
    println!("  Updates available: {}", report.summary.updates_available);
    println!(
        "  Vulnerabilities: {}",
        report.summary.vulnerabilities_found
    );
    if report.summary.critical_vulnerabilities > 0 {
        println!("    Critical: {}", report.summary.critical_vulnerabilities);
    }
    if report.summary.high_vulnerabilities > 0 {
        println!("    High: {}", report.summary.high_vulnerabilities);
    }
    if report.summary.medium_vulnerabilities > 0 {
        println!("    Medium: {}", report.summary.medium_vulnerabilities);
    }
    if report.summary.low_vulnerabilities > 0 {
        println!("    Low: {}", report.summary.low_vulnerabilities);
    }
    println!();

    if !report.updates.is_empty() {
        println!("Updates:");
        for update in &report.updates {
            println!(
                "  - {} ({} -> {})",
                update.dependency.name, update.current_version, update.latest_version
            );
        }
        println!();
    }

    if !report.vulnerabilities.is_empty() {
        println!("Vulnerabilities:");
        for vuln in &report.vulnerabilities {
            println!("  - [{}] {}", vuln.id, vuln.summary);
            println!("    Severity: {:?}", vuln.severity);
            if !vuln.fixed_versions.is_empty() {
                println!("    Fixed in: {}", vuln.fixed_versions.join(", "));
            }
        }
        println!();
    }

    if !report.warnings.is_empty() {
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {}", warning);
        }
        println!();
    }

    if report.has_critical_vulnerabilities() {
        println!("⚠️  Critical vulnerabilities found!");
    }
    if report.has_updates() {
        println!("📦 Updates available!");
    }
}

pub fn print_json(report: &ScanReport) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}
