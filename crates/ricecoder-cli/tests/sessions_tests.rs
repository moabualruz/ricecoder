use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sessions_command_creation() {
        let cmd = SessionsCommand::new(SessionsAction::List);
        assert!(matches!(cmd.action, SessionsAction::List));
    }

    #[test]
    fn test_create_session_action() {
        let cmd = SessionsCommand::new(SessionsAction::Create {
            name: "test".to_string(),
        });
        assert!(matches!(cmd.action, SessionsAction::Create { .. }));
    }

    #[test]
    fn test_delete_session_action() {
        let cmd = SessionsCommand::new(SessionsAction::Delete {
            id: "session-1".to_string(),
        });
        assert!(matches!(cmd.action, SessionsAction::Delete { .. }));
    }

    #[test]
    fn test_session_info_serialization() {
        let session = SessionInfo {
            id: "session-1".to_string(),
            name: "Test Session".to_string(),
            created_at: 1000,
            modified_at: 2000,
            message_count: 5,
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: SessionInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.name, deserialized.name);
        assert_eq!(session.message_count, deserialized.message_count);
    }

    #[test]
    fn test_rename_session_action() {
        let cmd = SessionsCommand::new(SessionsAction::Rename {
            id: "session-1".to_string(),
            name: "New Name".to_string(),
        });
        assert!(matches!(cmd.action, SessionsAction::Rename { .. }));
    }

    #[test]
    fn test_switch_session_action() {
        let cmd = SessionsCommand::new(SessionsAction::Switch {
            id: "session-1".to_string(),
        });
        assert!(matches!(cmd.action, SessionsAction::Switch { .. }));
    }

    #[test]
    fn test_info_session_action() {
        let cmd = SessionsCommand::new(SessionsAction::Info {
            id: "session-1".to_string(),
        });
        assert!(matches!(cmd.action, SessionsAction::Info { .. }));
    }

    #[test]
    fn test_share_action() {
        let cmd = SessionsCommand::new(SessionsAction::Share {
            expires_in: Some(3600),
            no_history: false,
            no_context: false,
        });
        assert!(matches!(cmd.action, SessionsAction::Share { .. }));
    }

    #[test]
    fn test_share_action_with_flags() {
        let cmd = SessionsCommand::new(SessionsAction::Share {
            expires_in: None,
            no_history: true,
            no_context: true,
        });
        assert!(matches!(cmd.action, SessionsAction::Share { .. }));
    }

    #[test]
    fn test_share_list_action() {
        let cmd = SessionsCommand::new(SessionsAction::ShareList);
        assert!(matches!(cmd.action, SessionsAction::ShareList));
    }

    #[test]
    fn test_share_revoke_action() {
        let cmd = SessionsCommand::new(SessionsAction::ShareRevoke {
            share_id: "share-123".to_string(),
        });
        assert!(matches!(cmd.action, SessionsAction::ShareRevoke { .. }));
    }

    #[test]
    fn test_share_info_action() {
        let cmd = SessionsCommand::new(SessionsAction::ShareInfo {
            share_id: "share-123".to_string(),
        });
        assert!(matches!(cmd.action, SessionsAction::ShareInfo { .. }));
    }

    #[test]
    fn test_share_command_with_expiration() {
        let cmd = SessionsCommand::new(SessionsAction::Share {
            expires_in: Some(3600),
            no_history: false,
            no_context: false,
        });

        match cmd.action {
            SessionsAction::Share {
                expires_in,
                no_history,
                no_context,
            } => {
                assert_eq!(expires_in, Some(3600));
                assert!(!no_history);
                assert!(!no_context);
            }
            _ => panic!("Expected Share action"),
        }
    }

    #[test]
    fn test_share_command_without_history() {
        let cmd = SessionsCommand::new(SessionsAction::Share {
            expires_in: None,
            no_history: true,
            no_context: false,
        });

        match cmd.action {
            SessionsAction::Share {
                expires_in,
                no_history,
                no_context,
            } => {
                assert_eq!(expires_in, None);
                assert!(no_history);
                assert!(!no_context);
            }
            _ => panic!("Expected Share action"),
        }
    }

    #[test]
    fn test_share_command_without_context() {
        let cmd = SessionsCommand::new(SessionsAction::Share {
            expires_in: None,
            no_history: false,
            no_context: true,
        });

        match cmd.action {
            SessionsAction::Share {
                expires_in,
                no_history,
                no_context,
            } => {
                assert_eq!(expires_in, None);
                assert!(!no_history);
                assert!(no_context);
            }
            _ => panic!("Expected Share action"),
        }
    }

    #[test]
    fn test_share_command_all_restrictions() {
        let cmd = SessionsCommand::new(SessionsAction::Share {
            expires_in: Some(7200),
            no_history: true,
            no_context: true,
        });

        match cmd.action {
            SessionsAction::Share {
                expires_in,
                no_history,
                no_context,
            } => {
                assert_eq!(expires_in, Some(7200));
                assert!(no_history);
                assert!(no_context);
            }
            _ => panic!("Expected Share action"),
        }
    }

    #[test]
    fn test_share_revoke_action_with_id() {
        let share_id = "test-share-id-12345";
        let cmd = SessionsCommand::new(SessionsAction::ShareRevoke {
            share_id: share_id.to_string(),
        });

        match cmd.action {
            SessionsAction::ShareRevoke { share_id: id } => {
                assert_eq!(id, share_id);
            }
            _ => panic!("Expected ShareRevoke action"),
        }
    }

    #[test]
    fn test_share_info_action_with_id() {
        let share_id = "test-share-id-67890";
        let cmd = SessionsCommand::new(SessionsAction::ShareInfo {
            share_id: share_id.to_string(),
        });

        match cmd.action {
            SessionsAction::ShareInfo { share_id: id } => {
                assert_eq!(id, share_id);
            }
            _ => panic!("Expected ShareInfo action"),
        }
    }

    #[test]
    fn test_session_info_with_zero_messages() {
        let session = SessionInfo {
            id: "session-1".to_string(),
            name: "Empty Session".to_string(),
            created_at: 1000,
            modified_at: 1000,
            message_count: 0,
        };

        assert_eq!(session.message_count, 0);
        assert_eq!(session.name, "Empty Session");
    }

    #[test]
    fn test_session_info_with_many_messages() {
        let session = SessionInfo {
            id: "session-2".to_string(),
            name: "Busy Session".to_string(),
            created_at: 1000,
            modified_at: 5000,
            message_count: 100,
        };

        assert_eq!(session.message_count, 100);
        assert!(session.modified_at > session.created_at);
    }

    #[test]
    fn test_share_permissions_all_enabled() {
        use ricecoder_sessions::SharePermissions;

        let perms = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        assert!(perms.read_only);
        assert!(perms.include_history);
        assert!(perms.include_context);
    }

    #[test]
    fn test_share_permissions_history_only() {
        use ricecoder_sessions::SharePermissions;

        let perms = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: false,
        };

        assert!(perms.read_only);
        assert!(perms.include_history);
        assert!(!perms.include_context);
    }

    #[test]
    fn test_share_permissions_context_only() {
        use ricecoder_sessions::SharePermissions;

        let perms = SharePermissions {
            read_only: true,
            include_history: false,
            include_context: true,
        };

        assert!(perms.read_only);
        assert!(!perms.include_history);
        assert!(perms.include_context);
    }

    #[test]
    fn test_share_permissions_nothing_included() {
        use ricecoder_sessions::SharePermissions;

        let perms = SharePermissions {
            read_only: true,
            include_history: false,
            include_context: false,
        };

        assert!(perms.read_only);
        assert!(!perms.include_history);
        assert!(!perms.include_context);
    }

    #[test]
    fn test_share_view_action() {
        let cmd = SessionsCommand::new(SessionsAction::ShareView {
            share_id: "share-123".to_string(),
        });
        assert!(matches!(cmd.action, SessionsAction::ShareView { .. }));
    }

    #[test]
    fn test_share_view_action_with_id() {
        let share_id = "test-share-view-id";
        let cmd = SessionsCommand::new(SessionsAction::ShareView {
            share_id: share_id.to_string(),
        });

        match cmd.action {
            SessionsAction::ShareView { share_id: id } => {
                assert_eq!(id, share_id);
            }
            _ => panic!("Expected ShareView action"),
        }
    }

    #[test]
    fn test_share_service_get_share() {
        use ricecoder_sessions::{SharePermissions, ShareService};

        let service = ShareService::new();

        // Generate a share
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        let share = service
            .generate_share_link("session-1", permissions, None)
            .expect("Failed to generate share");

        // Verify we can retrieve it
        let retrieved = service
            .get_share(&share.id)
            .expect("Failed to retrieve share");

        assert_eq!(retrieved.id, share.id);
        assert_eq!(retrieved.session_id, "session-1");
        assert!(retrieved.permissions.read_only);
    }

    #[test]
    fn test_share_service_get_nonexistent_share() {
        use ricecoder_sessions::ShareService;

        let service = ShareService::new();

        // Try to get a share that doesn't exist
        let result = service.get_share("nonexistent-share");

        assert!(result.is_err());
    }

    #[test]
    fn test_share_service_revoke_share() {
        use ricecoder_sessions::{SharePermissions, ShareService};

        let service = ShareService::new();

        // Generate a share
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        let share = service
            .generate_share_link("session-1", permissions, None)
            .expect("Failed to generate share");

        // Revoke it
        service
            .revoke_share(&share.id)
            .expect("Failed to revoke share");

        // Verify it's gone
        let result = service.get_share(&share.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_share_service_list_shares() {
        use ricecoder_sessions::{SharePermissions, ShareService};

        let service = ShareService::new();

        // Generate multiple shares
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        let share1 = service
            .generate_share_link("session-1", permissions.clone(), None)
            .expect("Failed to generate share 1");

        let share2 = service
            .generate_share_link("session-2", permissions.clone(), None)
            .expect("Failed to generate share 2");

        // List all shares
        let shares = service.list_shares().expect("Failed to list shares");

        assert!(shares.len() >= 2);
        assert!(shares.iter().any(|s| s.id == share1.id));
        assert!(shares.iter().any(|s| s.id == share2.id));
    }

    #[test]
    fn test_share_service_create_shared_session_view_with_history() {
        use ricecoder_sessions::{
            Message, MessageRole, Session, SessionContext, SessionMode, SharePermissions,
            ShareService,
        };

        let service = ShareService::new();

        // Create a session with messages
        let mut session = Session::new(
            "Test Session".to_string(),
            SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat),
        );

        session
            .history
            .push(Message::new(MessageRole::User, "Hello".to_string()));

        session.history.push(Message::new(
            MessageRole::Assistant,
            "Hi there!".to_string(),
        ));

        // Create a view with history included
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        let view = service.create_shared_session_view(&session, &permissions);

        assert_eq!(view.history.len(), 2);
    }

    #[test]
    fn test_share_service_create_shared_session_view_without_history() {
        use ricecoder_sessions::{
            Message, MessageRole, Session, SessionContext, SessionMode, SharePermissions,
            ShareService,
        };

        let service = ShareService::new();

        // Create a session with messages
        let mut session = Session::new(
            "Test Session".to_string(),
            SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat),
        );

        session
            .history
            .push(Message::new(MessageRole::User, "Hello".to_string()));

        session.history.push(Message::new(
            MessageRole::Assistant,
            "Hi there!".to_string(),
        ));

        // Create a view with history excluded
        let permissions = SharePermissions {
            read_only: true,
            include_history: false,
            include_context: true,
        };

        let view = service.create_shared_session_view(&session, &permissions);

        assert_eq!(view.history.len(), 0);
    }

    #[test]
    fn test_share_service_create_shared_session_view_without_context() {
        use ricecoder_sessions::{
            Session, SessionContext, SessionMode, SharePermissions, ShareService,
        };

        let service = ShareService::new();

        // Create a session with context
        let mut session = Session::new(
            "Test Session".to_string(),
            SessionContext::new("openai".to_string(), "gpt-4".to_string(), SessionMode::Chat),
        );

        session.context.files.push("file1.rs".to_string());
        session.context.files.push("file2.rs".to_string());

        // Create a view with context excluded
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: false,
        };

        let view = service.create_shared_session_view(&session, &permissions);

        assert_eq!(view.context.files.len(), 0);
    }

    #[test]
    fn test_share_service_list_shares_for_session() {
        use ricecoder_sessions::{SharePermissions, ShareService};

        let service = ShareService::new();

        // Generate shares for different sessions
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        let share1 = service
            .generate_share_link("session-1", permissions.clone(), None)
            .expect("Failed to generate share 1");

        let share2 = service
            .generate_share_link("session-1", permissions.clone(), None)
            .expect("Failed to generate share 2");

        let share3 = service
            .generate_share_link("session-2", permissions.clone(), None)
            .expect("Failed to generate share 3");

        // List shares for session-1
        let session1_shares = service
            .list_shares_for_session("session-1")
            .expect("Failed to list shares for session-1");

        assert_eq!(session1_shares.len(), 2);
        assert!(session1_shares.iter().any(|s| s.id == share1.id));
        assert!(session1_shares.iter().any(|s| s.id == share2.id));
        assert!(!session1_shares.iter().any(|s| s.id == share3.id));
    }

    #[test]
    fn test_share_service_invalidate_session_shares() {
        use ricecoder_sessions::{SharePermissions, ShareService};

        let service = ShareService::new();

        // Generate shares for a session
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        let share1 = service
            .generate_share_link("session-1", permissions.clone(), None)
            .expect("Failed to generate share 1");

        let share2 = service
            .generate_share_link("session-1", permissions.clone(), None)
            .expect("Failed to generate share 2");

        // Invalidate all shares for session-1
        let invalidated = service
            .invalidate_session_shares("session-1")
            .expect("Failed to invalidate shares");

        assert_eq!(invalidated, 2);

        // Verify shares are gone
        let result1 = service.get_share(&share1.id);
        let result2 = service.get_share(&share2.id);
        assert!(result1.is_err());
        assert!(result2.is_err());
    }

    #[test]
    fn test_share_service_read_only_enforcement() {
        use ricecoder_sessions::{SharePermissions, ShareService};

        let service = ShareService::new();

        // Generate a share with read_only=true
        let permissions = SharePermissions {
            read_only: true,
            include_history: true,
            include_context: true,
        };

        let share = service
            .generate_share_link("session-1", permissions, None)
            .expect("Failed to generate share");

        // Verify read_only is enforced
        assert!(share.permissions.read_only);
    }
}
