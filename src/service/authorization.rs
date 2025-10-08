// src/service/authorization.rs

use crate::domain::errors::{FhirError, FhirResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// User roles in the FHIR system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// System administrator - full access
    Admin,
    /// Healthcare provider - can read/write patient data
    Clinician,
    /// Patient - can only access their own data
    Patient,
    /// System service - for internal operations
    System,
}

/// Permission types for resources
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Read access to resources
    Read,
    /// Create new resources
    Create,
    /// Update existing resources
    Update,
    /// Delete resources (soft delete)
    Delete,
    /// Search for resources
    Search,
    /// Access resource history
    ReadHistory,
}

/// Security context containing user identity and permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Unique user identifier
    pub user_id: String,

    /// User's roles
    pub roles: HashSet<Role>,

    /// Optional patient ID if the user is a patient
    pub patient_id: Option<String>,

    /// Optional organization/tenant ID for multi-tenancy
    pub organization_id: Option<String>,

    /// Additional claims or attributes
    pub claims: std::collections::HashMap<String, String>,
}

impl SecurityContext {
    /// Create a new security context
    pub fn new(user_id: String, roles: Vec<Role>) -> Self {
        Self {
            user_id,
            roles: roles.into_iter().collect(),
            patient_id: None,
            organization_id: None,
            claims: std::collections::HashMap::new(),
        }
    }

    /// Create an admin security context
    pub fn admin(user_id: String) -> Self {
        Self::new(user_id, vec![Role::Admin])
    }

    /// Create a clinician security context
    pub fn clinician(user_id: String, organization_id: Option<String>) -> Self {
        let mut ctx = Self::new(user_id, vec![Role::Clinician]);
        ctx.organization_id = organization_id;
        ctx
    }

    /// Create a patient security context
    pub fn patient(user_id: String, patient_id: String) -> Self {
        let mut ctx = Self::new(user_id, vec![Role::Patient]);
        ctx.patient_id = Some(patient_id);
        ctx
    }

    /// Create a system security context (for internal operations)
    pub fn system() -> Self {
        Self::new("system".to_string(), vec![Role::System])
    }

    /// Check if the user has a specific role
    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }

    /// Check if the user has any of the specified roles
    pub fn has_any_role(&self, roles: &[Role]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    /// Check if the user has all of the specified roles
    pub fn has_all_roles(&self, roles: &[Role]) -> bool {
        roles.iter().all(|role| self.has_role(role))
    }

    /// Check if this is a patient context
    pub fn is_patient(&self) -> bool {
        self.has_role(&Role::Patient)
    }

    /// Check if this is an admin context
    pub fn is_admin(&self) -> bool {
        self.has_role(&Role::Admin)
    }

    /// Check if this is a clinician context
    pub fn is_clinician(&self) -> bool {
        self.has_role(&Role::Clinician)
    }

    /// Check if this is a system context
    pub fn is_system(&self) -> bool {
        self.has_role(&Role::System)
    }

    /// Get the patient ID if this is a patient context
    pub fn get_patient_id(&self) -> Option<&str> {
        self.patient_id.as_deref()
    }
}

/// Trait for authorization checks on resources
pub trait Authorizer {
    /// Check if the user can perform an action on a resource type
    fn check_permission(
        &self,
        context: &SecurityContext,
        resource_type: &str,
        permission: Permission,
    ) -> FhirResult<()>;

    /// Check if the user can access a specific resource
    fn check_resource_access(
        &self,
        context: &SecurityContext,
        resource_type: &str,
        resource_id: &str,
        permission: Permission,
    ) -> FhirResult<()>;

    /// Check if the user can access resources in a patient compartment
    fn check_patient_compartment_access(
        &self,
        context: &SecurityContext,
        patient_id: &str,
        permission: Permission,
    ) -> FhirResult<()>;
}

/// Default authorization implementation
#[derive(Debug, Clone)]
pub struct DefaultAuthorizer;

impl DefaultAuthorizer {
    pub fn new() -> Self {
        Self
    }

    /// Check if a role has permission for a resource type
    fn role_has_permission(role: &Role, permission: &Permission) -> bool {
        match role {
            Role::Admin | Role::System => true, // Admin and System have all permissions
            Role::Clinician => matches!(
                permission,
                Permission::Read
                    | Permission::Create
                    | Permission::Update
                    | Permission::Search
                    | Permission::ReadHistory
            ),
            Role::Patient => matches!(
                permission,
                Permission::Read | Permission::Search | Permission::ReadHistory
            ),
        }
    }
}

impl Default for DefaultAuthorizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Authorizer for DefaultAuthorizer {
    fn check_permission(
        &self,
        context: &SecurityContext,
        resource_type: &str,
        permission: Permission,
    ) -> FhirResult<()> {
        // Check if any of the user's roles have the required permission
        let has_permission = context.roles.iter().any(|role| {
            Self::role_has_permission(role, &permission)
        });

        if has_permission {
            Ok(())
        } else {
            Err(FhirError::Forbidden {
                message: format!(
                    "User {} does not have permission {:?} for resource type {}",
                    context.user_id, permission, resource_type
                ),
            })
        }
    }

    fn check_resource_access(
        &self,
        context: &SecurityContext,
        resource_type: &str,
        resource_id: &str,
        permission: Permission,
    ) -> FhirResult<()> {
        // First check if the user has the permission at all
        self.check_permission(context, resource_type, permission)?;

        // Admins and system can access everything
        if context.is_admin() || context.is_system() {
            return Ok(());
        }

        // For patients, they can only access their own patient resource
        if context.is_patient() && resource_type == "Patient" {
            if let Some(patient_id) = context.get_patient_id() {
                if patient_id == resource_id {
                    return Ok(());
                }
            }
            return Err(FhirError::Forbidden {
                message: format!(
                    "Patient {} cannot access Patient resource {}",
                    context.user_id, resource_id
                ),
            });
        }

        // Clinicians can access all resources (additional organization-based filtering can be added)
        Ok(())
    }

    fn check_patient_compartment_access(
        &self,
        context: &SecurityContext,
        patient_id: &str,
        permission: Permission,
    ) -> FhirResult<()> {
        // Admins and system can access everything
        if context.is_admin() || context.is_system() {
            return Ok(());
        }

        // Check if user has the base permission
        self.check_permission(context, "Patient", permission)?;

        // Patients can only access their own compartment
        if context.is_patient() {
            if let Some(ctx_patient_id) = context.get_patient_id() {
                if ctx_patient_id == patient_id {
                    return Ok(());
                }
            }
            return Err(FhirError::Forbidden {
                message: format!(
                    "Patient {} cannot access patient compartment for patient {}",
                    context.user_id, patient_id
                ),
            });
        }

        // Clinicians can access all patient compartments
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_context_creation() {
        let admin_ctx = SecurityContext::admin("admin1".to_string());
        assert!(admin_ctx.is_admin());
        assert!(!admin_ctx.is_patient());

        let patient_ctx = SecurityContext::patient("user1".to_string(), "patient1".to_string());
        assert!(patient_ctx.is_patient());
        assert_eq!(patient_ctx.get_patient_id(), Some("patient1"));

        let clinician_ctx = SecurityContext::clinician("doc1".to_string(), Some("org1".to_string()));
        assert!(clinician_ctx.is_clinician());
    }

    #[test]
    fn test_role_permissions() {
        let authorizer = DefaultAuthorizer::new();

        // Admin has all permissions
        let admin_ctx = SecurityContext::admin("admin1".to_string());
        assert!(authorizer.check_permission(&admin_ctx, "Patient", Permission::Delete).is_ok());

        // Clinician cannot delete
        let clinician_ctx = SecurityContext::clinician("doc1".to_string(), None);
        assert!(authorizer.check_permission(&clinician_ctx, "Patient", Permission::Read).is_ok());
        assert!(authorizer.check_permission(&clinician_ctx, "Patient", Permission::Delete).is_err());

        // Patient can only read
        let patient_ctx = SecurityContext::patient("user1".to_string(), "patient1".to_string());
        assert!(authorizer.check_permission(&patient_ctx, "Patient", Permission::Read).is_ok());
        assert!(authorizer.check_permission(&patient_ctx, "Patient", Permission::Create).is_err());
    }

    #[test]
    fn test_patient_compartment_access() {
        let authorizer = DefaultAuthorizer::new();

        // Patient can access their own compartment
        let patient_ctx = SecurityContext::patient("user1".to_string(), "patient1".to_string());
        assert!(authorizer.check_patient_compartment_access(&patient_ctx, "patient1", Permission::Read).is_ok());

        // Patient cannot access another patient's compartment
        assert!(authorizer.check_patient_compartment_access(&patient_ctx, "patient2", Permission::Read).is_err());

        // Clinician can access any compartment
        let clinician_ctx = SecurityContext::clinician("doc1".to_string(), None);
        assert!(authorizer.check_patient_compartment_access(&clinician_ctx, "patient1", Permission::Read).is_ok());
        assert!(authorizer.check_patient_compartment_access(&clinician_ctx, "patient2", Permission::Read).is_ok());
    }
}
