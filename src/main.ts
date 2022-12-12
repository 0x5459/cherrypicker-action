import core from '@actions/core';
import github from '@actions/github';
import {GithubClient, IssueCommentEvent, PullRequestEvent} from './core';
import {isCherryPickInviteCommand, matchCherryPickCommand} from './utils';

interface Opts {
  // Specifies whether everyone is allowed to cherry pick.
  allow_all: boolean;
  // Specifies whether to create an Issue when there is a PR conflict.
  create_issue_on_conflict: boolean;
  // Specifies the label prefix for cherrypicker.
  label_prefix: string;
  // Specifies the label prefix after picked.
  picked_label_prefix: string;
  // Specifies the labels that need to be excluded when copying the labels of the original PR.
  exclude_labels: string[];
  // Specifies whether to copy the issue numbers from the squashed commit message.
  copy_issue_numbers_from_squashed_commit: boolean;
}

class Cherrypicker {
  readonly gc: GithubClient;
  readonly opts: Opts;

  constructor(opts: Opts, gc: GithubClient) {
    this.opts = opts;
    this.gc = gc;
  }

  async onIssueComment(event: Readonly<IssueCommentEvent>) {
    // Only consider new comments in PRs.
    if (event.action !== 'created' || !event.issue.pull_request) {
      return;
    }
    if (isCherryPickInviteCommand(event.comment.body)) {
    }
    if (matchCherryPickCommand(event.comment.body)) {
    }
  }

  async onPullRequest(event: Readonly<PullRequestEvent>) {
    if (event.action !== 'labeled') {
      return;
    }
  }

  async cherryPick(targetBranches: string[]) {}
}

async function main() {
  const opts: Opts = {
    allow_all: core.getBooleanInput('allow-all'),
    create_issue_on_conflict: core.getBooleanInput('create-issue-on-conflict'),
    label_prefix: core.getInput('label-prefix'),
    picked_label_prefix: core.getInput('picked-label-prefix'),
    exclude_labels: core.getMultilineInput('exclude-labels'),
    copy_issue_numbers_from_squashed_commit: core.getBooleanInput(
      'copy-issue-numbers-from-squashed-commit'
    )
  };
  const token = core.getInput('repo-token', {required: true});
  const cherrypicker = new Cherrypicker(opts, github.getOctokit(token).rest);

  const eventName = github.context.eventName;
  const payload = github.context.payload as unknown as
    | IssueCommentEvent
    | PullRequestEvent;

  if (isIssueComment(github.context.eventName, payload)) {
    cherrypicker.onIssueComment(payload);
  } else if (isPullRequest(eventName, payload)) {
    cherrypicker.onPullRequest(payload);
  }
}

function isIssueComment(
  eventName: string,
  payload: IssueCommentEvent | PullRequestEvent
): payload is IssueCommentEvent {
  return eventName == 'issue_comment';
}

function isPullRequest(
  eventName: string,
  payload: IssueCommentEvent | PullRequestEvent
): payload is PullRequestEvent {
  return eventName == 'pull_request';
}

main();
