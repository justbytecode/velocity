//! Supply chain attack detection and typosquatting prevention

use std::collections::HashSet;
use once_cell::sync::Lazy;

/// Known popular packages for typosquatting detection
static POPULAR_PACKAGES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        // Core
        "react", "react-dom", "next", "vue", "svelte", "angular",
        "express", "fastify", "koa", "hono", "nestjs",
        // Utils
        "lodash", "underscore", "ramda", "axios", "ky", "got",
        "moment", "dayjs", "date-fns", "uuid", "nanoid",
        // Build
        "webpack", "vite", "rollup", "esbuild", "parcel", "turbo",
        "typescript", "babel", "eslint", "prettier",
        // Testing
        "jest", "vitest", "mocha", "chai", "cypress", "playwright",
        // DB
        "prisma", "drizzle", "sequelize", "mongoose", "typeorm",
        // Web3
        "ethers", "web3", "viem", "wagmi", "hardhat",
        // AI
        "openai", "langchain", "anthropic", "pinecone",
    ].into_iter().collect()
});

/// Characters commonly swapped in typosquatting
static SIMILAR_CHARS: &[(char, char)] = &[
    ('l', '1'), ('l', 'i'), ('1', 'i'),
    ('o', '0'), ('0', 'o'),
    ('rn', 'm'), ('m', 'rn'),
    ('n', 'm'),
    ('s', '5'),
    ('a', '4'),
    ('e', '3'),
];

/// Suspicious package name patterns
static SUSPICIOUS_PATTERNS: &[&str] = &[
    "-internal",
    "-private",
    "-corp",
    "-company",
    "-test",
    "-dev",
    "-debug",
    "-backup",
    "copy-of-",
    "fork-of-",
    "-clone",
];

/// Supply chain guard for detecting attacks
pub struct SupplyChainGuard;

impl SupplyChainGuard {
    /// Check if a package name might be a typosquat
    pub fn check_typosquat(name: &str) -> Option<TyposquatWarning> {
        let normalized = name.to_lowercase();
        
        for popular in POPULAR_PACKAGES.iter() {
            if *popular == normalized {
                return None; // Exact match, not a typosquat
            }
            
            let distance = Self::levenshtein(&normalized, popular);
            if distance > 0 && distance <= 2 {
                return Some(TyposquatWarning {
                    suspicious: name.to_string(),
                    similar_to: popular.to_string(),
                    distance,
                    severity: if distance == 1 {
                        TyposquatSeverity::High
                    } else {
                        TyposquatSeverity::Medium
                    },
                });
            }
        }
        
        None
    }

    /// Check for suspicious naming patterns
    pub fn check_suspicious_name(name: &str) -> Option<SuspiciousNameWarning> {
        let normalized = name.to_lowercase();
        
        for pattern in SUSPICIOUS_PATTERNS {
            if normalized.contains(pattern) {
                return Some(SuspiciousNameWarning {
                    package: name.to_string(),
                    pattern: pattern.to_string(),
                    reason: format!(
                        "Package name contains '{}' which is commonly used in dependency confusion attacks",
                        pattern
                    ),
                });
            }
        }
        
        None
    }

    /// Full security analysis of a package
    pub fn analyze(name: &str) -> SecurityAnalysis {
        let typosquat = Self::check_typosquat(name);
        let suspicious = Self::check_suspicious_name(name);
        
        let risk_level = if typosquat.as_ref().map(|t| t.severity == TyposquatSeverity::High).unwrap_or(false) {
            RiskLevel::High
        } else if typosquat.is_some() || suspicious.is_some() {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        SecurityAnalysis {
            package: name.to_string(),
            risk_level,
            typosquat_warning: typosquat,
            suspicious_name: suspicious,
            recommendations: Self::get_recommendations(name, &risk_level),
        }
    }

    /// Get recommendations based on analysis
    fn get_recommendations(name: &str, risk: &RiskLevel) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        match risk {
            RiskLevel::High => {
                recommendations.push(format!(
                    "⚠️  CRITICAL: Verify '{}' is the correct package before installing",
                    name
                ));
                recommendations.push("Check the npm registry page for author and download stats".to_string());
                recommendations.push("Consider using the official scoped package if available".to_string());
            }
            RiskLevel::Medium => {
                recommendations.push(format!(
                    "⚡ Verify '{}' is from a trusted source",
                    name
                ));
                recommendations.push("Review package.json scripts before installation".to_string());
            }
            RiskLevel::Low => {}
        }
        
        recommendations
    }

    /// Levenshtein distance for typosquat detection
    fn levenshtein(a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        
        let m = a_chars.len();
        let n = b_chars.len();
        
        if m == 0 { return n; }
        if n == 0 { return m; }
        
        let mut dp = vec![vec![0usize; n + 1]; m + 1];
        
        for i in 0..=m { dp[i][0] = i; }
        for j in 0..=n { dp[0][j] = j; }
        
        for i in 1..=m {
            for j in 1..=n {
                let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
                dp[i][j] = (dp[i - 1][j] + 1)
                    .min(dp[i][j - 1] + 1)
                    .min(dp[i - 1][j - 1] + cost);
            }
        }
        
        dp[m][n]
    }
}

/// Typosquat warning
#[derive(Debug, Clone)]
pub struct TyposquatWarning {
    pub suspicious: String,
    pub similar_to: String,
    pub distance: usize,
    pub severity: TyposquatSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TyposquatSeverity {
    High,   // 1 character difference
    Medium, // 2 character difference
}

/// Suspicious name warning
#[derive(Debug, Clone)]
pub struct SuspiciousNameWarning {
    pub package: String,
    pub pattern: String,
    pub reason: String,
}

/// Risk level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Complete security analysis
#[derive(Debug, Clone)]
pub struct SecurityAnalysis {
    pub package: String,
    pub risk_level: RiskLevel,
    pub typosquat_warning: Option<TyposquatWarning>,
    pub suspicious_name: Option<SuspiciousNameWarning>,
    pub recommendations: Vec<String>,
}

impl SecurityAnalysis {
    /// Check if installation should be blocked
    pub fn should_block(&self) -> bool {
        self.risk_level == RiskLevel::High
    }

    /// Check if warning should be shown
    pub fn should_warn(&self) -> bool {
        self.risk_level != RiskLevel::Low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typosquat_detection() {
        // Should detect typosquat
        let warning = SupplyChainGuard::check_typosquat("reacr");
        assert!(warning.is_some());
        assert_eq!(warning.unwrap().similar_to, "react");

        // Should not flag exact match
        let warning = SupplyChainGuard::check_typosquat("react");
        assert!(warning.is_none());
    }

    #[test]
    fn test_suspicious_patterns() {
        let warning = SupplyChainGuard::check_suspicious_name("lodash-internal");
        assert!(warning.is_some());

        let warning = SupplyChainGuard::check_suspicious_name("lodash");
        assert!(warning.is_none());
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(SupplyChainGuard::levenshtein("react", "reacr"), 1);
        assert_eq!(SupplyChainGuard::levenshtein("lodash", "lodash"), 0);
        assert_eq!(SupplyChainGuard::levenshtein("express", "expres"), 1);
    }
}
