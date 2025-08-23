//! 配置模块
use crate::error::{NanoError, Result};
use dotenv::dotenv;
use std::env;
use std::time::Duration;
use fastrand;

// ===============================================================================================
// 配置模块
// ===============================================================================================

/// LLM 客户端配置
///
/// 包含所有必要的配置参数，支持 Builder 模式和环境变量配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 模型名称
    pub(crate) model: String,
    /// 系统消息
    pub(crate) system_message: String,
    /// 温度参数 (0.0-2.0)
    pub(crate) temperature: f32,
    /// Top-p 参数 (0.0-1.0)
    pub(crate) top_p: f32,
    /// 最大生成 token 数
    pub(crate) max_tokens: u32,
    /// 请求超时时间
    pub(crate) timeout: Duration,
    /// API 基础 URL
    pub(crate) api_base: String,
    /// API 密钥
    pub(crate) api_key: String,
    /// 随机种子
    pub(crate) random_seed: Option<u64>,
    /// 最大并发请求数
    pub(crate) max_concurrent_requests: Option<usize>,
    /// 连接池空闲超时时间
    pub(crate) pool_idle_timeout: Duration,
    /// 每个主机的最大空闲连接数
    pub(crate) pool_max_idle_per_host: usize,
    /// TCP Keepalive
    pub(crate) tcp_keepalive: Duration,
    /// TCP Nodelay
    pub(crate) tcp_nodelay: bool,
}

impl Default for Config {
    /// 创建默认配置
    ///
    /// 使用 DeepSeek 免费模型作为默认选择
    fn default() -> Self {
        Self {
            model: "deepseek-chat".into(),
            system_message: "You are a helpful AI assistant.".into(),
            temperature: 0.7,
            top_p: 1.0,
            max_tokens: 4096,
            timeout: Duration::from_secs(60),
            api_base: "https://openrouter.ai/api/v1".into(),
            api_key: String::new(),
            random_seed: None,
            max_concurrent_requests: Some(64),
            pool_idle_timeout: Duration::from_secs(90),
            pool_max_idle_per_host: 16,
            tcp_keepalive: Duration::from_secs(60),
            tcp_nodelay: true,
        }
    }
}

/// 生成 Config Builder 方法的宏
///
/// 自动生成 `with_field_name` 形式的 builder 方法
macro_rules! config_builder {
    ($field:ident, $type:ty) => {
        paste::paste! {
            #[doc = "设置 `"]
            #[doc = stringify!($field)]
            #[doc = "`"]
            pub fn [<with_ $field>](mut self, $field: $type) -> Self {
                self.$field = $field;
                self
            }
        }
    };
    ($field:ident, $type:ty, option) => {
        paste::paste! {
            #[doc = "设置 `"]
            #[doc = stringify!($field)]
            #[doc = "`"]
            pub fn [<with_ $field>](mut self, $field: $type) -> Self {
                self.$field = Some($field);
                self
            }
        }
    };
}

impl Config {
    pub fn model(&self) -> &str { &self.model }
    pub fn temperature(&self) -> f32 { self.temperature }
    pub fn top_p(&self) -> f32 { self.top_p }
    pub fn max_tokens(&self) -> u32 { self.max_tokens }
    pub fn timeout(&self) -> Duration { self.timeout }
    pub fn api_base(&self) -> &str { &self.api_base }
    pub fn api_key(&self) -> &str { &self.api_key }

    /// 从环境变量和 `.env` 文件加载配置
    ///
    /// 环境变量会覆盖 `.env` 文件中的设置
    pub fn from_env() -> Result<Self> {
        dotenv().ok();
        let api_key = env::var("OPENROUTER_API_KEY")
            .map_err(|_| NanoError::Config("YOPENROUTER_API_KEY not found".into()))?;

        let model = env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string());
        let api_base = env::var("API_BASE").unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

        let config = Config {
            api_key,
            model,
            api_base,
            ..Default::default()
        };

        Ok(config)
    }

    // 使用宏生成 builder 方法
    config_builder!(api_base, String);
    config_builder!(model, String);
    config_builder!(api_key, String);
    config_builder!(temperature, f32);
    config_builder!(top_p, f32);
    config_builder!(max_tokens, u32);
    config_builder!(timeout, Duration);
    config_builder!(random_seed, u64, option);
    config_builder!(max_concurrent_requests, usize, option);
    config_builder!(pool_idle_timeout, Duration);
    config_builder!(pool_max_idle_per_host, usize);
    config_builder!(tcp_keepalive, Duration);
    config_builder!(tcp_nodelay, bool);

    /// 自动生成随机种子
    ///
    /// 使用高性能的 WyRand 算法生成随机种子
    pub fn with_random_seed_auto(mut self) -> Self {
        self.random_seed = Some(fastrand::u64(..));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::tempdir;

    // Mutex to protect environment variable tests from running in parallel
    lazy_static::lazy_static! {
        static ref ENV_LOCK: Mutex<()> = Mutex::new(());
    }

    /// Tests that the default configuration is created correctly.
    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.model, "deepseek-chat");
        assert_eq!(config.system_message, "You are a helpful AI assistant.");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.api_key, "");
        assert!(config.random_seed.is_none());
    }

    /// Tests the builder methods for setting configuration fields.
    #[test]
    fn test_config_builder_methods() {
        let config = Config::default()
            .with_model("test_model".to_string())
            .with_api_key("test_key".to_string())
            .with_temperature(0.9)
            .with_random_seed(12345);

        assert_eq!(config.model, "test_model");
        assert_eq!(config.api_key, "test_key");
        assert_eq!(config.temperature, 0.9);
        assert_eq!(config.random_seed, Some(12345));
    }

    /// Tests that `with_random_seed_auto` sets a random seed.
    #[test]
    fn test_with_random_seed_auto() {
        let config = Config::default().with_random_seed_auto();
        assert!(config.random_seed.is_some());
    }

    /// Tests loading configuration from a .env file.
    #[test]
    fn test_from_env_with_dotenv_file() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(".env");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "DEEPSEEK_API_KEY=dotenv_key").unwrap();
        writeln!(file, "MODEL=dotenv_model").unwrap();

        // Temporarily change the current directory to the temp dir
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        let config = Config::from_env().unwrap();
        assert_eq!(config.api_key, "dotenv_key");
        assert_eq!(config.model, "dotenv_model");

        // Restore the original directory
        env::set_current_dir(original_dir).unwrap();
    }

    /// Tests loading configuration from environment variables.
    #[test]
    fn test_from_env_with_env_vars() {
        let _lock = ENV_LOCK.lock().unwrap();

        // Run in a directory without a .env file to isolate the test
        let dir = tempdir().unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        env::set_var("DEEPSEEK_API_KEY", "env_var_key");
        env::set_var("MODEL", "env_var_model");

        let config = Config::from_env().unwrap();
        assert_eq!(config.api_key, "env_var_key");
        assert_eq!(config.model, "env_var_model");

        // Cleanup
        env::remove_var("DEEPSEEK_API_KEY");
        env::remove_var("MODEL");
        env::set_current_dir(original_dir).unwrap();
    }

    /// Tests that environment variables have priority over the .env file.
    #[test]
    fn test_from_env_priority() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(".env");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "DEEPSEEK_API_KEY=dotenv_key").unwrap();

        env::set_var("DEEPSEEK_API_KEY", "env_var_key");

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        let config = Config::from_env().unwrap();
        assert_eq!(config.api_key, "env_var_key");

        env::set_current_dir(original_dir).unwrap();
        env::remove_var("DEEPSEEK_API_KEY");
    }

    /// Tests that an error is returned if the API key is not found.
    #[test]
    fn test_from_env_missing_api_key() {
        let _lock = ENV_LOCK.lock().unwrap();
        // Ensure no relevant env vars are set
        env::remove_var("DEEPSEEK_API_KEY");

        // Run in a directory without a .env file
        let dir = tempdir().unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        let result = Config::from_env();
        assert!(matches!(result, Err(NanoError::Config(_))));

        env::set_current_dir(original_dir).unwrap();
    }

    /// Tests that default values are used when no env or .env values are set.
    #[test]
    fn test_from_env_uses_defaults_for_model() {
        let _lock = ENV_LOCK.lock().unwrap();
        env::set_var("DEEPSEEK_API_KEY", "some_key");
        // Ensure no model is set in env or .env
        env::remove_var("MODEL");

        let dir = tempdir().unwrap();
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        let config = Config::from_env().unwrap();
        assert_eq!(config.model, Config::default().model);

        env::set_current_dir(original_dir).unwrap();
        env::remove_var("DEEPSEEK_API_KEY");
    }
}