use bend_pvm::runtime::proxy::{Proxy, ProxyError, ProxyState, VersionInfo};

fn create_test_proxy() -> ProxyState {
    ProxyState::new("impl_v1".to_string())
}

mod proxy_tests {
    use super::*;

    #[test]
    fn test_proxy_state_creation() {
        let proxy = create_test_proxy();
        assert_eq!(proxy.implementation(), "impl_v1");
        assert_eq!(proxy.version(), 1);
        assert!(!proxy.is_paused());
    }

    #[test]
    fn test_proxy_initialization() {
        let mut proxy = ProxyState::new("initial_impl".to_string());
        assert_eq!(proxy.implementation(), "initial_impl");
        assert!(proxy.admin().is_none());

        proxy.set_admin("admin_address".to_string()).unwrap();
        assert_eq!(proxy.admin(), Some(&"admin_address".to_string()));
    }

    #[test]
    fn test_proxy_version_tracking() {
        let proxy = create_test_proxy();
        assert_eq!(proxy.version(), 1);

        let proxy_v2 = proxy.clone_with_version(2);
        assert_eq!(proxy_v2.version(), 2);
        assert_eq!(proxy_v2.implementation(), "impl_v1");
    }

    #[test]
    fn test_proxy_upgrade_requires_admin() {
        let mut proxy = create_test_proxy();

        let result = proxy.upgrade("impl_v2".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProxyError::Unauthorized));
    }

    #[test]
    fn test_proxy_upgrade_with_admin() {
        let mut proxy = create_test_proxy();
        proxy.set_admin("admin1".to_string()).unwrap();

        proxy.upgrade("impl_v2".to_string()).unwrap();
        assert_eq!(proxy.implementation(), "impl_v2");
        assert_eq!(proxy.version(), 2);
    }

    #[test]
    fn test_proxy_pause_requires_admin() {
        let mut proxy = create_test_proxy();

        let result = proxy.pause();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProxyError::Unauthorized));
    }

    #[test]
    fn test_proxy_pause_with_admin() {
        let mut proxy = create_test_proxy();
        proxy.set_admin("admin1".to_string()).unwrap();

        proxy.pause().unwrap();
        assert!(proxy.is_paused());
    }

    #[test]
    fn test_proxy_unpause() {
        let mut proxy = create_test_proxy();
        proxy.set_admin("admin1".to_string()).unwrap();

        proxy.pause().unwrap();
        assert!(proxy.is_paused());

        proxy.unpause().unwrap();
        assert!(!proxy.is_paused());
    }

    #[test]
    fn test_proxy_call_when_paused() {
        let mut proxy = create_test_proxy();
        proxy.set_admin("admin1".to_string()).unwrap();
        proxy.pause().unwrap();

        let result = proxy.forward_call("any_func", vec![]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProxyError::ContractPaused));
    }

    #[test]
    fn test_proxy_forward_call() {
        let proxy = create_test_proxy();
        let result = proxy.forward_call("get_value", vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_proxy_forward_call_with_params() {
        let proxy = create_test_proxy();
        let result = proxy.forward_call("set_value", vec!["value".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_proxy_change_admin() {
        let mut proxy = create_test_proxy();
        proxy.set_admin("admin1".to_string()).unwrap();

        proxy.change_admin("admin2".to_string()).unwrap();
        assert_eq!(proxy.admin(), Some(&"admin2".to_string()));
    }

    #[test]
    fn test_proxy_change_admin_requires_admin() {
        let mut proxy = create_test_proxy();

        let result = proxy.change_admin("admin2".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_proxy_upgrade_history() {
        let mut proxy = create_test_proxy();
        proxy.set_admin("admin1".to_string()).unwrap();

        proxy.upgrade("impl_v2".to_string()).unwrap();
        proxy.upgrade("impl_v3".to_string()).unwrap();

        let history = proxy.upgrade_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].from_version, 1);
        assert_eq!(history[0].to_version, 2);
        assert_eq!(history[1].from_version, 2);
        assert_eq!(history[1].to_version, 3);
    }

    #[test]
    fn test_proxy_version_struct() {
        let info = VersionInfo {
            version: 1,
            implementation: "impl_v1".to_string(),
            deployed_at: 1000,
        };

        assert_eq!(info.version, 1);
        assert_eq!(info.implementation, "impl_v1");
        assert_eq!(info.deployed_at, 1000);
    }

    #[test]
    fn test_proxy_resume_operations_after_unpause() {
        let mut proxy = create_test_proxy();
        proxy.set_admin("admin1".to_string()).unwrap();

        proxy.pause().unwrap();
        assert!(proxy.is_paused());

        proxy.unpause().unwrap();
        assert!(!proxy.is_paused());

        let result = proxy.forward_call("normal_func", vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_proxy_multiple_upgrades_tracking() {
        let mut proxy = create_test_proxy();
        proxy.set_admin("admin1".to_string()).unwrap();

        for i in 2..=5 {
            proxy.upgrade(format!("impl_v{}", i)).unwrap();
        }

        assert_eq!(proxy.version(), 5);
        assert_eq!(proxy.upgrade_history().len(), 4);
    }

    #[test]
    fn test_proxy_admin_cannot_be_empty() {
        let mut proxy = create_test_proxy();

        let result = proxy.set_admin("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_proxy_admin_can_be_set() {
        let mut proxy = create_test_proxy();
        assert!(proxy.admin().is_none());

        proxy.set_admin("valid_admin".to_string()).unwrap();
        assert_eq!(proxy.admin(), Some(&"valid_admin".to_string()));
    }

    #[test]
    fn test_proxy_unpause_requires_admin() {
        let mut proxy = create_test_proxy();

        let result = proxy.unpause();
        assert!(result.is_err());
    }

    #[test]
    fn test_proxy_forward_returns_implementation_name() {
        let proxy = create_test_proxy();
        let result = proxy.forward_call("test_func", vec![]).unwrap();
        assert!(result.contains("impl_v1"));
    }

    #[test]
    fn test_proxy_upgrade_when_paused_fails() {
        let mut proxy = create_test_proxy();
        proxy.set_admin("admin1".to_string()).unwrap();
        proxy.pause().unwrap();

        let result = proxy.upgrade("impl_v2".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProxyError::ContractPaused));
    }
}
