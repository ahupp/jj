// Copyright 2025 The Jujutsu Authors
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

use std::path::PathBuf;

use crate::common::TestEnvironment;

/// Integrating an already integrated operation is a no-op
#[test]
fn test_integrate_integrated_operation() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");

    let output = work_dir.run_jj(["op", "integrate", "@"]);
    insta::assert_snapshot!(output, @"");
    let output = work_dir.run_jj(["op", "log"]);
    insta::assert_snapshot!(output, @"
    @  f045ea2142cf test-username@host.example.com 2001-02-03 04:05:07.000 +07:00 - 2001-02-03 04:05:07.000 +07:00
    │  add workspace 'default'
    ○  000000000000 root()
    [EOF]
    ");
}

#[test]
fn test_integrate_sibling_operation() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");

    let base_op_id = work_dir.current_operation_id();
    work_dir.run_jj(["new", "-m=first"]).success();
    let unintegrated_id = work_dir.current_operation_id();
    assert_ne!(unintegrated_id, base_op_id);
    // Manually remove the last operation from the operation log
    let heads_dir = work_dir
        .root()
        .join(PathBuf::from_iter([".jj", "repo", "op_heads", "heads"]));
    std::fs::rename(
        heads_dir.join(&unintegrated_id),
        heads_dir.join(&base_op_id),
    )
    .unwrap();
    // We use --ignore-working-copy to prevent the automatic reloading of the repo
    // at the unintegrated operation that's mentioned in
    // `.jj/working_copy/checkout`.
    let output = work_dir.run_jj(["new", "-m=second", "--ignore-working-copy"]);
    insta::assert_snapshot!(output, @"");

    // The working copy should now be at the old unintegrated sibling operation
    let output = work_dir.run_jj(["op", "log"]);
    insta::assert_snapshot!(output, @"
    ------- stderr -------
    Internal error: The repo was loaded at operation dfb21b30fb23, which seems to be a sibling of the working copy's operation 725118ee780a
    Hint: Run `jj op integrate 725118ee780a` to add the working copy's operation to the operation log.
    [EOF]
    [exit status: 255]
    ");

    // Integrate the operation
    let output = work_dir.run_jj(["op", "integrate", &unintegrated_id]);
    insta::assert_snapshot!(output, @"
    ------- stderr -------
    The specified operation has been integrated with other existing operations.
    [EOF]
    ");
    let output = work_dir.run_jj(["op", "log"]);
    insta::assert_snapshot!(output, @"
    @    eca9e916852e test-username@host.example.com 2001-02-03 04:05:11.000 +07:00 - 2001-02-03 04:05:11.000 +07:00
    ├─╮  reconcile divergent operations
    │ │  args: jj op integrate 725118ee780aa7bb79dd4a830b429a5fdd11823107a0805635403d52c5029c4fe7da0ed18c9bb1a1c783f0ef0dfb768dce6ddc07b1d9aa4d8e6c3c918a14c71d
    ○ │  725118ee780a test-username@host.example.com 2001-02-03 04:05:08.000 +07:00 - 2001-02-03 04:05:08.000 +07:00
    │ │  new empty commit
    │ │  args: jj new '-m=first'
    │ ○  dfb21b30fb23 test-username@host.example.com 2001-02-03 04:05:09.000 +07:00 - 2001-02-03 04:05:09.000 +07:00
    ├─╯  new empty commit
    │    args: jj new '-m=second' --ignore-working-copy
    ○  f045ea2142cf test-username@host.example.com 2001-02-03 04:05:07.000 +07:00 - 2001-02-03 04:05:07.000 +07:00
    │  add workspace 'default'
    ○  000000000000 root()
    [EOF]
    ");
}
