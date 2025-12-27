//! Solid project template

use std::path::Path;

use crate::core::VelocityResult;
use crate::templates::Template;

/// Solid template
pub struct SolidTemplate {
    typescript: bool,
}

impl SolidTemplate {
    pub fn new(typescript: bool) -> Self {
        Self { typescript }
    }

    fn ext(&self) -> &str {
        if self.typescript { "tsx" } else { "jsx" }
    }
}

impl Template for SolidTemplate {
    fn name(&self) -> &str {
        "solid"
    }

    fn generate(&self, target: &Path) -> VelocityResult<()> {
        std::fs::create_dir_all(target.join("src"))?;
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
                    "preview": "vite preview"
                },
                "dependencies": {
                    "solid-js": "^1.8.0"
                },
                "devDependencies": {
                    "typescript": "^5.3.0",
                    "vite": "^5.0.0",
                    "vite-plugin-solid": "^2.8.0"
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
                    "solid-js": "^1.8.0"
                },
                "devDependencies": {
                    "vite": "^5.0.0",
                    "vite-plugin-solid": "^2.8.0"
                }
            })
        };
        std::fs::write(
            target.join("package.json"),
            serde_json::to_string_pretty(&package_json)?,
        )?;

        // vite.config
        let vite_config = r#"import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'

export default defineConfig({
  plugins: [solid()],
})
"#;
        std::fs::write(
            target.join(if self.typescript { "vite.config.ts" } else { "vite.config.js" }),
            vite_config,
        )?;

        // index.html
        let index_html = format!(r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Solid App</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/index.{}"></script>
  </body>
</html>
"#, self.ext());
        std::fs::write(target.join("index.html"), index_html)?;

        // src/index.tsx
        let index = r#"import { render } from 'solid-js/web'
import App from './App'
import './index.css'

render(() => <App />, document.getElementById('root')!)
"#;
        std::fs::write(target.join("src").join(format!("index.{}", self.ext())), index)?;

        // src/App.tsx
        let app = if self.typescript {
            r#"import { createSignal, Component } from 'solid-js'

const App: Component = () => {
  const [count, setCount] = createSignal<number>(0)

  return (
    <div class="app">
      <h1>Velocity + Solid</h1>
      <div class="card">
        <button onClick={() => setCount((c) => c + 1)}>
          count is {count()}
        </button>
      </div>
    </div>
  )
}

export default App
"#
        } else {
            r#"import { createSignal } from 'solid-js'

function App() {
  const [count, setCount] = createSignal(0)

  return (
    <div class="app">
      <h1>Velocity + Solid</h1>
      <div class="card">
        <button onClick={() => setCount((c) => c + 1)}>
          count is {count()}
        </button>
      </div>
    </div>
  )
}

export default App
"#
        };
        std::fs::write(target.join("src").join(format!("App.{}", self.ext())), app)?;

        // src/index.css
        let css = r#"* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: linear-gradient(135deg, #2c4f7c 0%, #446b9e 100%);
  min-height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
}

.app {
  text-align: center;
  color: white;
}

.app h1 {
  font-size: 3rem;
  margin-bottom: 2rem;
}

.card {
  padding: 2rem;
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
        std::fs::write(target.join("src").join("index.css"), css)?;

        // TypeScript config
        if self.typescript {
            let tsconfig = serde_json::json!({
                "compilerOptions": {
                    "target": "ESNext",
                    "module": "ESNext",
                    "moduleResolution": "bundler",
                    "allowSyntheticDefaultImports": true,
                    "esModuleInterop": true,
                    "jsx": "preserve",
                    "jsxImportSource": "solid-js",
                    "types": ["vite/client"],
                    "noEmit": true,
                    "isolatedModules": true,
                    "strict": true
                },
                "include": ["src"]
            });
            std::fs::write(
                target.join("tsconfig.json"),
                serde_json::to_string_pretty(&tsconfig)?,
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
