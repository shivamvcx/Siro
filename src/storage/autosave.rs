use crate::model::project::Project;
use crate::storage::siro_file::SiroFileManager;
use std::time::Instant;

/// Background interval autosave manager.
/// Continuously sweeps open projects, saving eligible projects (contentful, unsaved, named)
/// to their respective files on disk at a user-configured interval.
pub struct AutosaveManager {
    pub last_sweep: Instant,
}

impl Default for AutosaveManager {
    fn default() -> Self {
        Self {
            last_sweep: Instant::now(),
        }
    }
}

impl AutosaveManager {
    /// Resets the autosave timer. Called when settings change or upon manual saves.
    pub fn reset_timer(&mut self) {
        self.last_sweep = Instant::now();
    }

    /// Performs an interval sweep across all open projects.
    ///
    /// Guard conditions:
    /// - If `interval_secs` is None (Autosave Off): does nothing, returns None.
    /// - If `is_modal_open` is true: suppresses sweep, returns None.
    /// - If elapsed time < interval_secs: returns None.
    ///
    /// For eligible projects (`is_autosave_eligible`), writes to `project.filepath`.
    /// On success, advances `saved_revision`. On failure, leaves `saved_revision` unchanged.
    /// Returns Some(Ok(count)) if projects were autosaved, or Some(Err(msg)) on failure.
    pub fn poll_autosave_all(
        &mut self,
        open_projects: &mut [Project],
        interval_secs: Option<u64>,
        is_modal_open: bool,
    ) -> Option<Result<usize, String>> {
        let interval = interval_secs?;
        if is_modal_open {
            return None;
        }

        if self.last_sweep.elapsed().as_secs() < interval {
            return None;
        }

        self.last_sweep = Instant::now();

        let mut saved_count = 0;
        let mut first_error: Option<String> = None;

        for project in open_projects.iter_mut() {
            if is_autosave_eligible(project, Some(interval)) {
                if let Some(path) = project.filepath.clone() {
                    match SiroFileManager::save_project(project, &path) {
                        Ok(()) => {
                            saved_count += 1;
                        }
                        Err(err) => {
                            if first_error.is_none() {
                                first_error = Some(format!(
                                    "Autosave failed for '{}': {}",
                                    project.display_name(),
                                    err
                                ));
                            }
                        }
                    }
                }
            }
        }

        if let Some(err) = first_error {
            Some(Err(err))
        } else if saved_count > 0 {
            Some(Ok(saved_count))
        } else {
            None
        }
    }
}

/// Pure eligibility predicate for autosave:
/// 1. Autosave is enabled (interval_secs is Some).
/// 2. Project has content (not a blank untouched document).
/// 3. Project is unsaved (edit_revision != saved_revision).
/// 4. Project is named (has a valid filepath on disk).
pub fn is_autosave_eligible(project: &Project, interval_secs: Option<u64>) -> bool {
    interval_secs.is_some()
        && project.has_content()
        && project.is_unsaved()
        && project.filepath.is_some()
}
