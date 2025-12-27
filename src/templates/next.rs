//! Next.js project template

use std::path::Path;

use crate::core::VelocityResult;
use crate::templates::Template;

/// Next.js template
pub struct NextTemplate {
    typescript: bool,
}

impl NextTemplate {
    pub fn new(typescript: bool) -> Self {
        Self { typescript }
    }

    fn ext(&self) -> &str {
        if self.typescript { "tsx" } else { "jsx" }
    }
}

impl Template for NextTemplate {
    fn name(&self) -> &str {
        "next"
    }

    fn generate(&self, target: &Path) -> VelocityResult<()> {
        // Create directories
        std::fs::create_dir_all(target.join("app"))?;
        std::fs::create_dir_all(target.join("public"))?;

        // package.json
        let package_json = if self.typescript {
            serde_json::json!({
                "name": target.file_name().unwrap().to_str().unwrap(),
                "version": "0.1.0",
                "private": true,
                "scripts": {
                    "dev": "next dev",
                    "build": "next build",
                    "start": "next start",
                    "lint": "next lint"
                },
                "dependencies": {
                    "next": "^14.0.0",
                    "react": "^18.2.0",
                    "react-dom": "^18.2.0"
                },
                "devDependencies": {
                    "@types/node": "^20.0.0",
                    "@types/react": "^18.2.0",
                    "@types/react-dom": "^18.2.0",
                    "typescript": "^5.3.0"
                }
            })
        } else {
            serde_json::json!({
                "name": target.file_name().unwrap().to_str().unwrap(),
                "version": "0.1.0",
                "private": true,
                "scripts": {
                    "dev": "next dev",
                    "build": "next build",
                    "start": "next start",
                    "lint": "next lint"
                },
                "dependencies": {
                    "next": "^14.0.0",
                    "react": "^18.2.0",
                    "react-dom": "^18.2.0"
                }
            })
        };
        std::fs::write(
            target.join("package.json"),
            serde_json::to_string_pretty(&package_json)?,
        )?;

        // next.config
        let next_config = if self.typescript {
            r#"import type { NextConfig } from 'next'

const nextConfig: NextConfig = {
  reactStrictMode: true,
}

export default nextConfig
"#
        } else {
            r#"/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
}

module.exports = nextConfig
"#
        };
        std::fs::write(
            target.join(if self.typescript { "next.config.ts" } else { "next.config.js" }),
            next_config,
        )?;

        // app/layout.tsx
        let layout = if self.typescript {
            r#"import type { Metadata } from 'next'
import './globals.css'

export const metadata: Metadata = {
  title: 'Next.js App',
  description: 'Created with Velocity',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  )
}
"#
        } else {
            r#"import './globals.css'

export const metadata = {
  title: 'Next.js App',
  description: 'Created with Velocity',
}

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  )
}
"#
        };
        std::fs::write(target.join("app").join(format!("layout.{}", self.ext())), layout)?;

        // app/page.tsx
        let page = r#"export default function Home() {
  return (
    <main className="main">
      <h1>Velocity + Next.js</h1>
      <p>Get started by editing <code>app/page.tsx</code></p>
    </main>
  )
}
"#;
        std::fs::write(target.join("app").join(format!("page.{}", self.ext())), page)?;

        // app/globals.css
        let css = r#"* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: linear-gradient(135deg, #0f0f0f 0%, #1a1a2e 100%);
  min-height: 100vh;
  color: white;
}

.main {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  padding: 2rem;
}

.main h1 {
  font-size: 3rem;
  margin-bottom: 1rem;
  background: linear-gradient(90deg, #00d4ff, #7b2ff7);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.main p {
  font-size: 1.2rem;
  color: #888;
}

.main code {
  background: rgba(255, 255, 255, 0.1);
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-family: monospace;
}
"#;
        std::fs::write(target.join("app").join("globals.css"), css)?;

        // TypeScript config
        if self.typescript {
            let tsconfig = serde_json::json!({
                "compilerOptions": {
                    "lib": ["dom", "dom.iterable", "esnext"],
                    "allowJs": true,
                    "skipLibCheck": true,
                    "strict": true,
                    "noEmit": true,
                    "esModuleInterop": true,
                    "module": "esnext",
                    "moduleResolution": "bundler",
                    "resolveJsonModule": true,
                    "isolatedModules": true,
                    "jsx": "preserve",
                    "incremental": true,
                    "plugins": [{ "name": "next" }],
                    "paths": { "@/*": ["./*"] }
                },
                "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
                "exclude": ["node_modules"]
            });
            std::fs::write(
                target.join("tsconfig.json"),
                serde_json::to_string_pretty(&tsconfig)?,
            )?;
        }

        // .gitignore
        let gitignore = r#"# Dependencies
node_modules/

# Build
.next/
out/
build/

# Velocity
velocity.lock

# IDE
.idea/
.vscode/
*.swp

# Logs
*.log

# Environment
.env
.env.local
.env.production.local
"#;
        std::fs::write(target.join(".gitignore"), gitignore)?;

        Ok(())
    }
}
