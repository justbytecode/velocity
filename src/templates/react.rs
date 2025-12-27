//! React project template

use std::path::Path;

use crate::core::VelocityResult;
use crate::templates::Template;

/// React template
pub struct ReactTemplate {
    typescript: bool,
}

impl ReactTemplate {
    pub fn new(typescript: bool) -> Self {
        Self { typescript }
    }

    fn ext(&self) -> &str {
        if self.typescript { "tsx" } else { "jsx" }
    }
}

impl Template for ReactTemplate {
    fn name(&self) -> &str {
        "react"
    }

    fn generate(&self, target: &Path) -> VelocityResult<()> {
        // Create directories
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
                    "build": "tsc && vite build",
                    "preview": "vite preview"
                },
                "dependencies": {
                    "react": "^18.2.0",
                    "react-dom": "^18.2.0"
                },
                "devDependencies": {
                    "@types/react": "^18.2.0",
                    "@types/react-dom": "^18.2.0",
                    "@vitejs/plugin-react": "^4.2.0",
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
                "dependencies": {
                    "react": "^18.2.0",
                    "react-dom": "^18.2.0"
                },
                "devDependencies": {
                    "@vitejs/plugin-react": "^4.2.0",
                    "vite": "^5.0.0"
                }
            })
        };
        std::fs::write(
            target.join("package.json"),
            serde_json::to_string_pretty(&package_json)?,
        )?;

        // vite.config
        let vite_config = if self.typescript {
            r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
})
"#
        } else {
            r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
})
"#
        };
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
    <title>React App</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.{}"></script>
  </body>
</html>
"#, self.ext());
        std::fs::write(target.join("index.html"), index_html)?;

        // src/main.tsx
        let main = if self.typescript {
            r#"import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
"#
        } else {
            r#"import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
"#
        };
        std::fs::write(target.join("src").join(format!("main.{}", self.ext())), main)?;

        // src/App.tsx
        let app = if self.typescript {
            r#"import { useState } from 'react'

function App() {
  const [count, setCount] = useState<number>(0)

  return (
    <div className="app">
      <h1>Velocity + React</h1>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
      </div>
    </div>
  )
}

export default App
"#
        } else {
            r#"import { useState } from 'react'

function App() {
  const [count, setCount] = useState(0)

  return (
    <div className="app">
      <h1>Velocity + React</h1>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
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
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
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
                    "target": "ES2020",
                    "useDefineForClassFields": true,
                    "lib": ["ES2020", "DOM", "DOM.Iterable"],
                    "module": "ESNext",
                    "skipLibCheck": true,
                    "moduleResolution": "bundler",
                    "allowImportingTsExtensions": true,
                    "resolveJsonModule": true,
                    "isolatedModules": true,
                    "noEmit": true,
                    "jsx": "react-jsx",
                    "strict": true,
                    "noUnusedLocals": true,
                    "noUnusedParameters": true,
                    "noFallthroughCasesInSwitch": true
                },
                "include": ["src"],
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
        let gitignore = r#"# Dependencies
node_modules/

# Build
dist/
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
"#;
        std::fs::write(target.join(".gitignore"), gitignore)?;

        Ok(())
    }
}
