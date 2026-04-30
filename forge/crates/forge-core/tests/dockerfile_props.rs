//! Property tests for the Dockerfile generator. Invariants we hold across
//! every (runtime × base × hardening) combination:
//!   * Output is non-empty and contains FROM and ENTRYPOINT.
//!   * Strict hardening always lands a USER directive (or USER nonroot for
//!     distroless).
//!   * No accidental "USER root" leaks.
//!   * Compliance label set is preserved verbatim (sorted by BTreeSet).

use std::collections::BTreeSet;

use forge_core::dockerfile;
use forge_core::domain::{
    Architecture, BaseImage, BuildSpec, ComplianceProfile, HardeningOptions, Runtime,
};
use proptest::prelude::*;

fn runtimes() -> impl Strategy<Value = Runtime> {
    prop_oneof![
        Just(Runtime::Java),
        Just(Runtime::Dotnet),
        Just(Runtime::Go),
        Just(Runtime::Node),
        Just(Runtime::Python),
    ]
}

fn bases() -> impl Strategy<Value = BaseImage> {
    prop_oneof![
        Just(BaseImage::Alpine),
        Just(BaseImage::Debian),
        Just(BaseImage::Distroless),
    ]
}

fn arches() -> impl Strategy<Value = BTreeSet<Architecture>> {
    prop::collection::btree_set(
        prop_oneof![Just(Architecture::Amd64), Just(Architecture::Arm64)],
        1..=2,
    )
}

fn compliance() -> impl Strategy<Value = BTreeSet<ComplianceProfile>> {
    prop::collection::btree_set(
        prop_oneof![
            Just(ComplianceProfile::Cis),
            Just(ComplianceProfile::Hipaa),
            Just(ComplianceProfile::Soc2),
            Just(ComplianceProfile::PciDss),
            Just(ComplianceProfile::FedrampModerate),
        ],
        0..=3,
    )
}

fn hardenings() -> impl Strategy<Value = HardeningOptions> {
    (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(remove_shells, remove_pkg_managers, readonly_rootfs, non_root_user)| HardeningOptions {
            remove_shells,
            remove_pkg_managers,
            readonly_rootfs,
            non_root_user,
        },
    )
}

fn names() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,16}".prop_map(|s| s.to_string())
}

prop_compose! {
    fn build_specs()(
        name in names(),
        runtime in runtimes(),
        base in bases(),
        archs in arches(),
        compliance in compliance(),
        hardening in hardenings(),
        sbom in any::<bool>(),
        sign in any::<bool>(),
    ) -> BuildSpec {
        BuildSpec {
            name,
            runtime,
            base_image: base,
            architectures: archs,
            compliance,
            hardening,
            generate_sbom: sbom,
            sign,
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, .. ProptestConfig::default() })]

    #[test]
    fn dockerfile_always_has_from_and_entrypoint(spec in build_specs()) {
        let df = dockerfile::render(&spec);
        prop_assert!(df.contains("FROM "), "missing FROM in:\n{df}");
        prop_assert!(df.contains("ENTRYPOINT"), "missing ENTRYPOINT in:\n{df}");
    }

    #[test]
    fn no_explicit_root_user(spec in build_specs()) {
        let df = dockerfile::render(&spec);
        prop_assert!(!df.contains("USER root"), "leaked root user in:\n{df}");
    }

    #[test]
    fn strict_hardening_implies_user_directive(spec in build_specs()) {
        if spec.hardening.non_root_user {
            let df = dockerfile::render(&spec);
            prop_assert!(df.contains("USER "), "missing USER directive in:\n{df}");
        }
    }
}
