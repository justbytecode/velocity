//! Astro project template

use std::path::Path;

use crate::core::VelocityResult;
use crate::templates::Template;

/// Astro template
pub struct AstroTemplate {
    typescript: bool,
}

impl AstroTemplate {
    pub fn new(typescript: bool) -> Self {
        Self { typescript }
    }
}

impl Template for AstroTemplate {
    fn name(&self) -> &str {
        "astro"
    }

    fn generate(&self, target: &Path) -> VelocityResult<()> {
        std::fs::create_dir_all(target.join("src/pages"))?;
        std::fs::create_dir_all(target.join("src/layouts"))?;
        std::fs::create_dir_all(target.join("src/components"))?;
        std::fs::create_dir_all(target.join("public"))?;

        // package.json
        let package_json = if self.typescript {
            serde_json::json!({
                "name": target.file_name().unwrap().to_str().unwrap(),
                "version": "0.1.0",
                "private": true,
                "type": "module",
                "scripts": {
                    "dev": "astro dev",
                    "build": "astro build",
                    "preview": "astro preview"
                },
                "dependencies": {
                    "astro": "^4.0.0"
                },
                "devDependencies": {
                    "typescript": "^5.3.0"
                }
            })
        } else {
            serde_json::json!({
                "name": target.file_name().unwrap().to_str().unwrap(),
                "version": "0.1.0",
                "private": true,
                "type": "module",
                "scripts": {
                    "dev": "astro dev",
                    "build": "astro build",
                    "preview": "astro preview"
                },
                "dependencies": {
                    "astro": "^4.0.0"
                }
            })
        };
        std::fs::write(
            target.join("package.json"),
            serde_json::to_string_pretty(&package_json)?,
        )?;

        // astro.config.mjs
        let astro_config = r#"import { defineConfig } from 'astro/config';

export default defineConfig({});
"#;
        std::fs::write(target.join("astro.config.mjs"), astro_config)?;

        // src/pages/index.astro
        let index_page = r#"---
import Layout from '../layouts/Layout.astro';
---

<Layout title="Welcome to Astro">
  <main>
    <h1>Velocity + <span class="gradient">Astro</span></h1>
    <p>Build fast websites, faster.</p>
  </main>
</Layout>

<style>
  main {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    text-align: center;
  }

  h1 {
    font-size: 4rem;
    margin-bottom: 1rem;
  }

  .gradient {
    background: linear-gradient(90deg, #ff5d00, #ff00d4);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  p {
    font-size: 1.5rem;
    color: #666;
  }
</style>
"#;
        std::fs::write(target.join("src/pages/index.astro"), index_page)?;

        // src/layouts/Layout.astro
        let layout = r#"---
interface Props {
  title: string;
}

const { title } = Astro.props;
---

<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width" />
    <title>{title}</title>
  </head>
  <body>
    <slot />
  </body>
</html>

<style is:global>
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: linear-gradient(135deg, #13151a 0%, #1a1c25 100%);
    color: white;
    min-height: 100vh;
  }
</style>
"#;
        std::fs::write(target.join("src/layouts/Layout.astro"), layout)?;

        // TypeScript config
        if self.typescript {
            let tsconfig = serde_json::json!({
                "extends": "astro/tsconfigs/strict"
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
dist/
.astro/

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
"#;
        std::fs::write(target.join(".gitignore"), gitignore)?;

        Ok(())
    }
}
