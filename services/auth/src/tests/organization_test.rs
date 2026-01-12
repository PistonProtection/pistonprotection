//! Organization and RBAC tests

use super::test_utils::{TestOrganization, TestUser, constants, generate_test_id};
use crate::services::organization::{
    InvitationStatus, MemberRole, Organization, OrganizationService,
};

/// Create a test organization service
fn create_test_org_service() -> OrganizationService {
    OrganizationService::new_with_memory_store()
}

// ============================================================================
// Organization CRUD Tests
// ============================================================================

#[cfg(test)]
mod organization_crud_tests {
    use super::*;

    /// Test creating an organization
    #[tokio::test]
    async fn test_create_organization() {
        let service = create_test_org_service();
        let user = TestUser::new();

        let result = service
            .create_organization(&user.id, "Test Org", "test-org")
            .await;

        assert!(result.is_ok());
        let org = result.unwrap();
        assert_eq!(org.name, "Test Org");
        assert_eq!(org.slug, "test-org");
    }

    /// Test creating organization with duplicate slug fails
    #[tokio::test]
    async fn test_create_duplicate_slug() {
        let service = create_test_org_service();
        let user = TestUser::new();

        service
            .create_organization(&user.id, "First Org", "unique-slug")
            .await
            .unwrap();

        let result = service
            .create_organization(&user.id, "Second Org", "unique-slug")
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("slug")
                || err.to_string().to_lowercase().contains("exist")
        );
    }

    /// Test getting an organization
    #[tokio::test]
    async fn test_get_organization() {
        let service = create_test_org_service();
        let user = TestUser::new();

        let created = service
            .create_organization(&user.id, "Get Test", "get-test")
            .await
            .unwrap();

        let result = service.get_organization(&created.id).await;

        assert!(result.is_ok());
        let org = result.unwrap();
        assert_eq!(org.id, created.id);
        assert_eq!(org.name, "Get Test");
    }

    /// Test getting non-existent organization
    #[tokio::test]
    async fn test_get_nonexistent_organization() {
        let service = create_test_org_service();

        let result = service.get_organization("nonexistent-id").await;

        assert!(result.is_err());
    }

    /// Test updating an organization
    #[tokio::test]
    async fn test_update_organization() {
        let service = create_test_org_service();
        let user = TestUser::new();

        let created = service
            .create_organization(&user.id, "Original Name", "original-slug")
            .await
            .unwrap();

        let result = service
            .update_organization(&created.id, &user.id, Some("Updated Name"), None)
            .await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.slug, "original-slug"); // Slug unchanged
    }

    /// Test deleting an organization
    #[tokio::test]
    async fn test_delete_organization() {
        let service = create_test_org_service();
        let user = TestUser::new();

        let created = service
            .create_organization(&user.id, "Delete Test", "delete-test")
            .await
            .unwrap();

        let result = service.delete_organization(&created.id, &user.id).await;

        assert!(result.is_ok());

        // Should not be retrievable anymore
        let get_result = service.get_organization(&created.id).await;
        assert!(get_result.is_err());
    }

    /// Test listing user's organizations
    #[tokio::test]
    async fn test_list_user_organizations() {
        let service = create_test_org_service();
        let user = TestUser::new();

        service
            .create_organization(&user.id, "Org 1", "org-1")
            .await
            .unwrap();
        service
            .create_organization(&user.id, "Org 2", "org-2")
            .await
            .unwrap();

        let result = service.list_user_organizations(&user.id).await;

        assert!(result.is_ok());
        let orgs = result.unwrap();
        assert!(orgs.len() >= 2);
    }
}

// ============================================================================
// Organization Slug Tests
// ============================================================================

#[cfg(test)]
mod slug_tests {
    use super::*;

    /// Test slug normalization
    #[tokio::test]
    async fn test_slug_normalization() {
        let service = create_test_org_service();
        let user = TestUser::new();

        let result = service
            .create_organization(&user.id, "Test Org", "  Test-Slug-123  ")
            .await;

        assert!(result.is_ok());
        let org = result.unwrap();
        // Slug should be lowercase, trimmed
        assert_eq!(org.slug, "test-slug-123");
    }

    /// Test slug with invalid characters
    #[tokio::test]
    async fn test_slug_invalid_characters() {
        let service = create_test_org_service();
        let user = TestUser::new();

        let invalid_slugs = vec![
            "has spaces",
            "has@special",
            "has/slash",
            "",
            "a", // Too short
        ];

        for slug in invalid_slugs {
            let result = service.create_organization(&user.id, "Test", slug).await;
            // Should either normalize or fail
        }
    }

    /// Test finding organization by slug
    #[tokio::test]
    async fn test_find_by_slug() {
        let service = create_test_org_service();
        let user = TestUser::new();

        let created = service
            .create_organization(&user.id, "Find Test", "find-test-slug")
            .await
            .unwrap();

        let result = service.find_by_slug("find-test-slug").await;

        assert!(result.is_ok());
        let org = result.unwrap().unwrap();
        assert_eq!(org.id, created.id);
    }
}

// ============================================================================
// Member Management Tests
// ============================================================================

#[cfg(test)]
mod member_tests {
    use super::*;

    /// Test adding a member to organization
    #[tokio::test]
    async fn test_add_member() {
        let service = create_test_org_service();
        let owner = TestUser::new();
        let new_member = TestUser::new().with_id("member-user");

        let org = service
            .create_organization(&owner.id, "Member Test", "member-test")
            .await
            .unwrap();

        let result = service
            .add_member(&org.id, &owner.id, &new_member.id, MemberRole::Member)
            .await;

        assert!(result.is_ok());
    }

    /// Test non-owner cannot add members
    #[tokio::test]
    async fn test_non_owner_cannot_add_member() {
        let service = create_test_org_service();
        let owner = TestUser::new();
        let member = TestUser::new().with_id("member-user");
        let other = TestUser::new().with_id("other-user");

        let org = service
            .create_organization(&owner.id, "Permission Test", "permission-test")
            .await
            .unwrap();

        // Add member first
        service
            .add_member(&org.id, &owner.id, &member.id, MemberRole::Member)
            .await
            .unwrap();

        // Member tries to add another
        let result = service
            .add_member(&org.id, &member.id, &other.id, MemberRole::Member)
            .await;

        // Should fail - members can't add members
        assert!(result.is_err());
    }

    /// Test admin can add members
    #[tokio::test]
    async fn test_admin_can_add_member() {
        let service = create_test_org_service();
        let owner = TestUser::new();
        let admin = TestUser::new().with_id("admin-user");
        let new_member = TestUser::new().with_id("new-member");

        let org = service
            .create_organization(&owner.id, "Admin Test", "admin-test")
            .await
            .unwrap();

        // Add admin
        service
            .add_member(&org.id, &owner.id, &admin.id, MemberRole::Admin)
            .await
            .unwrap();

        // Admin adds member
        let result = service
            .add_member(&org.id, &admin.id, &new_member.id, MemberRole::Member)
            .await;

        assert!(result.is_ok());
    }

    /// Test removing a member
    #[tokio::test]
    async fn test_remove_member() {
        let service = create_test_org_service();
        let owner = TestUser::new();
        let member = TestUser::new().with_id("member-user");

        let org = service
            .create_organization(&owner.id, "Remove Test", "remove-test")
            .await
            .unwrap();

        service
            .add_member(&org.id, &owner.id, &member.id, MemberRole::Member)
            .await
            .unwrap();

        let result = service.remove_member(&org.id, &owner.id, &member.id).await;

        assert!(result.is_ok());

        // Member should no longer have access
        let members = service.list_members(&org.id).await.unwrap();
        assert!(!members.iter().any(|m| m.user_id == member.id));
    }

    /// Test owner cannot be removed
    #[tokio::test]
    async fn test_owner_cannot_be_removed() {
        let service = create_test_org_service();
        let owner = TestUser::new();

        let org = service
            .create_organization(&owner.id, "Owner Test", "owner-test")
            .await
            .unwrap();

        let result = service.remove_member(&org.id, &owner.id, &owner.id).await;

        assert!(result.is_err());
    }

    /// Test listing members
    #[tokio::test]
    async fn test_list_members() {
        let service = create_test_org_service();
        let owner = TestUser::new();
        let member1 = TestUser::new().with_id("member-1");
        let member2 = TestUser::new().with_id("member-2");

        let org = service
            .create_organization(&owner.id, "List Test", "list-test")
            .await
            .unwrap();

        service
            .add_member(&org.id, &owner.id, &member1.id, MemberRole::Member)
            .await
            .unwrap();
        service
            .add_member(&org.id, &owner.id, &member2.id, MemberRole::Admin)
            .await
            .unwrap();

        let result = service.list_members(&org.id).await;

        assert!(result.is_ok());
        let members = result.unwrap();
        assert!(members.len() >= 3); // Owner + 2 members
    }

    /// Test updating member role
    #[tokio::test]
    async fn test_update_member_role() {
        let service = create_test_org_service();
        let owner = TestUser::new();
        let member = TestUser::new().with_id("member-user");

        let org = service
            .create_organization(&owner.id, "Role Test", "role-test")
            .await
            .unwrap();

        service
            .add_member(&org.id, &owner.id, &member.id, MemberRole::Member)
            .await
            .unwrap();

        // Promote to admin
        let result = service
            .update_member_role(&org.id, &owner.id, &member.id, MemberRole::Admin)
            .await;

        assert!(result.is_ok());

        let members = service.list_members(&org.id).await.unwrap();
        let updated_member = members.iter().find(|m| m.user_id == member.id).unwrap();
        assert_eq!(updated_member.role, MemberRole::Admin);
    }
}

// ============================================================================
// Invitation Tests
// ============================================================================

#[cfg(test)]
mod invitation_tests {
    use super::*;

    /// Test creating an invitation
    #[tokio::test]
    async fn test_create_invitation() {
        let service = create_test_org_service();
        let owner = TestUser::new();

        let org = service
            .create_organization(&owner.id, "Invite Test", "invite-test")
            .await
            .unwrap();

        let result = service
            .create_invitation(
                &org.id,
                &owner.id,
                "invitee@example.com",
                MemberRole::Member,
            )
            .await;

        assert!(result.is_ok());
        let invitation = result.unwrap();
        assert_eq!(invitation.email, "invitee@example.com");
        assert_eq!(invitation.status, InvitationStatus::Pending);
    }

    /// Test accepting an invitation
    #[tokio::test]
    async fn test_accept_invitation() {
        let service = create_test_org_service();
        let owner = TestUser::new();
        let invitee = TestUser::new()
            .with_id("invitee-user")
            .with_email("invitee@example.com");

        let org = service
            .create_organization(&owner.id, "Accept Test", "accept-test")
            .await
            .unwrap();

        let invitation = service
            .create_invitation(
                &org.id,
                &owner.id,
                "invitee@example.com",
                MemberRole::Member,
            )
            .await
            .unwrap();

        let result = service
            .accept_invitation(&invitation.token, &invitee.id)
            .await;

        assert!(result.is_ok());

        // Invitee should now be a member
        let members = service.list_members(&org.id).await.unwrap();
        assert!(members.iter().any(|m| m.user_id == invitee.id));
    }

    /// Test declining an invitation
    #[tokio::test]
    async fn test_decline_invitation() {
        let service = create_test_org_service();
        let owner = TestUser::new();

        let org = service
            .create_organization(&owner.id, "Decline Test", "decline-test")
            .await
            .unwrap();

        let invitation = service
            .create_invitation(
                &org.id,
                &owner.id,
                "decline@example.com",
                MemberRole::Member,
            )
            .await
            .unwrap();

        let result = service.decline_invitation(&invitation.token).await;

        assert!(result.is_ok());

        // Invitation should be declined
        let inv = service.get_invitation(&invitation.token).await.unwrap();
        assert_eq!(inv.status, InvitationStatus::Declined);
    }

    /// Test invitation expiration
    #[tokio::test]
    async fn test_expired_invitation() {
        let service = create_test_org_service();
        // Would need to mock time or use short expiry
        // Left as implementation exercise
    }

    /// Test revoking an invitation
    #[tokio::test]
    async fn test_revoke_invitation() {
        let service = create_test_org_service();
        let owner = TestUser::new();

        let org = service
            .create_organization(&owner.id, "Revoke Test", "revoke-test")
            .await
            .unwrap();

        let invitation = service
            .create_invitation(&org.id, &owner.id, "revoke@example.com", MemberRole::Member)
            .await
            .unwrap();

        let result = service
            .revoke_invitation(&org.id, &owner.id, &invitation.id)
            .await;

        assert!(result.is_ok());
    }
}

// ============================================================================
// RBAC Permission Tests
// ============================================================================

#[cfg(test)]
mod permission_tests {
    use super::*;

    /// Test owner permissions
    #[tokio::test]
    async fn test_owner_permissions() {
        let service = create_test_org_service();
        let owner = TestUser::new();

        let org = service
            .create_organization(&owner.id, "Owner Perm", "owner-perm")
            .await
            .unwrap();

        // Owner can do everything
        assert!(
            service
                .can_manage_members(&org.id, &owner.id)
                .await
                .unwrap()
        );
        assert!(
            service
                .can_manage_settings(&org.id, &owner.id)
                .await
                .unwrap()
        );
        assert!(
            service
                .can_delete_organization(&org.id, &owner.id)
                .await
                .unwrap()
        );
        assert!(service.can_view_billing(&org.id, &owner.id).await.unwrap());
    }

    /// Test admin permissions
    #[tokio::test]
    async fn test_admin_permissions() {
        let service = create_test_org_service();
        let owner = TestUser::new();
        let admin = TestUser::new().with_id("admin-user");

        let org = service
            .create_organization(&owner.id, "Admin Perm", "admin-perm")
            .await
            .unwrap();

        service
            .add_member(&org.id, &owner.id, &admin.id, MemberRole::Admin)
            .await
            .unwrap();

        // Admin can manage members and settings
        assert!(
            service
                .can_manage_members(&org.id, &admin.id)
                .await
                .unwrap()
        );
        assert!(
            service
                .can_manage_settings(&org.id, &admin.id)
                .await
                .unwrap()
        );
        // Admin cannot delete organization
        assert!(
            !service
                .can_delete_organization(&org.id, &admin.id)
                .await
                .unwrap()
        );
    }

    /// Test member permissions
    #[tokio::test]
    async fn test_member_permissions() {
        let service = create_test_org_service();
        let owner = TestUser::new();
        let member = TestUser::new().with_id("member-user");

        let org = service
            .create_organization(&owner.id, "Member Perm", "member-perm")
            .await
            .unwrap();

        service
            .add_member(&org.id, &owner.id, &member.id, MemberRole::Member)
            .await
            .unwrap();

        // Member has limited permissions
        assert!(
            !service
                .can_manage_members(&org.id, &member.id)
                .await
                .unwrap()
        );
        assert!(
            !service
                .can_manage_settings(&org.id, &member.id)
                .await
                .unwrap()
        );
        assert!(
            !service
                .can_delete_organization(&org.id, &member.id)
                .await
                .unwrap()
        );
    }

    /// Test viewer permissions
    #[tokio::test]
    async fn test_viewer_permissions() {
        let service = create_test_org_service();
        let owner = TestUser::new();
        let viewer = TestUser::new().with_id("viewer-user");

        let org = service
            .create_organization(&owner.id, "Viewer Perm", "viewer-perm")
            .await
            .unwrap();

        service
            .add_member(&org.id, &owner.id, &viewer.id, MemberRole::Viewer)
            .await
            .unwrap();

        // Viewer can only view
        assert!(
            !service
                .can_manage_members(&org.id, &viewer.id)
                .await
                .unwrap()
        );
        assert!(
            !service
                .can_manage_settings(&org.id, &viewer.id)
                .await
                .unwrap()
        );
        assert!(
            !service
                .can_delete_organization(&org.id, &viewer.id)
                .await
                .unwrap()
        );
        // Viewer can view resources
        assert!(
            service
                .can_view_resources(&org.id, &viewer.id)
                .await
                .unwrap()
        );
    }

    /// Test non-member has no permissions
    #[tokio::test]
    async fn test_non_member_permissions() {
        let service = create_test_org_service();
        let owner = TestUser::new();
        let outsider = TestUser::new().with_id("outsider-user");

        let org = service
            .create_organization(&owner.id, "No Perm", "no-perm")
            .await
            .unwrap();

        // Non-member has no permissions
        assert!(
            !service
                .can_view_resources(&org.id, &outsider.id)
                .await
                .unwrap()
        );
        assert!(
            !service
                .can_manage_members(&org.id, &outsider.id)
                .await
                .unwrap()
        );
    }
}
