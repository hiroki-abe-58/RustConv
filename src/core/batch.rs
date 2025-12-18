//! Batch processing engine

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::converter;
use crate::formats::detect::{detect, Format};

/// Batch job configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// List of jobs to execute
    pub jobs: Vec<BatchJob>,
    /// Continue on error (default: false)
    #[serde(default)]
    pub continue_on_error: bool,
    /// Parallel execution (default: false)
    #[serde(default)]
    pub parallel: bool,
    /// Variables for template substitution
    #[serde(default)]
    pub variables: Option<JsonValue>,
}

/// Individual batch job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJob {
    /// Job name for logging
    pub name: String,
    /// Job type
    #[serde(flatten)]
    pub action: BatchAction,
    /// Condition to run this job (optional)
    #[serde(default)]
    pub condition: Option<String>,
}

/// Batch action types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum BatchAction {
    /// Convert files between formats
    Convert {
        input: String,
        output: String,
        #[serde(default)]
        from: Option<String>,
        to: String,
    },
    /// Merge multiple files
    Merge {
        inputs: Vec<String>,
        output: String,
        #[serde(default)]
        strategy: Option<String>,
    },
    /// Validate files
    Validate {
        input: String,
        #[serde(default)]
        schema: Option<String>,
    },
    /// Copy files
    Copy { input: String, output: String },
    /// Transform with query
    Transform {
        input: String,
        output: String,
        query: String,
    },
}

/// Batch execution result
#[derive(Debug)]
pub struct BatchResult {
    pub job_name: String,
    pub success: bool,
    pub message: String,
    pub duration_ms: u128,
}

/// Execute batch jobs from config
pub fn execute_batch(config: &BatchConfig, base_dir: &Path) -> Vec<BatchResult> {
    let mut results = Vec::new();

    for job in &config.jobs {
        // Check condition if present
        if let Some(ref condition) = job.condition {
            if !evaluate_condition(condition, &config.variables) {
                results.push(BatchResult {
                    job_name: job.name.clone(),
                    success: true,
                    message: "Skipped (condition not met)".to_string(),
                    duration_ms: 0,
                });
                continue;
            }
        }

        let start = std::time::Instant::now();
        let result = execute_job(job, base_dir, &config.variables);
        let duration = start.elapsed().as_millis();

        let batch_result = match result {
            Ok(msg) => BatchResult {
                job_name: job.name.clone(),
                success: true,
                message: msg,
                duration_ms: duration,
            },
            Err(e) => BatchResult {
                job_name: job.name.clone(),
                success: false,
                message: format!("Error: {}", e),
                duration_ms: duration,
            },
        };

        let should_stop = !batch_result.success && !config.continue_on_error;
        results.push(batch_result);

        if should_stop {
            break;
        }
    }

    results
}

fn execute_job(job: &BatchJob, base_dir: &Path, variables: &Option<JsonValue>) -> Result<String> {
    match &job.action {
        BatchAction::Convert {
            input,
            output,
            from,
            to,
        } => {
            let input_path = resolve_path(input, base_dir, variables);
            let output_path = resolve_path(output, base_dir, variables);

            let content = fs::read_to_string(&input_path)
                .with_context(|| format!("Failed to read: {}", input_path.display()))?;

            let from_format = if let Some(f) = from {
                parse_format(f)?
            } else {
                detect(Some(&input_path), &content)
                    .context("Could not detect source format")?
            };

            let to_format = parse_format(to)?;
            let converted = converter::convert(&content, from_format, to_format)?;

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&output_path, converted)?;

            Ok(format!(
                "Converted {} -> {}",
                input_path.display(),
                output_path.display()
            ))
        }

        BatchAction::Merge {
            inputs,
            output,
            strategy,
        } => {
            let mut values = Vec::new();

            for input in inputs {
                let input_path = resolve_path(input, base_dir, variables);
                let content = fs::read_to_string(&input_path)
                    .with_context(|| format!("Failed to read: {}", input_path.display()))?;

                let format = detect(Some(&input_path), &content)
                    .context("Could not detect format")?;

                let json_str = converter::convert(&content, format, Format::Json)?;
                let value: JsonValue = serde_json::from_str(&json_str)?;
                values.push(value);
            }

            let merge_strategy = match strategy.as_deref() {
                Some("shallow") => crate::core::merger::MergeStrategy::Shallow,
                Some("concat") => crate::core::merger::MergeStrategy::ConcatArrays,
                Some("union") => crate::core::merger::MergeStrategy::UnionArrays,
                _ => crate::core::merger::MergeStrategy::Deep,
            };

            let merged = crate::core::merger::merge_all(&values, merge_strategy)?;

            let output_path = resolve_path(output, base_dir, variables);
            let output_format = detect(Some(&output_path), "")
                .unwrap_or(Format::Json);

            let output_content = match output_format {
                Format::Yaml => serde_yaml::to_string(&merged)?,
                Format::Toml => toml::to_string_pretty(&merged)?,
                _ => serde_json::to_string_pretty(&merged)?,
            };

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&output_path, output_content)?;

            Ok(format!(
                "Merged {} files -> {}",
                inputs.len(),
                output_path.display()
            ))
        }

        BatchAction::Validate { input, schema } => {
            let input_path = resolve_path(input, base_dir, variables);
            let content = fs::read_to_string(&input_path)
                .with_context(|| format!("Failed to read: {}", input_path.display()))?;

            let format = detect(Some(&input_path), &content)
                .context("Could not detect format")?;

            if let Some(schema_path) = schema {
                let schema_path = resolve_path(schema_path, base_dir, variables);
                let schema_content = fs::read_to_string(&schema_path)?;
                let schema: JsonValue = serde_json::from_str(&schema_content)?;

                let json_str = converter::convert(&content, format, Format::Json)?;
                let data: JsonValue = serde_json::from_str(&json_str)?;

                let result = crate::core::validator::validate_json_schema(&data, &schema)?;
                if result.valid {
                    Ok(format!("Validated: {} (schema: {})", input_path.display(), schema_path.display()))
                } else {
                    anyhow::bail!("Validation failed: {} errors", result.errors.len())
                }
            } else {
                // Lint only
                let result = match format {
                    Format::Json => crate::core::validator::lint_json(&content)?,
                    Format::Yaml => crate::core::validator::lint_yaml(&content)?,
                    Format::Toml => crate::core::validator::lint_toml(&content)?,
                    Format::Csv => crate::core::validator::validate_csv(&content, true)?,
                    _ => {
                        let mut r = crate::core::validator::ValidationResult::new();
                        r.valid = true;
                        r
                    }
                };

                if result.valid {
                    Ok(format!("Validated: {}", input_path.display()))
                } else {
                    anyhow::bail!("Lint failed: {} errors", result.errors.len())
                }
            }
        }

        BatchAction::Copy { input, output } => {
            let input_path = resolve_path(input, base_dir, variables);
            let output_path = resolve_path(output, base_dir, variables);

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&input_path, &output_path)?;

            Ok(format!(
                "Copied {} -> {}",
                input_path.display(),
                output_path.display()
            ))
        }

        BatchAction::Transform {
            input,
            output,
            query,
        } => {
            let input_path = resolve_path(input, base_dir, variables);
            let content = fs::read_to_string(&input_path)?;

            let format = detect(Some(&input_path), &content)
                .context("Could not detect format")?;

            let json_str = converter::convert(&content, format, Format::Json)?;
            let value: JsonValue = serde_json::from_str(&json_str)?;

            let result = crate::core::query::jsonpath_query(&value, query)?;

            let output_path = resolve_path(output, base_dir, variables);
            let output_format = detect(Some(&output_path), "")
                .unwrap_or(Format::Json);

            let output_content = match output_format {
                Format::Yaml => serde_yaml::to_string(&result)?,
                _ => serde_json::to_string_pretty(&result)?,
            };

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&output_path, output_content)?;

            Ok(format!(
                "Transformed {} -> {}",
                input_path.display(),
                output_path.display()
            ))
        }
    }
}

fn resolve_path(path: &str, base_dir: &Path, variables: &Option<JsonValue>) -> PathBuf {
    let resolved = if let Some(vars) = variables {
        let options = crate::core::template::TemplateOptions::default();
        crate::core::template::render_string(path, vars, &options)
            .unwrap_or_else(|_| path.to_string())
    } else {
        path.to_string()
    };

    let path = PathBuf::from(&resolved);
    if path.is_absolute() {
        path
    } else {
        base_dir.join(path)
    }
}

fn parse_format(s: &str) -> Result<Format> {
    match s.to_lowercase().as_str() {
        "json" => Ok(Format::Json),
        "yaml" | "yml" => Ok(Format::Yaml),
        "toml" => Ok(Format::Toml),
        "csv" => Ok(Format::Csv),
        "xml" => Ok(Format::Xml),
        _ => anyhow::bail!("Unknown format: {}", s),
    }
}

fn evaluate_condition(condition: &str, variables: &Option<JsonValue>) -> bool {
    // Simple condition evaluation: check if variable exists and is truthy
    if let Some(vars) = variables {
        if let Some(value) = vars.get(condition) {
            return match value {
                JsonValue::Bool(b) => *b,
                JsonValue::Null => false,
                JsonValue::String(s) => !s.is_empty(),
                JsonValue::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
                _ => true,
            };
        }
    }
    false
}

/// Format batch results for display
pub fn format_results(results: &[BatchResult]) -> String {
    let mut output = String::new();

    let total = results.len();
    let passed = results.iter().filter(|r| r.success).count();
    let failed = total - passed;

    output.push_str(&format!(
        "\n{}\n",
        "=== Batch Execution Results ===".bold()
    ));

    for result in results {
        let status = if result.success {
            "PASS".green()
        } else {
            "FAIL".red()
        };

        output.push_str(&format!(
            "[{}] {} ({:.2}ms): {}\n",
            status,
            result.job_name.cyan(),
            result.duration_ms,
            result.message
        ));
    }

    output.push_str(&format!(
        "\n{}: {} total, {} passed, {} failed\n",
        "Summary".bold(),
        total,
        passed.to_string().green(),
        if failed > 0 {
            failed.to_string().red().to_string()
        } else {
            "0".to_string()
        }
    ));

    output
}

/// Parse batch config from file
pub fn parse_config(content: &str, format: Format) -> Result<BatchConfig> {
    match format {
        Format::Yaml => serde_yaml::from_str(content).context("Failed to parse batch config as YAML"),
        Format::Json => serde_json::from_str(content).context("Failed to parse batch config as JSON"),
        Format::Toml => toml::from_str(content).context("Failed to parse batch config as TOML"),
        _ => anyhow::bail!("Batch config must be YAML, JSON, or TOML"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let yaml = r#"
jobs:
  - name: "Convert JSON to YAML"
    action: convert
    input: "input.json"
    output: "output.yaml"
    to: "yaml"
continue_on_error: true
"#;

        let config = parse_config(yaml, Format::Yaml).unwrap();
        assert_eq!(config.jobs.len(), 1);
        assert!(config.continue_on_error);
    }
}

