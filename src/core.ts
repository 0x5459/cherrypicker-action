// function all<P>(p: P): number {

// }

export interface Git {
  // Directory exposes the directory in which the repository has been cloned
  readonly directory: () => string;

  // Commit stages all changes and commits them with the message
  readonly commit: (x: {title: string; body: string}) => Promise<void>;

  // PushToCentral pushes the local state to the central remote
  readonly pushToNamedFork: (x: {
    forkName: string;
    branch: string;
    force: boolean;
  }) => Promise<void>;

  // Clean removes the repository. It is up to the user to call this once they are done
  readonly clean: () => Promise<void>;

  // Am calls `git am`
  readonly am: (x: {path: string}) => Promise<void>;

  // Checkout runs `git checkout`
  readonly checkout: (x: {commitlike: string}) => Promise<void>;

  // CheckoutNewBranch creates a new branch from HEAD and checks it out
  readonly checkoutNewBranch: (x: {branch: string}) => Promise<void>;

  // BranchExists determines if a branch with the name exists
  readonly branchExists: (x: {branch: string}) => Promise<boolean>;

  // Config runs `git config`
  readonly config: (...args: string[]) => Promise<void>;
}

// https://docs.github.com/developers/webhooks-and-events/webhooks/webhook-events-and-payloads#issue_comment
export interface IssueCommentEvent {
  readonly action: 'created' | 'edited' | 'deleted';
  readonly comment: {
    body: string;
  };
  readonly issue: {
    state: 'open' | 'closed';
    pull_request?: unknown;
  };
  readonly repository: Repo;
}

// https://docs.github.com/developers/webhooks-and-events/webhooks/webhook-events-and-payloads#pull_request
export interface PullRequestEvent {
  readonly action:
    | 'assigned'
    | 'auto_merge_enabled'
    | 'auto_merge_disabled'
    | 'closed'
    | 'converted_to_draft'
    | 'demilestoned'
    | 'dequeued'
    | 'edited'
    | 'labeled'
    | 'locked'
    | 'milestoned'
    | 'opened'
    | 'ready_for_review'
    | 'reopened'
    | 'review_request_removed'
    | 'review_requested'
    | 'synchronize'
    | 'unassigned'
    | 'unlabeled'
    | 'unlocked';
  readonly label?: Label;
  readonly number: number;
  readonly pull_request: PullRequest;
  readonly repository: Repo;
}

export interface GithubClient {
  readonly issues: Issues;
  readonly pulls: Pulls;
  readonly orgs: Orgs;
  readonly repos: Repos;
}

interface Issues {
  // https://docs.github.com/en/rest/issues/issues?apiVersion=2022-11-28#create-an-issue
  readonly create: (x: {
    owner: string;
    repo: string;
    title: string;
    body: string;
    milestone?: (string | number) | null;
    labels?: string[];
    assignees?: string[];
  }) => Promise<void>;

  // https://docs.github.com/en/rest/issues/labels?apiVersion=2022-11-28#add-labels-to-an-issue
  readonly addLabels: (x: {
    owner: string;
    repo: string;
    issue_number: number;
    labels: string[];
  }) => Promise<void>;

  // https://docs.github.com/en/rest/issues/labels?apiVersion=2022-11-28#list-labels-for-an-issue
  readonly listLabelsOnIssue: (x: {
    owner: string;
    repo: string;
    issue_number: number;
  }) => Promise<Label>;

  // https://docs.github.com/en/rest/issues/comments?apiVersion=2022-11-28#list-issue-comments-for-a-repository
  readonly listComments: (x: {
    owner: string;
    repo: string;
    issue_number: number;
  }) => Promise<IssueComment[]>;

  // https://docs.github.com/en/rest/issues/comments?apiVersion=2022-11-28#create-an-issue-comment
  readonly createComment: (x: {
    owner: string;
    repo: string;
    issue_number: number;
    body: string;
  }) => Promise<IssueComment>;

  // https://docs.github.com/en/rest/issues/assignees?apiVersion=2022-11-28#add-assignees-to-an-issue
  readonly addAssignees: (x: {
    owner: string;
    repo: string;
    issue_number: number;
    assignees?: string[];
  }) => Promise<void>;
}

interface Pulls {
  // https://docs.github.com/en/rest/pulls/pulls?apiVersion=2022-11-28#create-a-pull-request
  readonly create: (x: {
    owner: string;
    repo: string;
    body: string;
    head: string;
    base: string;
  }) => Promise<PullRequest>;

  // https://docs.github.com/en/rest/pulls/pulls?apiVersion=2022-11-28#get-a-pull-request
  readonly get: (x: {
    owner: string;
    repo: string;
    pull_number: number;
  }) => Promise<PullRequest>;

  // https://docs.github.com/en/rest/pulls/pulls?apiVersion=2022-11-28#list-pull-requests
  readonly list: (x: {
    owner: string;
    repo: string;
    head?: string;
  }) => Promise<PullRequestSimple[]>;
}

interface Orgs {
  // https://docs.github.com/en/rest/orgs/members?apiVersion=2022-11-28#check-organization-membership-for-a-user
  readonly checkMembershipForUser: (x: {
    org: string;
    username: string;
  }) => Promise<boolean>;

  // https://docs.github.com/en/rest/orgs/members?apiVersion=2022-11-28#list-organization-members
  readonly listMembers: (x: {org: string}) => Promise<SimpleUser[]>;
}

interface Repos {
  // https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#get-a-repository
  readonly get: (x: {owner: string; repo: string}) => Promise<FullRepo>;

  // https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#list-repositories-for-a-user
  readonly listForUser: (x: {username: string}) => Promise<MinimalRepo[]>;

  // https://docs.github.com/en/rest/repos/forks?apiVersion=2022-11-28#create-a-fork
  readonly createFork: (x: {
    owner: string;
    repo: string;
    organization?: string;
    name?: string;
  }) => Promise<FullRepo>;

  // https://docs.github.com/en/rest/commits/commits?apiVersion=2022-11-28#get-a-commit
  readonly getCommit: (x: {
    owner: string;
    repo: string;
    ref: string;
  }) => Promise<Commit>;

  // https://docs.github.com/en/rest/collaborators/collaborators?apiVersion=2022-11-28#check-if-a-user-is-a-repository-collaborator
  readonly checkCollaborator: (x: {
    owner: string;
    repo: string;
    username: string;
  }) => Promise<boolean>;

  // https://docs.github.com/en/rest/collaborators/collaborators?apiVersion=2022-11-28#add-a-repository-collaborator
  readonly addCollaborator: (x: {
    owner: string;
    repo: string;
    username: string;
    permission?: 'pull' | 'push' | 'admin' | 'maintain' | 'triage';
  }) => Promise<void>;
}

interface Issue {}

interface Label {
  id: number;
  node_id: string;
  url: string;
  name: string;
  description: string | null;
  color: string;
  default: boolean;
}

export interface IssueComment {
  body: string;
}

interface FullRepo {
  parent?: Repo;
}

interface Repo {
  name: string;
  full_name: string;
}

interface MinimalRepo {
  name: string;
  fork: boolean;
  full_name: string;
}

interface PullRequestSimple {}

interface PullRequest {
  patch_url: string;
}

interface Commit {}

interface SimpleUser {}
