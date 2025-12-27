//! Svelte project template

use std::path::Path;

use crate::core::VelocityResult;
use crate::templates::Template;

/// Svelte template
pub struct SvelteTemplate {
    typescript: bool,
}

impl SvelteTemplate {
    pub fn new(typescript: bool) -> Self {
        Self { typescript }
    }
}

impl Template for SvelteTemplate {
    fn name(&self) -> &str {
        "svelte"
    }

    fn generate(&self, target: &Path) -> VelocityResult<()> {
        std::fs::create_dir_all(target.join("src"))?;
        std::fs::create_dir_all(target.join("src/lib"))?;
        std::fs::create_dir_all(target.join("public"))?;

        // package.json
        let package_json = if self.typescript {
            serde_json::json!({
                "name": target.file_name().unwrap().to_str().unwrap(),
                "version": "0.1.0",
                "private": true,
                "type": "module",
                "scripts": {
                    "dev": "vite",
                    "build": "vite build",
                    "preview": "vite preview",
                    "check": "svelte-check --tsconfig ./tsconfig.json"
                },
                "devDependencies": {
                    "@sveltejs/vite-plugin-svelte": "^3.0.0",
                    "svelte": "^4.2.0",
                    "svelte-check": "^3.6.0",
                    "tslib": "^2.6.0",
                    "typescript": "^5.3.0",
                    "vite": "^5.0.0"
                }
            })
        } else {
            serde_json::json!({
                "name": target.file_name().unwrap().to_str().unwrap(),
                "version": "0.1.0",
                "private": true,
                "type": "module",
                "scripts": {
                    "dev": "vite",
                    "build": "vite build",
                    "preview": "vite preview"
                },
                "devDependencies": {
                    "@sveltejs/vite-plugin-svelte": "^3.0.0",
                    "svelte": "^4.2.0",
                    "vite": "^5.0.0"
                }
            })
        };
        std::fs::write(
            target.join("package.json"),
            serde_json::to_string_pretty(&package_json)?,
        )?;

        // vite.config
        let vite_config = r#"import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  plugins: [svelte()],
})
"#;
        std::fs::write(
            target.join(if self.typescript { "vite.config.ts" } else { "vite.config.js" }),
            vite_config,
        )?;

        // svelte.config.js
        let svelte_config = r#"import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

export default {
  preprocess: vitePreprocess(),
}
"#;
        std::fs::write(target.join("svelte.config.js"), svelte_config)?;

        // index.html
        let index_html = format!(r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Svelte App</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.{}"></script>
  </body>
</html>
"#, if self.typescript { "ts" } else { "js" });
        std::fs::write(target.join("index.html"), index_html)?;

        // src/main.ts
        let main = r#"import './app.css'
import App from './App.svelte'

const app = new App({
  target: document.getElementById('app')!,
})

export default app
"#;
        std::fs::write(
            target.join("src").join(if self.typescript { "main.ts" } else { "main.js" }),
            main,
        )?;

        // src/App.svelte
        let app = if self.typescript {
            r#"<script lang="ts">
  let count: number = 0
</script>

<main>
  <h1>Velocity + Svelte</h1>
  <div class="card">
    <button on:click={() => count++}>
      count is {count}
    </button>
  </div>
</main>

<style>
  main {
    text-align: center;
  }
  .card {
    padding: 2rem;
  }
</style>
"#
        } else {
            r#"<script>
  let count = 0
</script>

<main>
  <h1>Velocity + Svelte</h1>
  <div class="card">
    <button on:click={() => count++}>
      count is {count}
    </button>
  </div>
</main>

<style>
  main {
    text-align: center;
  }
  .card {
    padding: 2rem;
  }
</style>
"#
        };
        std::fs::write(target.join("src").join("App.svelte"), app)?;

        // src/app.css
        let css = r#"* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: linear-gradient(135deg, #ff3e00 0%, #ff7700 100%);
  min-height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
  color: white;
}

button {
  padding: 1rem 2rem;
  font-size: 1rem;
  border: none;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.2);
  color: white;
  cursor: pointer;
  transition: background 0.3s;
}

button:hover {
  background: rgba(255, 255, 255, 0.3);
}
"#;
        std::fs::write(target.join("src").join("app.css"), css)?;

        // TypeScript config
        if self.typescript {
            let tsconfig = serde_json::json!({
                "extends": "@tsconfig/svelte/tsconfig.json",
                "compilerOptions": {
                    "target": "ESNext",
                    "useDefineForClassFields": true,
                    "module": "ESNext",
                    "resolveJsonModule": true,
                    "allowJs": true,
                    "checkJs": true,
                    "isolatedModules": true
                },
                "include": ["src/**/*.ts", "src/**/*.js", "src/**/*.svelte"],
                "references": [{ "path": "./tsconfig.node.json" }]
            });
            std::fs::write(
                target.join("tsconfig.json"),
                serde_json::to_string_pretty(&tsconfig)?,
            )?;

            let tsconfig_node = serde_json::json!({
                "compilerOptions": {
                    "composite": true,
                    "skipLibCheck": true,
                    "module": "ESNext",
                    "moduleResolution": "bundler"
                },
                "include": ["vite.config.ts"]
            });
            std::fs::write(
                target.join("tsconfig.node.json"),
                serde_json::to_string_pretty(&tsconfig_node)?,
            )?;
        }

        // .gitignore
        let gitignore = r#"node_modules/
dist/
velocity.lock
.idea/
.vscode/
*.log
"#;
        std::fs::write(target.join(".gitignore"), gitignore)?;

        Ok(())
    }
}
