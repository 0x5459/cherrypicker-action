import download from 'download';
import os from 'os';
import path from 'path';
import {GithubClient} from './core';

export function matchCherryPickCommand(text: string): {
  matched: boolean;
  branches: string[];
} {
  const cherryPickRe: RegExp = /^(?:\/cherrypick|\/cherry-pick)\s+(.+)$/gm;
  const branches = unique(
    [...text.matchAll(cherryPickRe)].map(m => (m[1] as string).trim())
  );
  return {
    matched: branches.length > 0,
    branches: branches
  };
}

export function matchLabel(
  labelPrefix: string[]
): (label: string) => string | null {
  return (label: string) => {
    const matched = labelPrefix.find(prefix => label.startsWith(prefix, 0));
    if (matched == undefined) {
      return null;
    }
    return label.substring(matched.length);
  };
}

export function isCherryPickInviteCommand(text: string): boolean {
  const cherryPickInviteRe: RegExp =
    /^(?:\/cherrypick|\/cherry-pick)-invite\b/gm;
  return cherryPickInviteRe.test(text);
}

export function downloadPatch(gc: GithubClient) {
  return async (
    p: {
      owner: string;
      repo: string;
      pull_number: number;
    },
    targetBranch: string
  ) => {
    const pr = await gc.pulls.get(p);

    const target = path.join(
      os.tmpdir(),
      `${p.owner}-${p.repo}-${p.pull_number}-${normalize(targetBranch)}`
    );
    await download(pr.patch_url, target);
    return target;
  };
}

// `ensureFork` checks to see that there is a fork of org/repo in the forkedUsers repositories.
// If there is not, it makes one, and waits for the fork to be created before returning.
// The return value is the name of the repo that was created
// (This may be different then the one that is forked due to naming conflict)
export function ensureFork(gc: GithubClient) {
  const waitForRepo1 = waitForRepo(gc);
  return async (q: {forkingUser: string; owner: string; repo: string}) => {
    if (await isForked(gc, q)) {
      return q.repo;
    }

    const forked = await gc.repos.createFork(q);
    await waitForRepo({owner: q.forkingUser, repo: q.repo});
    return q.repo;
  };
}

// Returns true if forkingUser forked owner/repo
async function isForked(
  gc: GithubClient,
  q: {forkingUser: string; owner: string; repo: string}
) {
  const fork = `${q.forkingUser}/${q.repo}`;
  const repos = await gc.repos.listForUser({username: q.forkingUser});
  const forkedRepo = repos.find(repo => repo.fork && repo.full_name === fork);
  if (forkedRepo === undefined) {
    return false;
  }
  const parentFullName = await gc.repos
    .get({owner: q.forkingUser, repo: forkedRepo.name})
    .then(r => r.parent?.full_name);
  if (parentFullName === undefined) {
    return false;
  }
  return parentFullName === `${q.owner}/${q.repo}`;
}

function waitForRepo(gc: GithubClient) {
  return async (q: {owner: string; repo: string}) => {
    const repo = await gc.repos.get(q);
    new Promise(resolve => {
      setTimeout(resolve, 1000 * 10);
    });
  };
}

function unique<T>(iterable: Iterable<T> | undefined) {
  return Array.from(new Set(iterable));
}

function normalize(input: string): string {
  return input.replaceAll('/', '-');
}
