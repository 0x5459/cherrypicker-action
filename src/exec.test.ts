import { describe, expect, test } from "@jest/globals";
import { ResultAsync } from "neverthrow";
import { CensoringExecutor } from './exec';
import { ExecOutput } from '@actions/exec'

describe.each([
    {
        name: "happy path with nothing to censor returns all output",
        dir: "/somewhere/repo",
        git: "/usr/bin/git",
        args: ["status"],
        censor: (content: string) => content,
        executeOut: "hi",
        executeErr: undefined,
        expectedOut: "hi",
        expectedErr: false,
    },
    {
        name: "happy path with nonstandard git binary",
        dir: "/somewhere/repo",
        git: "/usr/local/bin/git",
        args: ["status"],
        censor: (content: string) => content,
        executeOut: "hi",
        executeErr: undefined,
        expectedOut: "hi",
        expectedErr: false,
    },
    {
        name: "happy path with something to censor returns altered output",
        dir: "/somewhere/repo",
        git: "/usr/bin/git",
        args: ["status"],
        censor: (content: string) => {
            return content.replaceAll("secret", "CENSORED");
        },
        executeOut: "hi secret",
        executeErr: undefined,
        expectedOut: "hi CENSORED",
        expectedErr: false,
    },
    {
        name: "error is propagated",
        dir: "/somewhere/repo",
        git: "/usr/bin/git",
        args: ["status"],
        censor: (content: string) => {
            return content.replaceAll("secret", "CENSORED");
        },
        executeOut: "hi secret",
        executeErr: new Error("oops"),
        expectedOut: "hi CENSORED",
        expectedErr: true,
    },
])('test censoring executor', (testcase) => {
    test('test exec', () => {
        type Mutable<T> = {
            -readonly [k in keyof T]: T[k];
        };

        let executor = new CensoringExecutor(testcase.dir, testcase.censor) as Mutable<CensoringExecutor>;

        executor.git = testcase.git;
        executor._exec = (dir: string, commandLine: string, ...args: string[]): ResultAsync<ExecOutput, Error> => {
            expect(dir).toBe(testcase.dir);
            expect(commandLine).toBe(testcase.git);
            expect(args).toStrictEqual(testcase.args);

        }
    });
});
