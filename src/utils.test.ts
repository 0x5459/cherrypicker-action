import {describe, expect, test} from '@jest/globals';
import {
  isCherryPickInviteCommand,
  matchCherryPickCommand,
  matchLabel
} from './utils';

describe('test matchCherryPickCommand', () => {
  test('should match', () => {
    expect(matchCherryPickCommand('/cherrypick xx')).toStrictEqual({
      matched: true,
      branches: ['xx']
    });
    expect(matchCherryPickCommand('/cherry-pick xx')).toStrictEqual({
      matched: true,
      branches: ['xx']
    });
  });

  test('mismatch', () => {
    expect(matchCherryPickCommand('/cherrypickxxx')).toStrictEqual({
      matched: false,
      branches: []
    });
    expect(matchCherryPickCommand('/cherry-pickxxx')).toStrictEqual({
      matched: false,
      branches: []
    });
  });

  test('multiline', () => {
    expect(
      matchCherryPickCommand(`
/cherry-pick r
xxxx

/cherry-pick    releasev0.3
/cherrypick releasev0.3
/cherrypick release/v0.5
/cherrypick release/v0.5😊
        `)
    ).toStrictEqual({
      matched: true,
      branches: ['r', 'releasev0.3', 'release/v0.5', 'release/v0.5😊']
    });
  });
});

describe('test matchLabel', () => {
  test('normal', () => {
    const m = matchLabel(['needs-cherry-pick-', 'lbw']);
    expect(m('lbwnb')).toBe('nb');
    expect(m('needs-cherry-pick-xxx')).toBe('xxx');
    expect(m('needs-cherry-pick-lbw')).toBe('lbw');
  });

  test('empty label prefix', () => {
    const m = matchLabel([]);
    expect(m('lbwnb')).toBeNull();
    expect(m('needs-cherry-pick-xxx')).toBeNull();
    expect(m('needs-cherry-pick-lbw')).toBeNull();
  });
});

describe('test isCherryPickInviteCommand', () => {
  test('normal', () => {
    expect(isCherryPickInviteCommand('lbwnb')).toBeFalsy();
    expect(isCherryPickInviteCommand('/cherrypick-')).toBeFalsy();
    expect(isCherryPickInviteCommand('/cherry-pick')).toBeFalsy();
    expect(isCherryPickInviteCommand('/cherrypick-invite')).toBeTruthy();
    expect(isCherryPickInviteCommand('/cherry-pick-invite')).toBeTruthy();
  });

  test('Test word bound', () => {
    expect(isCherryPickInviteCommand('/cherrypick-invitexx')).toBeFalsy();
    expect(isCherryPickInviteCommand('/cherrypick-invite lbw')).toBeTruthy();
    expect(isCherryPickInviteCommand('/cherrypick-invite_lbw')).toBeFalsy();
  });
});
