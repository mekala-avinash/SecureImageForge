use std::collections::BTreeSet;

use dioxus::prelude::*;

use forge_core::domain::{
    Architecture, BaseImage, BuildSpec, ComplianceProfile, HardeningOptions, Runtime,
};

use crate::services::orchestration;
use crate::state::use_app_state;
use crate::views::Route;

#[component]
pub fn NewBuild(route: Signal<Route>) -> Element {
    let state = use_app_state();

    let mut name = use_signal(String::new);
    let mut runtime = use_signal(|| RuntimeChoice::Java);
    let mut base = use_signal(|| BaseChoice::Alpine);
    let mut amd64 = use_signal(|| true);
    let mut arm64 = use_signal(|| false);
    let mut sign = use_signal(|| true);
    let mut sbom = use_signal(|| true);
    let mut cis = use_signal(|| true);
    let mut hipaa = use_signal(|| false);
    let mut soc2 = use_signal(|| false);
    let mut error = use_signal(String::new);
    let mut building = use_signal(|| false);

    let mut submit = move || {
        building.set(true);
        let n = name.read().trim().to_string();
        if n.is_empty() {
            building.set(false);
            error.set("Name is required".into());
            return;
        }
        let mut archs = BTreeSet::new();
        if *amd64.read() {
            archs.insert(Architecture::Amd64);
        }
        if *arm64.read() {
            archs.insert(Architecture::Arm64);
        }
        if archs.is_empty() {
            building.set(false);
            error.set("Pick at least one architecture".into());
            return;
        }
        let mut compliance = BTreeSet::new();
        if *cis.read() {
            compliance.insert(ComplianceProfile::Cis);
        }
        if *hipaa.read() {
            compliance.insert(ComplianceProfile::Hipaa);
        }
        if *soc2.read() {
            compliance.insert(ComplianceProfile::Soc2);
        }

        let spec = BuildSpec {
            name: n,
            runtime: (*runtime.read()).into(),
            base_image: (*base.read()).into(),
            architectures: archs,
            compliance,
            hardening: HardeningOptions::strict(),
            generate_sbom: *sbom.read(),
            sign: *sign.read(),
        };
        orchestration::start_build(&state, spec);
        error.set(String::new());
        route.set(Route::Builds);
    };

    rsx! {
        section {
            class: "view",
            header { class: "view-header", h1 { "Initialize New Forge" } }
            form {
                class: "glass-card form",
                style: "max-width: 800px;",
                onsubmit: move |_| { submit(); },

                div { class: "form-row",
                    label { "Forge Identity (Name)" }
                    input {
                        r#type: "text",
                        placeholder: "secure-service-alpha",
                        value: "{name.read()}",
                        oninput: move |evt| name.set(evt.value()),
                    }
                }

                div { 
                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 24px;",
                    div { class: "form-row",
                        label { "Target Runtime" }
                        select {
                            oninput: move |evt| runtime.set(RuntimeChoice::parse(&evt.value())),
                            option { value: "java",   selected: matches!(*runtime.read(), RuntimeChoice::Java),   "Java (OpenJRE)" }
                            option { value: "dotnet", selected: matches!(*runtime.read(), RuntimeChoice::Dotnet), ".NET Core" }
                            option { value: "go",     selected: matches!(*runtime.read(), RuntimeChoice::Go),     "Go (Static)" }
                            option { value: "node",   selected: matches!(*runtime.read(), RuntimeChoice::Node),   "Node.js" }
                            option { value: "python", selected: matches!(*runtime.read(), RuntimeChoice::Python), "Python 3" }
                        }
                    }

                    div { class: "form-row",
                        label { "Base Matrix" }
                        select {
                            oninput: move |evt| base.set(BaseChoice::parse(&evt.value())),
                            option { value: "alpine",     selected: matches!(*base.read(), BaseChoice::Alpine),     "Alpine (Minimal)" }
                            option { value: "debian",     selected: matches!(*base.read(), BaseChoice::Debian),     "Debian (Stable)" }
                            option { value: "distroless", selected: matches!(*base.read(), BaseChoice::Distroless), "Distroless (Hardened)" }
                        }
                    }
                }

                div {
                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 24px;",
                    fieldset { 
                        legend { "Architecture Support" }
                        label { class: "checkbox",
                            input { r#type: "checkbox", checked: *amd64.read(),
                                oninput: move |evt| amd64.set(evt.value().parse().unwrap_or(false)) }
                            "linux/amd64 (x86_64)"
                        }
                        label { class: "checkbox",
                            input { r#type: "checkbox", checked: *arm64.read(),
                                oninput: move |evt| arm64.set(evt.value().parse().unwrap_or(false)) }
                            "linux/arm64 (Silicon)"
                        }
                    }

                    fieldset { 
                        legend { "Compliance Profiles" }
                        label { class: "checkbox",
                            input { r#type: "checkbox", checked: *cis.read(),
                                oninput: move |evt| cis.set(evt.value().parse().unwrap_or(false)) }
                            "CIS (Docker Benchmark)"
                        }
                        label { class: "checkbox",
                            input { r#type: "checkbox", checked: *hipaa.read(),
                                oninput: move |evt| hipaa.set(evt.value().parse().unwrap_or(false)) }
                            "HIPAA (Healthcare)"
                        }
                        label { class: "checkbox",
                            input { r#type: "checkbox", checked: *soc2.read(),
                                oninput: move |evt| soc2.set(evt.value().parse().unwrap_or(false)) }
                            "SOC2 (Enterprise)"
                        }
                    }
                }

                fieldset { 
                    legend { "Artifact Generation" }
                    div {
                        style: "display: flex; gap: 40px;",
                        label { class: "checkbox",
                            input { r#type: "checkbox", checked: *sbom.read(),
                                oninput: move |evt| sbom.set(evt.value().parse().unwrap_or(false)) }
                            "Generate SBOM (CycloneDX)"
                        }
                        label { class: "checkbox",
                            input { r#type: "checkbox", checked: *sign.read(),
                                oninput: move |evt| sign.set(evt.value().parse().unwrap_or(false)) }
                            "Sign Image (Cosign/HCP)"
                        }
                    }
                }

                if !error.read().is_empty() {
                    p { style: "color: var(--signal); font-weight: 600;", "{error.read()}" }
                }

                div { class: "form-actions",
                    button { class: "btn-ghost", r#type: "button",
                        onclick: move |_| route.set(Route::Builds), "Abort" }
                    button { class: "btn-primary", r#type: "submit", disabled: *building.read(),
                        if *building.read() { "Starting Forge..." } else { "Initiate Forge" }
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum RuntimeChoice {
    Java,
    Dotnet,
    Go,
    Node,
    Python,
}

impl RuntimeChoice {
    fn parse(s: &str) -> Self {
        match s {
            "dotnet" => Self::Dotnet,
            "go" => Self::Go,
            "node" => Self::Node,
            "python" => Self::Python,
            _ => Self::Java,
        }
    }
}

impl From<RuntimeChoice> for Runtime {
    fn from(c: RuntimeChoice) -> Self {
        match c {
            RuntimeChoice::Java => Runtime::Java,
            RuntimeChoice::Dotnet => Runtime::Dotnet,
            RuntimeChoice::Go => Runtime::Go,
            RuntimeChoice::Node => Runtime::Node,
            RuntimeChoice::Python => Runtime::Python,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum BaseChoice {
    Alpine,
    Debian,
    Distroless,
}

impl BaseChoice {
    fn parse(s: &str) -> Self {
        match s {
            "debian" => Self::Debian,
            "distroless" => Self::Distroless,
            _ => Self::Alpine,
        }
    }
}

impl From<BaseChoice> for BaseImage {
    fn from(c: BaseChoice) -> Self {
        match c {
            BaseChoice::Alpine => BaseImage::Alpine,
            BaseChoice::Debian => BaseImage::Debian,
            BaseChoice::Distroless => BaseImage::Distroless,
        }
    }
}
