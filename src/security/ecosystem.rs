//! Ecosystem-specific handling for Web3, AI, and security-sensitive packages

use std::collections::HashSet;
use once_cell::sync::Lazy;

/// Categories of packages that require special handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcosystemCategory {
    /// Standard JavaScript/TypeScript packages
    Standard,
    /// Web3/Blockchain packages (ethers, viem, solana, etc.)
    Web3,
    /// AI/ML packages (openai, langchain, etc.)
    AI,
    /// Wallet and cryptographic packages
    Wallet,
    /// Native binary packages
    NativeBinary,
    /// Network-heavy SDKs
    NetworkHeavy,
}

/// Security level for packages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    /// Standard security - normal install
    Standard,
    /// Elevated - warn on install
    Elevated,
    /// High - require explicit confirmation
    High,
    /// Critical - deny scripts by default
    Critical,
}

/// Web3 ecosystem packages
static WEB3_PACKAGES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        // Ethereum/EVM
        "ethers", "web3", "viem", "wagmi", "rainbowkit",
        "hardhat", "@nomiclabs/hardhat-ethers", "@nomiclabs/hardhat-waffle",
        "@openzeppelin/contracts", "@openzeppelin/hardhat-upgrades",
        "thirdweb", "@thirdweb-dev/sdk", "@thirdweb-dev/react",
        "typechain", "@typechain/ethers-v6", "@typechain/hardhat",
        "abitype", "permissionless", "siwe",
        // Solana
        "@solana/web3.js", "@solana/spl-token", "@solana/wallet-adapter-base",
        "@solana/wallet-adapter-react", "@solana/wallet-adapter-wallets",
        "@project-serum/anchor", "@metaplex-foundation/js",
        "@coral-xyz/anchor",
        // Other chains
        "@polkadot/api", "@polkadot/util", "@polkadot/keyring",
        "near-api-js", "@near-js/providers",
        "@cosmjs/stargate", "@cosmjs/proto-signing",
        "aptos", "@aptos-labs/ts-sdk",
        "@mysten/sui.js",
        "algosdk",
        // Wallet adapters
        "@rainbow-me/rainbowkit", "@web3modal/ethereum",
        "@walletconnect/web3-provider", "@metamask/sdk",
    ].into_iter().collect()
});

/// AI/ML ecosystem packages
static AI_PACKAGES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        // AI SDKs
        "openai", "@anthropic-ai/sdk", "cohere-ai", "@mistralai/mistralai",
        "groq-sdk", "replicate", "@huggingface/inference",
        "@google/generative-ai", "google-generativeai",
        // Frameworks
        "langchain", "@langchain/core", "@langchain/openai", "@langchain/anthropic",
        "llamaindex", "crewai",
        // Vector DBs
        "@pinecone-database/pinecone", "weaviate-ts-client",
        "@qdrant/js-client-rest", "chromadb",
        // AI Frontend
        "ai", "@vercel/ai", "@ai-sdk/openai", "@ai-sdk/anthropic",
        // Embeddings
        "@tensorflow/tfjs", "onnxruntime-node", "transformers",
    ].into_iter().collect()
});

/// Wallet/crypto packages requiring elevated security
static WALLET_PACKAGES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "@metamask/sdk", "@walletconnect/web3-provider",
        "@solana/wallet-adapter-base", "@rainbow-me/rainbowkit",
        "ethers", "web3", "@solana/web3.js",
        "@openzeppelin/contracts", "hardhat",
    ].into_iter().collect()
});

/// Packages with native binaries
static NATIVE_BINARY_PACKAGES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        // Database
        "prisma", "@prisma/client", "better-sqlite3", "sqlite3",
        // Crypto
        "argon2", "bcrypt", "node-argon2",
        // Image/Media
        "sharp", "canvas", "node-canvas",
        // System
        "node-gyp", "node-pre-gyp", "prebuild",
        // AI/ML
        "@tensorflow/tfjs-node", "onnxruntime-node",
        // Build tools
        "esbuild", "swc", "@swc/core", "lightningcss",
    ].into_iter().collect()
});

/// Network-heavy packages
static NETWORK_HEAVY_PACKAGES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "openai", "@anthropic-ai/sdk", "replicate",
        "@pinecone-database/pinecone", "weaviate-ts-client",
        "firebase", "@firebase/app", "supabase",
        "@aws-sdk/client-s3", "aws-sdk",
    ].into_iter().collect()
});

/// Ecosystem utilities
pub struct EcosystemAnalyzer;

impl EcosystemAnalyzer {
    /// Categorize a package
    pub fn categorize(name: &str) -> EcosystemCategory {
        let normalized = Self::normalize_name(name);
        
        if WALLET_PACKAGES.contains(normalized.as_str()) {
            EcosystemCategory::Wallet
        } else if WEB3_PACKAGES.contains(normalized.as_str()) {
            EcosystemCategory::Web3
        } else if AI_PACKAGES.contains(normalized.as_str()) {
            EcosystemCategory::AI
        } else if NATIVE_BINARY_PACKAGES.contains(normalized.as_str()) {
            EcosystemCategory::NativeBinary
        } else if NETWORK_HEAVY_PACKAGES.contains(normalized.as_str()) {
            EcosystemCategory::NetworkHeavy
        } else {
            EcosystemCategory::Standard
        }
    }

    /// Get security level for a package
    pub fn security_level(name: &str) -> SecurityLevel {
        let category = Self::categorize(name);
        match category {
            EcosystemCategory::Wallet => SecurityLevel::Critical,
            EcosystemCategory::Web3 => SecurityLevel::High,
            EcosystemCategory::AI => SecurityLevel::Elevated,
            EcosystemCategory::NativeBinary => SecurityLevel::High,
            EcosystemCategory::NetworkHeavy => SecurityLevel::Elevated,
            EcosystemCategory::Standard => SecurityLevel::Standard,
        }
    }

    /// Check if package requires script confirmation
    pub fn requires_script_confirmation(name: &str) -> bool {
        matches!(
            Self::security_level(name),
            SecurityLevel::High | SecurityLevel::Critical
        )
    }

    /// Get security warning message
    pub fn security_warning(name: &str) -> Option<String> {
        let category = Self::categorize(name);
        match category {
            EcosystemCategory::Wallet => Some(format!(
                "⚠️  {} is a wallet-related package. Verify source before use.",
                name
            )),
            EcosystemCategory::Web3 => Some(format!(
                "🔗 {} is a Web3 package. Scripts are disabled by default.",
                name
            )),
            EcosystemCategory::AI => Some(format!(
                "🤖 {} is an AI package. May make network requests.",
                name
            )),
            EcosystemCategory::NativeBinary => Some(format!(
                "💾 {} contains native binaries. Requires build tools.",
                name
            )),
            EcosystemCategory::NetworkHeavy => Some(format!(
                "🌐 {} is network-heavy. Configure API keys securely.",
                name
            )),
            EcosystemCategory::Standard => None,
        }
    }

    /// Normalize package name for lookup
    fn normalize_name(name: &str) -> String {
        name.to_lowercase()
    }

    /// Check if package is Web3 ecosystem
    pub fn is_web3(name: &str) -> bool {
        matches!(
            Self::categorize(name),
            EcosystemCategory::Web3 | EcosystemCategory::Wallet
        )
    }

    /// Check if package is AI ecosystem
    pub fn is_ai(name: &str) -> bool {
        Self::categorize(name) == EcosystemCategory::AI
    }

    /// Get recommended peers for Web3 packages
    pub fn web3_recommended_peers(name: &str) -> Vec<&'static str> {
        match name {
            "wagmi" => vec!["viem", "@tanstack/react-query"],
            "rainbowkit" => vec!["wagmi", "viem"],
            "@solana/wallet-adapter-react" => vec![
                "@solana/wallet-adapter-base",
                "@solana/wallet-adapter-wallets",
                "@solana/web3.js"
            ],
            "hardhat" => vec!["ethers", "@nomicfoundation/hardhat-toolbox"],
            _ => vec![],
        }
    }

    /// Get recommended peers for AI packages
    pub fn ai_recommended_peers(name: &str) -> Vec<&'static str> {
        match name {
            "langchain" => vec!["@langchain/core"],
            "@langchain/openai" => vec!["langchain", "@langchain/core", "openai"],
            "ai" => vec!["@ai-sdk/openai"],
            _ => vec![],
        }
    }
}

/// Template flags for ecosystem support
#[derive(Debug, Clone, Default)]
pub struct TemplateFlags {
    pub web3: bool,
    pub ai: bool,
    pub typescript: bool,
}

impl TemplateFlags {
    /// Get additional dependencies for Web3 flag
    pub fn web3_dependencies(&self) -> Vec<(&'static str, &'static str)> {
        if self.web3 {
            vec![
                ("wagmi", "^2.0.0"),
                ("viem", "^2.0.0"),
                ("@tanstack/react-query", "^5.0.0"),
            ]
        } else {
            vec![]
        }
    }

    /// Get additional dependencies for AI flag
    pub fn ai_dependencies(&self) -> Vec<(&'static str, &'static str)> {
        if self.ai {
            vec![
                ("ai", "^3.0.0"),
                ("@ai-sdk/openai", "^0.0.1"),
            ]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_web3() {
        assert_eq!(EcosystemAnalyzer::categorize("ethers"), EcosystemCategory::Wallet);
        assert_eq!(EcosystemAnalyzer::categorize("wagmi"), EcosystemCategory::Web3);
    }

    #[test]
    fn test_categorize_ai() {
        assert_eq!(EcosystemAnalyzer::categorize("openai"), EcosystemCategory::AI);
        assert_eq!(EcosystemAnalyzer::categorize("langchain"), EcosystemCategory::AI);
    }

    #[test]
    fn test_security_levels() {
        assert_eq!(EcosystemAnalyzer::security_level("ethers"), SecurityLevel::Critical);
        assert_eq!(EcosystemAnalyzer::security_level("openai"), SecurityLevel::Elevated);
        assert_eq!(EcosystemAnalyzer::security_level("lodash"), SecurityLevel::Standard);
    }
}
