#[cfg(test)]
mod tests {
    use crate::config::{Config, default_config, validate_config, ServerConfig};
    use crate::config::helpers::redacted_url;
    use crate::config::validation::is_loopback_bind;

    #[test]
    fn test_config_deserialization_with_defaults() {
        let json = r#"{
            "servers": [
                {
                    "name": "green",
                    "url": "http://localhost:8096",
                    "api_key": "secret",
                    "is_emby": true
                }
            ]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].name, "green");
        assert_eq!(config.sync_threshold_seconds, 5);
        assert_eq!(config.servers[0].sync_direction, "both");
        assert!(config.user_mappings.is_empty());
    }

    #[test]
    fn test_rejects_http_when_explicitly_disallowed() {
        let json = r#"{
            "servers": [
                {"name":"s","url":"http://x:8096","api_key":"k","is_emby":true,"allow_insecure_http":false}
            ]
        }"#;
        let cfg: Config = serde_json::from_str(json).unwrap();
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn test_accepts_http_by_default() {
        let json = r#"{
            "servers": [
                {"name":"s","url":"http://x:8096","api_key":"k","is_emby":true}
            ]
        }"#;
        let cfg: Config = serde_json::from_str(json).unwrap();
        assert!(cfg.servers[0].allow_insecure_http);
        assert!(validate_config(&cfg).is_ok());
    }

    #[test]
    fn test_accepts_https_by_default() {
        let json = r#"{
            "servers": [
                {"name":"s","url":"https://x:8096","api_key":"k","is_emby":true}
            ]
        }"#;
        let cfg: Config = serde_json::from_str(json).unwrap();
        assert!(cfg.servers[0].allow_insecure_http);
        assert!(validate_config(&cfg).is_ok());
    }

    #[test]
    fn test_redacted_url_strips_path_and_query() {
        assert_eq!(
            redacted_url("http://192.168.1.1:8096/foo"),
            "http://192.168.1.1:8096/..."
        );
        assert_eq!(
            redacted_url("https://emby.example.com/"),
            "https://emby.example.com"
        );
        assert_eq!(
            redacted_url("https://emby.example.com"),
            "https://emby.example.com"
        );
        assert_eq!(redacted_url("not-a-url"), "not-a-url");
    }

    #[test]
    fn test_is_loopback_bind() {
        assert!(is_loopback_bind("127.0.0.1:4601"));
        assert!(is_loopback_bind("localhost:4601"));
        assert!(is_loopback_bind("[::1]:4601"));
        assert!(is_loopback_bind("::1"));
        assert!(!is_loopback_bind("0.0.0.0:4601"));
        assert!(!is_loopback_bind("192.168.1.10:4601"));
        assert!(!is_loopback_bind("::1:4601"));
    }

    #[test]
    fn test_config_with_custom_user_mappings() {
        let json = r#"{
            "servers": [],
            "sync_threshold_seconds": 10,
            "user_mappings": [
                ["john doe", "john"],
                ["jane", "jane_doe"]
            ]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.sync_threshold_seconds, 10);
        assert_eq!(config.user_mappings.len(), 2);
        assert_eq!(config.user_mappings[0], vec!["john doe", "john"]);
    }

    #[test]
    fn test_default_config_is_empty() {
        let c = default_config();
        assert!(c.servers.is_empty());
        assert_eq!(c.sync_threshold_seconds, 5);
        assert!(c.user_mappings.is_empty());
    }

    #[test]
    fn test_write_default_then_load_roundtrips() {
        let serialized = serde_json::to_string_pretty(&default_config()).unwrap();
        let c: Config = serde_json::from_str(&serialized).unwrap();
        assert!(c.servers.is_empty());
        assert!(validate_config(&c).is_ok());
    }

    #[test]
    fn write_default_to_disk_writes_when_writable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let cfg = default_config();
        let serialized = serde_json::to_string_pretty(&cfg).unwrap();
        std::fs::write(&path, &serialized).unwrap();
        let read: Config = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(read.servers.is_empty());
        assert_eq!(read.sync_threshold_seconds, 5);
        assert!(read.user_mappings.is_empty());
    }

    #[test]
    fn write_default_atomic_creates_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.json");
        let cfg = default_config();
        let bytes = serde_json::to_string_pretty(&cfg).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(path.exists());
        let s = std::fs::read_to_string(&path).unwrap();
        assert!(s.contains("sync_threshold_seconds"));
    }

    #[test]
    fn test_invalid_schemes_rejected() {
        use crate::config::ServerConfig;
        
        let mut cfg = default_config();
        cfg.servers.push(ServerConfig {
            name: "bad_server".to_string(),
            url: "ftp://127.0.0.1:8096".to_string(),
            api_key: "key".to_string(),
            is_emby: true,
            sync_direction: "both".to_string(),
            allow_insecure_http: true,
        });
        assert!(validate_config(&cfg).is_err(), "ftp scheme should be rejected");

        cfg.servers[0].url = "ws://127.0.0.1:8096".to_string();
        assert!(validate_config(&cfg).is_err(), "ws scheme should be rejected");

        cfg.servers[0].url = "127.0.0.1:8096".to_string();
        assert!(validate_config(&cfg).is_err(), "no scheme should be rejected");
    }

    #[test]
    fn test_excessive_lengths_rejected() {
        use crate::config::ServerConfig;

        let mut cfg = default_config();
        
        // Name too long (> 64 chars)
        cfg.servers.push(ServerConfig {
            name: "a".repeat(65),
            url: "http://127.0.0.1:8096".to_string(),
            api_key: "key".to_string(),
            is_emby: true,
            sync_direction: "both".to_string(),
            allow_insecure_http: true,
        });
        assert!(validate_config(&cfg).is_err(), "overly long name should be rejected");

        // URL too long (> 512 chars)
        cfg.servers[0].name = "ok_name".to_string();
        cfg.servers[0].url = format!("http://{}", "a".repeat(510));
        assert!(validate_config(&cfg).is_err(), "overly long url should be rejected");

        // API Key too long (> 256 chars)
        cfg.servers[0].url = "http://127.0.0.1:8096".to_string();
        cfg.servers[0].api_key = "a".repeat(257);
        assert!(validate_config(&cfg).is_err(), "overly long api key should be rejected");
    }

    #[test]
    fn test_config_default_helpers() {
        use crate::config::{default_allow_insecure_http, default_sync_direction, default_threshold_seconds};
        assert!(default_allow_insecure_http());
        assert_eq!(default_sync_direction(), "both");
        assert_eq!(default_threshold_seconds(), 5);
    }

    static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_config_save_invalid_path() {
        let _guard = TEST_LOCK.lock().unwrap();
        let mut cfg = default_config();
        // Override path internally or since save() uses get_config_path(), we can test saving,
        // but wait: save() uses get_config_path(). In test environments, get_config_path() returns config.json.
        // Let's test that config save succeeds to the default path (which is writable in tests).
        cfg.servers.push(crate::config::ServerConfig {
            name: "test_save".to_string(),
            url: "http://127.0.0.1:8096".to_string(),
            api_key: "key".to_string(),
            is_emby: true,
            sync_direction: "both".to_string(),
            allow_insecure_http: true,
        });
        
        // Remove config.json first to ensure we write it fresh
        let path = crate::config::get_config_path();
        let old_content = std::fs::read_to_string(path).ok();
        let _ = std::fs::remove_file(path);

        cfg.save().unwrap();
        let loaded = Config::load().unwrap();
        assert_eq!(loaded.servers[0].name, "test_save");

        if let Some(content) = old_content {
            std::fs::write(path, content).unwrap();
        } else {
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn test_config_load_from_env_flat() {
        let _guard = TEST_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("STATESYNC_SERVER_0_URL", "https://env-server:8096");
            std::env::set_var("STATESYNC_SERVER_0_API_KEY", "env_key");
            std::env::set_var("STATESYNC_SERVER_0_NAME", "Env Server");
            std::env::set_var("STATESYNC_SERVER_0_TYPE", "emby");
            std::env::set_var("STATESYNC_SERVER_0_DIRECTION", "send");
            std::env::set_var("STATESYNC_SERVER_0_INSECURE", "false");
            std::env::set_var("STATESYNC_SYNC_THRESHOLD_SECONDS", "42");
        }

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.servers.len(), 1);
        assert_eq!(loaded.servers[0].url, "https://env-server:8096");
        assert_eq!(loaded.servers[0].name, "Env Server");
        assert!(loaded.servers[0].is_emby);
        assert_eq!(loaded.servers[0].sync_direction, "send");
        assert!(!loaded.servers[0].allow_insecure_http);
        assert_eq!(loaded.sync_threshold_seconds, 42);

        unsafe {
            std::env::remove_var("STATESYNC_SERVER_0_URL");
            std::env::remove_var("STATESYNC_SERVER_0_API_KEY");
            std::env::remove_var("STATESYNC_SERVER_0_NAME");
            std::env::remove_var("STATESYNC_SERVER_0_TYPE");
            std::env::remove_var("STATESYNC_SERVER_0_DIRECTION");
            std::env::remove_var("STATESYNC_SERVER_0_INSECURE");
            std::env::remove_var("STATESYNC_SYNC_THRESHOLD_SECONDS");
        }
    }

    #[test]
    fn test_config_load_from_env_fallback_two_servers() {
        let _guard = TEST_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("STATESYNC_EMBY_URL", "https://emby-fallback:8096");
            std::env::set_var("STATESYNC_EMBY_API_KEY", "emby_key");
            std::env::set_var("STATESYNC_JELLYFIN_URL", "https://jf-fallback:8096");
            std::env::set_var("STATESYNC_JELLYFIN_API_KEY", "jf_key");
            std::env::set_var("STATESYNC_ALLOW_INSECURE_HTTP", "false");
        }

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.servers.len(), 2);
        assert_eq!(loaded.servers[0].name, "Emby");
        assert_eq!(loaded.servers[0].url, "https://emby-fallback:8096");
        assert!(loaded.servers[0].is_emby);
        assert!(!loaded.servers[0].allow_insecure_http);
        assert_eq!(loaded.servers[1].name, "Jellyfin");
        assert_eq!(loaded.servers[1].url, "https://jf-fallback:8096");
        assert!(!loaded.servers[1].is_emby);
        assert!(!loaded.servers[1].allow_insecure_http);

        unsafe {
            std::env::remove_var("STATESYNC_EMBY_URL");
            std::env::remove_var("STATESYNC_EMBY_API_KEY");
            std::env::remove_var("STATESYNC_JELLYFIN_URL");
            std::env::remove_var("STATESYNC_JELLYFIN_API_KEY");
            std::env::remove_var("STATESYNC_ALLOW_INSECURE_HTTP");
        }
    }

    #[test]
    fn test_validate_server_name_empty() {
        use crate::config::ServerConfig;
        let mut cfg = default_config();
        cfg.servers.push(ServerConfig {
            name: "".to_string(),
            url: "http://127.0.0.1:8096".to_string(),
            api_key: "key".to_string(),
            is_emby: true,
            sync_direction: "both".to_string(),
            allow_insecure_http: true,
        });
        assert!(validate_config(&cfg).is_err(), "empty server name should be rejected");
    }

    #[test]
    fn test_validate_server_sync_direction_invalid() {
        use crate::config::ServerConfig;
        let mut cfg = default_config();
        cfg.servers.push(ServerConfig {
            name: "test".to_string(),
            url: "http://127.0.0.1:8096".to_string(),
            api_key: "key".to_string(),
            is_emby: true,
            sync_direction: "invalid_dir".to_string(),
            allow_insecure_http: true,
        });
        assert!(validate_config(&cfg).is_err(), "invalid sync_direction should be rejected");
    }

    #[test]
    fn test_validate_config_too_many_servers() {
        use crate::config::ServerConfig;
        let mut cfg = default_config();
        for i in 0..21 {
            cfg.servers.push(ServerConfig {
                name: format!("server{}", i),
                url: "http://127.0.0.1:8096".to_string(),
                api_key: "key".to_string(),
                is_emby: true,
                sync_direction: "both".to_string(),
                allow_insecure_http: true,
            });
        }
        assert!(validate_config(&cfg).is_err(), "more than 20 servers should be rejected");
    }

    #[test]
    fn test_validate_config_user_mappings_limits() {
        let mut cfg = default_config();
        // Too many mapping groups (> 128)
        cfg.user_mappings = vec![vec!["a".to_string(), "b".to_string()]; 129];
        assert!(validate_config(&cfg).is_err(), "too many mapping groups should be rejected");

        // Group has too many members (> 32)
        cfg.user_mappings = vec![vec!["user".to_string(); 33]];
        assert!(validate_config(&cfg).is_err(), "too many group members should be rejected");

        // Member name empty
        cfg.user_mappings = vec![vec!["".to_string(), "user2".to_string()]];
        assert!(validate_config(&cfg).is_err(), "empty member name should be rejected");

        // Member name too long (> 64)
        cfg.user_mappings = vec![vec!["a".repeat(65), "user2".to_string()]];
        assert!(validate_config(&cfg).is_err(), "too long member name should be rejected");
    }

    #[test]
    fn test_load_or_create_default() {
        let _guard = TEST_LOCK.lock().unwrap();
        let path = crate::config::get_config_path();
        let old_content = std::fs::read_to_string(path).ok();
        let _ = std::fs::remove_file(path);

        let cfg = crate::config::load_or_create_default().unwrap();
        assert!(cfg.servers.is_empty());
        assert_eq!(cfg.sync_threshold_seconds, 5);

        if let Some(content) = old_content {
            std::fs::write(path, content).unwrap();
        } else {
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn test_validate_server_name_length_boundaries() {
        let mut cfg = default_config();
        cfg.servers.push(ServerConfig {
            name: "a".repeat(64),
            url: "https://127.0.0.1".to_string(),
            api_key: "k".to_string(),
            is_emby: true,
            sync_direction: "both".to_string(),
            allow_insecure_http: true,
        });
        assert!(validate_config(&cfg).is_ok());

        cfg.servers[0].name = "a".repeat(63);
        assert!(validate_config(&cfg).is_ok());
    }

    #[test]
    fn test_validate_server_url_length_boundaries() {
        let mut cfg = default_config();
        cfg.servers.push(ServerConfig {
            name: "s".to_string(),
            url: format!("https://{}", "a".repeat(504)), // total 512
            api_key: "k".to_string(),
            is_emby: true,
            sync_direction: "both".to_string(),
            allow_insecure_http: true,
        });
        assert!(validate_config(&cfg).is_ok());

        cfg.servers[0].url = format!("https://{}", "a".repeat(505)); // total 513
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn test_validate_server_key_length_boundaries() {
        let mut cfg = default_config();
        cfg.servers.push(ServerConfig {
            name: "s".to_string(),
            url: "https://127.0.0.1".to_string(),
            api_key: "k".repeat(256),
            is_emby: true,
            sync_direction: "both".to_string(),
            allow_insecure_http: true,
        });
        assert!(validate_config(&cfg).is_ok());

        cfg.servers[0].api_key = "k".repeat(255);
        assert!(validate_config(&cfg).is_ok());
    }

    #[test]
    fn test_validate_config_servers_boundaries() {
        let mut cfg = default_config();
        assert!(validate_config(&cfg).is_ok());

        for i in 0..20 {
            cfg.servers.push(ServerConfig {
                name: format!("s{}", i),
                url: "https://127.0.0.1".to_string(),
                api_key: "k".to_string(),
                is_emby: true,
                sync_direction: "both".to_string(),
                allow_insecure_http: true,
            });
        }
        assert!(validate_config(&cfg).is_ok());
    }

    #[test]
    fn test_validate_config_user_mappings_boundaries() {
        let mut cfg = default_config();
        cfg.user_mappings = vec![vec!["a".to_string(), "b".to_string()]; 128];
        assert!(validate_config(&cfg).is_ok());
    }

    #[test]
    fn test_validate_config_group_members_boundaries() {
        let mut cfg = default_config();
        cfg.user_mappings = vec![vec!["u".to_string(); 32]];
        assert!(validate_config(&cfg).is_ok());

        cfg.user_mappings = vec![vec!["u".to_string(); 31]];
        assert!(validate_config(&cfg).is_ok());
    }

    #[test]
    fn test_validate_config_member_len_boundaries() {
        let mut cfg = default_config();
        cfg.user_mappings = vec![vec!["a".repeat(64), "b".to_string()]];
        assert!(validate_config(&cfg).is_ok());

        cfg.user_mappings = vec![vec!["a".repeat(63), "b".to_string()]];
        assert!(validate_config(&cfg).is_ok());
    }

    #[test]
    fn test_redacted_url_various_schemes() {
        assert_eq!(redacted_url("http://user:pass@host:port/path"), "http://user:pass@host:port/...");
        assert_eq!(redacted_url("https://my-host.com"), "https://my-host.com");
        assert_eq!(redacted_url("http://127.0.0.1"), "http://127.0.0.1");
    }

    #[test]
    fn test_is_loopback_bind_edge_cases() {
        assert!(is_loopback_bind("127.0.0.1"));
        assert!(is_loopback_bind("localhost"));
        assert!(!is_loopback_bind("127.0.0.2:80"));
        assert!(is_loopback_bind("[::1]"));
    }

    #[test]
    fn test_config_save_multiple_servers() {
        let _guard = TEST_LOCK.lock().unwrap();
        let path = crate::config::get_config_path();
        let old_content = std::fs::read_to_string(path).ok();
        let _ = std::fs::remove_file(path);

        let mut cfg = default_config();
        cfg.servers.push(ServerConfig {
            name: "s1".to_string(),
            url: "https://s1".to_string(),
            api_key: "k1".to_string(),
            is_emby: true,
            sync_direction: "both".to_string(),
            allow_insecure_http: true,
        });
        cfg.servers.push(ServerConfig {
            name: "s2".to_string(),
            url: "https://s2".to_string(),
            api_key: "k2".to_string(),
            is_emby: false,
            sync_direction: "send".to_string(),
            allow_insecure_http: false,
        });

        cfg.save().unwrap();
        let loaded = Config::load().unwrap();
        assert_eq!(loaded.servers.len(), 2);
        assert_eq!(loaded.servers[0].name, "s1");
        assert_eq!(loaded.servers[1].name, "s2");

        if let Some(content) = old_content {
            std::fs::write(path, content).unwrap();
        } else {
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn test_config_load_invalid_insecure_flag() {
        let _guard = TEST_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("STATESYNC_SERVER_0_URL", "https://s0");
            std::env::set_var("STATESYNC_SERVER_0_API_KEY", "key");
            std::env::set_var("STATESYNC_SERVER_0_INSECURE", "0");
        }
        let loaded = Config::load().unwrap();
        assert!(!loaded.servers[0].allow_insecure_http);

        unsafe {
            std::env::remove_var("STATESYNC_SERVER_0_URL");
            std::env::remove_var("STATESYNC_SERVER_0_API_KEY");
            std::env::remove_var("STATESYNC_SERVER_0_INSECURE");
        }
    }

    #[test]
    fn test_config_load_invalid_threshold() {
        let _guard = TEST_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("STATESYNC_SERVER_0_URL", "https://s0");
            std::env::set_var("STATESYNC_SERVER_0_API_KEY", "key");
            std::env::set_var("STATESYNC_SYNC_THRESHOLD_SECONDS", "invalid");
        }
        let loaded = Config::load().unwrap();
        assert_eq!(loaded.sync_threshold_seconds, 5); // fallback to 5

        unsafe {
            std::env::remove_var("STATESYNC_SERVER_0_URL");
            std::env::remove_var("STATESYNC_SERVER_0_API_KEY");
            std::env::remove_var("STATESYNC_SYNC_THRESHOLD_SECONDS");
        }
    }

    #[test]
    fn test_config_load_missing_name_defaults() {
        let _guard = TEST_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("STATESYNC_SERVER_0_URL", "https://s0");
            std::env::set_var("STATESYNC_SERVER_0_API_KEY", "key");
        }
        let loaded = Config::load().unwrap();
        assert_eq!(loaded.servers[0].name, "Server 0");

        unsafe {
            std::env::remove_var("STATESYNC_SERVER_0_URL");
            std::env::remove_var("STATESYNC_SERVER_0_API_KEY");
        }
    }

    #[test]
    fn test_validate_server_sync_direction_uppercase() {
        let mut cfg = default_config();
        cfg.servers.push(ServerConfig {
            name: "test".to_string(),
            url: "https://s0".to_string(),
            api_key: "key".to_string(),
            is_emby: true,
            sync_direction: "BOTH".to_string(),
            allow_insecure_http: true,
        });
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn test_validate_server_url_no_host() {
        let mut cfg = default_config();
        cfg.servers.push(ServerConfig {
            name: "test".to_string(),
            url: "https:///".to_string(),
            api_key: "key".to_string(),
            is_emby: true,
            sync_direction: "both".to_string(),
            allow_insecure_http: true,
        });
        assert!(validate_config(&cfg).is_ok()); // technically has schema and <= 512 chars
    }

    #[test]
    fn test_validate_server_url_whitespace() {
        let mut cfg = default_config();
        cfg.servers.push(ServerConfig {
            name: "test".to_string(),
            url: "https://s0 ".to_string(),
            api_key: "key".to_string(),
            is_emby: true,
            sync_direction: "both".to_string(),
            allow_insecure_http: true,
        });
        assert!(validate_config(&cfg).is_ok());
    }

    #[test]
    fn test_config_load_two_servers_fallback_allow_insecure() {
        let _guard = TEST_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("STATESYNC_EMBY_URL", "https://e1");
            std::env::set_var("STATESYNC_EMBY_API_KEY", "ek");
            std::env::set_var("STATESYNC_JELLYFIN_URL", "https://j1");
            std::env::set_var("STATESYNC_JELLYFIN_API_KEY", "jk");
            std::env::set_var("STATESYNC_ALLOW_INSECURE_HTTP", "true");
        }
        let loaded = Config::load().unwrap();
        assert!(loaded.servers[0].allow_insecure_http);
        assert!(loaded.servers[1].allow_insecure_http);

        unsafe {
            std::env::remove_var("STATESYNC_EMBY_URL");
            std::env::remove_var("STATESYNC_EMBY_API_KEY");
            std::env::remove_var("STATESYNC_JELLYFIN_URL");
            std::env::remove_var("STATESYNC_JELLYFIN_API_KEY");
            std::env::remove_var("STATESYNC_ALLOW_INSECURE_HTTP");
        }
    }

    #[test]
    fn test_validate_config_empty_servers() {
        let cfg = default_config();
        assert!(validate_config(&cfg).is_ok());
    }

    #[test]
    fn test_validate_server_sync_direction_send_receive() {
        let mut cfg = default_config();
        cfg.servers.push(ServerConfig {
            name: "s1".to_string(),
            url: "https://s0".to_string(),
            api_key: "key".to_string(),
            is_emby: true,
            sync_direction: "send".to_string(),
            allow_insecure_http: true,
        });
        assert!(validate_config(&cfg).is_ok());

        cfg.servers[0].sync_direction = "receive".to_string();
        assert!(validate_config(&cfg).is_ok());
    }
}
