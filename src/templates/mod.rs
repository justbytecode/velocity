//! Project templates for framework scaffolding

mod react;
mod next;
mod vue;
mod svelte;
mod solid;
mod astro;

use std::path::Path;

use crate::core::{VelocityResult, VelocityError};

pub use react::ReactTemplate;
pub use next::NextTemplate;
pub use vue::VueTemplate;
pub use svelte::SvelteTemplate;
pub use solid::SolidTemplate;
pub use astro::AstroTemplate;

/// Template trait for project scaffolding
pub trait Template {
    /// Get the template name
    fn name(&self) -> &str;

    /// Generate project files
    fn generate(&self, target: &Path) -> VelocityResult<()>;
}

/// Template manager
pub struct TemplateManager;

impl TemplateManager {
    /// Create a new template manager
    pub fn new() -> Self {
        Self
    }

    /// Get a template by framework name
    pub fn get_template(&self, framework: &str, typescript: bool) -> VelocityResult<Box<dyn Template>> {
        match framework.to_lowercase().as_str() {
            "react" => Ok(Box::new(ReactTemplate::new(typescript))),
            "next" => Ok(Box::new(NextTemplate::new(typescript))),
            "vue" => Ok(Box::new(VueTemplate::new(typescript))),
            "svelte" => Ok(Box::new(SvelteTemplate::new(typescript))),
            "solid" => Ok(Box::new(SolidTemplate::new(typescript))),
            "astro" => Ok(Box::new(AstroTemplate::new(typescript))),
            _ => Err(VelocityError::template(format!(
                "Unknown framework: {}",
                framework
            ))),
        }
    }

    /// List available templates
    pub fn list(&self) -> Vec<&str> {
        vec!["react", "next", "vue", "svelte", "solid", "astro"]
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}
