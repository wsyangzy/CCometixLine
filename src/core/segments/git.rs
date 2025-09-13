use super::{Segment, SegmentData};
use crate::config::{InputData, SegmentId, GitSegmentConfig, GitStatusFormat};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug)]
pub struct GitInfo {
    pub branch: String,
    pub status: GitStatus,
    pub status_counts: GitStatusCounts,
    pub ahead: u32,
    pub behind: u32,
    pub sha: Option<String>,
    pub stash_count: Option<u32>,
    pub tag: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum GitStatus {
    Clean,
    Dirty,
    Conflicts,
}

#[derive(Debug, Default)]
pub struct GitStatusCounts {
    pub added: u32,
    pub modified: u32,
    pub deleted: u32,
}

pub struct GitSegment {
    config: GitSegmentConfig,
}

impl Default for GitSegment {
    fn default() -> Self {
        Self::new()
    }
}

impl GitSegment {
    pub fn new() -> Self {
        Self {
            config: GitSegmentConfig::default(),
        }
    }
    
    pub fn with_config(options: &HashMap<String, serde_json::Value>) -> Self {
        Self {
            config: GitSegmentConfig::from_options(options),
        }
    }

    fn get_git_info(&self, working_dir: &str) -> Option<GitInfo> {
        if !self.is_git_repository(working_dir) {
            return None;
        }

        let branch = self
            .get_branch(working_dir)
            .unwrap_or_else(|| "detached".to_string());
        let (status, status_counts) = self.get_status_with_counts(working_dir);
        let (ahead, behind) = self.get_ahead_behind(working_dir);
        let sha = if self.config.show_sha {
            self.get_sha(working_dir, self.config.sha_length)
        } else {
            None
        };
        
        let stash_count = if self.config.show_stash {
            self.get_stash_count(working_dir)
        } else {
            None
        };
        
        let tag = if self.config.show_tag {
            self.get_latest_tag(working_dir)
        } else {
            None
        };

        Some(GitInfo {
            branch: self.format_branch_name(branch),
            status,
            status_counts,
            ahead,
            behind,
            sha,
            stash_count,
            tag,
        })
    }

    fn is_git_repository(&self, working_dir: &str) -> bool {
        Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(working_dir)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn get_branch(&self, working_dir: &str) -> Option<String> {
        if let Ok(output) = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(working_dir)
            .output()
        {
            if output.status.success() {
                let branch = String::from_utf8(output.stdout).ok()?.trim().to_string();
                if !branch.is_empty() {
                    return Some(branch);
                }
            }
        }

        if let Ok(output) = Command::new("git")
            .args(["symbolic-ref", "--short", "HEAD"])
            .current_dir(working_dir)
            .output()
        {
            if output.status.success() {
                let branch = String::from_utf8(output.stdout).ok()?.trim().to_string();
                if !branch.is_empty() {
                    return Some(branch);
                }
            }
        }

        None
    }

    fn get_status_with_counts(&self, working_dir: &str) -> (GitStatus, GitStatusCounts) {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(working_dir)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let status_text = String::from_utf8(output.stdout).unwrap_or_default();

                if status_text.trim().is_empty() {
                    return (GitStatus::Clean, GitStatusCounts::default());
                }

                let mut counts = GitStatusCounts::default();
                let mut has_conflicts = false;

                for line in status_text.lines() {
                    if line.len() < 2 { continue; }
                    
                    let staged = line.chars().nth(0).unwrap_or(' ');
                    let unstaged = line.chars().nth(1).unwrap_or(' ');
                    
                    // Check for conflicts
                    if staged == 'U' && unstaged == 'U' 
                        || staged == 'A' && unstaged == 'A'
                        || staged == 'D' && unstaged == 'D' {
                        has_conflicts = true;
                        continue;
                    }
                    
                    // Count changes
                    if staged == 'A' || unstaged == 'A' {
                        counts.added += 1;
                    } else if staged == 'M' || unstaged == 'M' {
                        counts.modified += 1;
                    } else if staged == 'D' || unstaged == 'D' {
                        counts.deleted += 1;
                    } else if staged != ' ' || unstaged != ' ' {
                        // Other changes count as modified
                        counts.modified += 1;
                    }
                }

                let status = if has_conflicts {
                    GitStatus::Conflicts
                } else {
                    GitStatus::Dirty
                };
                
                (status, counts)
            }
            _ => (GitStatus::Clean, GitStatusCounts::default()),
        }
    }

    fn get_ahead_behind(&self, working_dir: &str) -> (u32, u32) {
        let ahead = self.get_commit_count(working_dir, "@{u}..HEAD");
        let behind = self.get_commit_count(working_dir, "HEAD..@{u}");
        (ahead, behind)
    }

    fn get_commit_count(&self, working_dir: &str, range: &str) -> u32 {
        let output = Command::new("git")
            .args(["rev-list", "--count", range])
            .current_dir(working_dir)
            .output();

        match output {
            Ok(output) if output.status.success() => String::from_utf8(output.stdout)
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0),
            _ => 0,
        }
    }

    fn get_sha(&self, working_dir: &str, length: u8) -> Option<String> {
        let output = Command::new("git")
            .args(["rev-parse", &format!("--short={}", length), "HEAD"])
            .current_dir(working_dir)
            .output()
            .ok()?;

        if output.status.success() {
            let sha = String::from_utf8(output.stdout).ok()?.trim().to_string();
            if sha.is_empty() {
                None
            } else {
                Some(sha)
            }
        } else {
            None
        }
    }
    
    fn get_stash_count(&self, working_dir: &str) -> Option<u32> {
        let output = Command::new("git")
            .args(["stash", "list"])
            .current_dir(working_dir)
            .output()
            .ok()?;

        if output.status.success() {
            let stash_list = String::from_utf8(output.stdout).ok()?;
            let count = stash_list.lines().count() as u32;
            if count > 0 { Some(count) } else { None }
        } else {
            None
        }
    }
    
    fn get_latest_tag(&self, working_dir: &str) -> Option<String> {
        let output = Command::new("git")
            .args(["describe", "--tags", "--abbrev=0"])
            .current_dir(working_dir)
            .output()
            .ok()?;

        if output.status.success() {
            let tag = String::from_utf8(output.stdout).ok()?.trim().to_string();
            if tag.is_empty() { None } else { Some(tag) }
        } else {
            None
        }
    }
    
    fn format_branch_name(&self, branch: String) -> String {
        if branch.len() > self.config.branch_max_length {
            let truncated = &branch[..self.config.branch_max_length.saturating_sub(3)];
            format!("{}...", truncated)
        } else {
            branch
        }
    }
    
    fn format_status(&self, status: &GitStatus, changes: &GitStatusCounts) -> String {
        if self.config.hide_clean_status && *status == GitStatus::Clean {
            return String::new();
        }
        
        match self.config.status_format {
            GitStatusFormat::Symbols => {
                match status {
                    GitStatus::Clean => "✓".to_string(),
                    GitStatus::Dirty => {
                        let mut parts = Vec::new();
                        if changes.added > 0 { parts.push(format!("+{}", changes.added)); }
                        if changes.modified > 0 { parts.push(format!("~{}", changes.modified)); }
                        if changes.deleted > 0 { parts.push(format!("-{}", changes.deleted)); }
                        if parts.is_empty() { "●".to_string() } else { parts.join(" ") }
                    },
                    GitStatus::Conflicts => "⚠".to_string(),
                }
            },
            GitStatusFormat::Text => {
                match status {
                    GitStatus::Clean => "clean".to_string(),
                    GitStatus::Dirty => {
                        let mut parts = Vec::new();
                        if changes.added > 0 { parts.push(format!("added:{}", changes.added)); }
                        if changes.modified > 0 { parts.push(format!("modified:{}", changes.modified)); }
                        if changes.deleted > 0 { parts.push(format!("deleted:{}", changes.deleted)); }
                        if parts.is_empty() { "dirty".to_string() } else { parts.join(" ") }
                    },
                    GitStatus::Conflicts => "conflicts".to_string(),
                }
            },
            GitStatusFormat::Count => {
                match status {
                    GitStatus::Clean => "clean".to_string(),
                    GitStatus::Dirty => {
                        let total = changes.added + changes.modified + changes.deleted;
                        if total > 0 {
                            format!("({} changes)", total)
                        } else {
                            "dirty".to_string()
                        }
                    },
                    GitStatus::Conflicts => "conflicts".to_string(),
                }
            },
        }
    }
}

impl Segment for GitSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let git_info = self.get_git_info(&input.workspace.current_dir)?;

        let mut metadata = HashMap::new();
        metadata.insert("branch".to_string(), git_info.branch.clone());
        metadata.insert("status".to_string(), format!("{:?}", git_info.status));
        metadata.insert("ahead".to_string(), git_info.ahead.to_string());
        metadata.insert("behind".to_string(), git_info.behind.to_string());
        metadata.insert("added".to_string(), git_info.status_counts.added.to_string());
        metadata.insert("modified".to_string(), git_info.status_counts.modified.to_string());
        metadata.insert("deleted".to_string(), git_info.status_counts.deleted.to_string());

        if let Some(ref sha) = git_info.sha {
            metadata.insert("sha".to_string(), sha.clone());
        }
        if let Some(stash_count) = git_info.stash_count {
            metadata.insert("stash_count".to_string(), stash_count.to_string());
        }
        if let Some(ref tag) = git_info.tag {
            metadata.insert("tag".to_string(), tag.clone());
        }

        let primary = git_info.branch;
        let mut status_parts = Vec::new();

        // Format status based on configuration
        let status_str = self.format_status(&git_info.status, &git_info.status_counts);
        if !status_str.is_empty() {
            status_parts.push(status_str);
        }

        // Add remote tracking info if enabled
        if self.config.show_remote {
            if git_info.ahead > 0 {
                status_parts.push(format!("↑{}", git_info.ahead));
            }
            if git_info.behind > 0 {
                status_parts.push(format!("↓{}", git_info.behind));
            }
        }

        // Add SHA if enabled
        if let Some(ref sha) = git_info.sha {
            status_parts.push(format!("@{}", sha));
        }

        // Add tag if enabled and available
        if let Some(ref tag) = git_info.tag {
            status_parts.push(format!("[{}]", tag));
        }

        // Add stash count if enabled and available
        if let Some(stash_count) = git_info.stash_count {
            status_parts.push(format!("{{{}}}", stash_count));
        }

        Some(SegmentData {
            primary,
            secondary: status_parts.join(" "),
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::Git
    }
}
