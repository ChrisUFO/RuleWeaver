#![allow(dead_code)]

pub struct FeatureFlag {
    pub key: &'static str,
    pub default: bool,
    pub description: &'static str,
}

pub const NATIVE_SKILL_SYNC: FeatureFlag = FeatureFlag {
    key: "native_skill_sync",
    default: true,
    description: "Native skill distribution to AI tool directories with adapter targeting",
};

pub const UNIFIED_ARTIFACT_STATUS: FeatureFlag = FeatureFlag {
    key: "unified_artifact_status",
    default: true,
    description: "Unified artifact status view with repair actions across rules/commands/skills",
};

pub const EXECUTION_REDACTION: FeatureFlag = FeatureFlag {
    key: "execution_redaction",
    default: true,
    description: "Secret redaction in command execution output",
};

pub const ALL_FLAGS: &[&FeatureFlag] = &[
    &NATIVE_SKILL_SYNC,
    &UNIFIED_ARTIFACT_STATUS,
    &EXECUTION_REDACTION,
];

impl FeatureFlag {
    pub fn is_enabled(&self) -> bool {
        self.default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_flags_default_enabled() {
        for flag in ALL_FLAGS {
            assert!(
                flag.is_enabled(),
                "Feature flag {} should be enabled by default",
                flag.key
            );
        }
    }

    #[test]
    fn test_native_skill_sync_flag() {
        assert!(NATIVE_SKILL_SYNC.is_enabled());
        assert_eq!(NATIVE_SKILL_SYNC.key, "native_skill_sync");
    }

    #[test]
    fn test_unified_artifact_status_flag() {
        assert!(UNIFIED_ARTIFACT_STATUS.is_enabled());
        assert_eq!(UNIFIED_ARTIFACT_STATUS.key, "unified_artifact_status");
    }

    #[test]
    fn test_execution_redaction_flag() {
        assert!(EXECUTION_REDACTION.is_enabled());
        assert_eq!(EXECUTION_REDACTION.key, "execution_redaction");
    }
}
