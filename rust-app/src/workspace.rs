//! Capability-aware workspaces exposed by the Doctor AI shell.
//! A workspace is not advertised as operational until its required backend is connected.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceId {
    Chat,
    ImageStudio,
    VideoStudio,
    CodeStudio,
    Skills,
    Training,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    TextGeneration,
    ImageGeneration,
    ImageEditing,
    VideoGeneration,
    FrameInterpolation,
    CodeExecution,
    SkillImport,
    RetrievalKnowledge,
    FineTuning,
}

#[derive(Debug, Clone, Copy)]
pub struct WorkspaceSpec {
    pub id: WorkspaceId,
    pub title: &'static str,
    pub summary: &'static str,
    pub required: &'static [Capability],
    pub status: &'static str,
}

const TEXT: &[Capability] = &[Capability::TextGeneration];
const IMAGE: &[Capability] = &[Capability::ImageGeneration, Capability::ImageEditing];
const VIDEO: &[Capability] = &[Capability::VideoGeneration, Capability::FrameInterpolation];
const CODE: &[Capability] = &[Capability::CodeExecution];
const SKILLS: &[Capability] = &[Capability::SkillImport];
const TRAINING: &[Capability] = &[Capability::RetrievalKnowledge, Capability::FineTuning];

pub const WORKSPACES: &[WorkspaceSpec] = &[
    WorkspaceSpec { id: WorkspaceId::Chat, title: "Chat", summary: "Talk to the selected local model.", required: TEXT, status: "Connected to the existing local text engine." },
    WorkspaceSpec { id: WorkspaceId::ImageStudio, title: "Image Studio", summary: "Image generation, editing, and visual workflows.", required: IMAGE, status: "Workspace shell only — an image backend is not connected yet." },
    WorkspaceSpec { id: WorkspaceId::VideoStudio, title: "Video Studio", summary: "Video generation or frame-by-frame animation on limited hardware.", required: VIDEO, status: "Workspace shell only — WAN/video inference and frame assembly are not connected yet." },
    WorkspaceSpec { id: WorkspaceId::CodeStudio, title: "Code Studio", summary: "Build, review, and safely run code projects.", required: CODE, status: "Workspace shell only — sandboxed code execution is not connected yet." },
    WorkspaceSpec { id: WorkspaceId::Skills, title: "Skills", summary: "Import, inspect, enable, and manage reusable skills.", required: SKILLS, status: "Workspace shell only — validated skill import and execution are not connected yet." },
    WorkspaceSpec { id: WorkspaceId::Training, title: "Training & Knowledge", summary: "Add documents for retrieval, curate knowledge, and manage supported fine-tuning jobs.", required: TRAINING, status: "Workspace shell only — document indexing and fine-tuning pipelines are not connected yet." },
];

pub fn get(id: WorkspaceId) -> &'static WorkspaceSpec {
    WORKSPACES.iter().find(|item| item.id == id).expect("every workspace id must be registered")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_workspace_has_a_unique_id_and_nonempty_contract() {
        for (index, item) in WORKSPACES.iter().enumerate() {
            assert!(!item.title.is_empty());
            assert!(!item.summary.is_empty());
            assert!(!item.required.is_empty());
            assert!(!WORKSPACES[..index].iter().any(|prior| prior.id == item.id));
        }
    }

    #[test]
    fn video_workspace_declares_a_fallback_for_limited_hardware() {
        let video = get(WorkspaceId::VideoStudio);
        assert!(video.required.contains(&Capability::VideoGeneration));
        assert!(video.required.contains(&Capability::FrameInterpolation));
        assert!(video.summary.contains("frame-by-frame"));
    }

    #[test]
    fn chat_is_the_only_workspace_marked_connected() {
        assert_eq!(WORKSPACES.iter().filter(|w| w.status.starts_with("Connected")).count(), 1);
    }
}
