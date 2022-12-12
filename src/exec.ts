import { ExecOutput, getExecOutput } from "@actions/exec";
import { lookpath } from "lookpath";
import { ResultAsync } from "neverthrow";

interface Exec {
  readonly exec: (commandLine: string, ...args: string[]) => ResultAsync<ExecOutput, Error>;
}


// Censor censors content to remove secrets
type Censor = (content: string) => string


export class CensoringExecutor implements Exec {

  readonly dir: string;
  readonly _exec: (dir: string, commandLine: string, ...args: string[]) => ResultAsync<ExecOutput, Error>;
  readonly censor: Censor;
  git: string | undefined;

  constructor(dir: string, censor: Censor) {
    this.dir = dir;
    this._exec = (dir: string, commandLine: string, ...args: string[]) => {
      return ResultAsync.fromPromise(getExecOutput(commandLine, args, {
        cwd: dir,
      }), e => e as Error);
    };
    this.censor = censor;
  }

  async init() {
    this.git = await lookpath("git");
  }

  exec(commandLine: string, ...args: string[]): ResultAsync<ExecOutput, Error> {
    return this._exec(this.dir, commandLine, ...args).map((out) => {
      out.stdout = this.censor(out.stdout);
      out.stderr = this.censor(out.stderr);
      return out;
    });
  }
}
