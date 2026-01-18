// Copyright 2020-2023 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use jj_lib::git;

use crate::cli_util::CommandHelper;
use crate::command_error::CommandError;
use crate::git_util::print_git_export_stats;
use crate::ui::Ui;

/// Update the underlying Git repo with changes made in the repo
///
/// There is no need to run this command if you're in colocated workspace
/// because the export happens automatically there.
#[derive(clap::Args, Clone, Debug)]
pub struct GitExportArgs {}

pub fn cmd_git_export(
    ui: &mut Ui,
    command: &CommandHelper,
    _args: &GitExportArgs,
) -> Result<(), CommandError> {
    let mut workspace_command = command.workspace_helper(ui)?;
    let working_copy_shared_with_git = workspace_command.working_copy_shared_with_git();
    let git_repo = if working_copy_shared_with_git {
        Some(crate::git_util::open_git_repo_for_workspace(
            workspace_command.workspace(),
        )?)
    } else {
        None
    };
    let mut tx = workspace_command.start_transaction();
    let stats = if let Some(git_repo) = git_repo {
        git::export_refs_with_repo(tx.repo_mut(), &git_repo)?
    } else {
        git::export_refs(tx.repo_mut())?
    };
    tx.finish(ui, "export git refs")?;
    print_git_export_stats(ui, &stats)?;
    Ok(())
}
