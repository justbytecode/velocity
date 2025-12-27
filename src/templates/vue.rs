//! Vue project template

use std::path::Path;

use crate::core::VelocityResult;
use crate::templates::Template;

/// Vue template
pub struct VueTemplate {
    typescript: bool,
}

impl VueTemplate {
    pub fn new(typescript: bool) -> Self {
        Self { typescript }
    }
}

impl Template for VueTemplate {
    fn name(&self) -> &str {
        "vue"
    }

    fn generate(&self, target: &Path) -> VelocityResult<()> {
        std::fs::create_dir_all(target.join("src"))?;
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
                    "dev": "vite",
                    "build": "vue-tsc && vite build",
                    "preview": "vite preview"
                },
                "dependencies": {
                    "vue": "^3.4.0"
                },
                "devDependencies": {
                    "@vitejs/plugin-vue": "^5.0.0",
                    "typescript": "^5.3.0",
                    "vite": "^5.0.0",
                    "vue-tsc": "^1.8.0"
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
                "dependencies": {
                    "vue": "^3.4.0"
                },
                "devDependencies": {
                    "@vitejs/plugin-vue": "^5.0.0",
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
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
})
"#;
        std::fs::write(
            target.join(if self.typescript { "vite.config.ts" } else { "vite.config.js" }),
            vite_config,
        )?;

        // index.html
        let index_html = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Vue App</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
"#;
        std::fs::write(target.join("index.html"), index_html)?;

        // src/main.ts
        let main = r#"import { createApp } from 'vue'
import App from './App.vue'
import './style.css'

createApp(App).mount('#app')
"#;
        std::fs::write(
            target.join("src").join(if self.typescript { "main.ts" } else { "main.js" }),
            main,
        )?;

        // src/App.vue
        let app = if self.typescript {
            r#"<script setup lang="ts">
import { ref } from 'vue'

const count = ref<number>(0)
</script>

<template>
  <div class="app">
    <h1>Velocity + Vue</h1>
    <div class="card">
      <button @click="count++">count is {{ count }}</button>
    </div>
  </div>
</template>

<style scoped>
.app {
  text-align: center;
}
.card {
  padding: 2rem;
}
</style>
"#
        } else {
            r#"<script setup>
import { ref } from 'vue'

const count = ref(0)
</script>

<template>
  <div class="app">
    <h1>Velocity + Vue</h1>
    <div class="card">
      <button @click="count++">count is {{ count }}</button>
    </div>
  </div>
</template>

<style scoped>
.app {
  text-align: center;
}
.card {
  padding: 2rem;
}
</style>
"#
        };
        std::fs::write(target.join("src").join("App.vue"), app)?;

        // src/style.css
        let css = r#"* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: linear-gradient(135deg, #42b883 0%, #35495e 100%);
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
        std::fs::write(target.join("src").join("style.css"), css)?;

        // TypeScript config
        if self.typescript {
            let tsconfig = serde_json::json!({
                "compilerOptions": {
                    "target": "ES2020",
                    "useDefineForClassFields": true,
                    "module": "ESNext",
                    "lib": ["ES2020", "DOM", "DOM.Iterable"],
                    "skipLibCheck": true,
                    "moduleResolution": "bundler",
                    "allowImportingTsExtensions": true,
                    "resolveJsonModule": true,
                    "isolatedModules": true,
                    "noEmit": true,
                    "jsx": "preserve",
                    "strict": true,
                    "noUnusedLocals": true,
                    "noUnusedParameters": true,
                    "noFallthroughCasesInSwitch": true
                },
                "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue"],
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
                    "moduleResolution": "bundler",
                    "allowSyntheticDefaultImports": true
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
.env
.env.local
"#;
        std::fs::write(target.join(".gitignore"), gitignore)?;

        Ok(())
    }
}
