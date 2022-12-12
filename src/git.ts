import {Git} from './core';
import exec from '@actions/exec';

// GitUserGetter fetches a name and email for us in git commits on-demand
type GitUserGetter = () => {name: string; email: string} | error;

class Git1 implements Git {
  readonly info: GitUserGetter;

  directory(): string {
    return '';
  }

  async commit(x: {title: string; body: string}) {
    // this.info(),
    const name = '',
      email = '';
    const commands = [
      ['add', '--all'],
      [
        'commit',
        '--message',
        x.title,
        '--message',
        x.body,
        '--author',
        `${name} <${email}>`
      ]
    ];
  }

  async pushToNamedFork(x: {
    forkName: string;
    branch: string;
    force: boolean;
  }) {}

  async clean() {}

  async am(x: {path: string}) {}
  async checkout(x: {commitlike: string}) {}
  async checkoutNewBranch(x: {branch: string}) {}

  async branchExists(x: {branch: string}) {
    return true;
  }

  async config(...args: string[]) {}

  private async exec(commandLine: string, args?: string[]) {
    exec.exec(commandLine, args, {
      cwd: ''
    });
  }
}
