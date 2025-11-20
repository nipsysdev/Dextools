import { atom } from 'nanostores';

export enum CodexConnectionStatus {
  Disconnected = 'Disconnected',
  Connecting = 'Connecting',
  Connected = 'Connected',
  Error = 'Error',
}

export const $isCodexDialogOpened = atom(false);
export const $codexStatus = atom<CodexConnectionStatus>(CodexConnectionStatus.Disconnected);
export const $codexError = atom<string | null>(null);
export const $codexPeerId = atom<string | null>(null);
export const $codexVersion = atom<string | null>(null);
export const $nodeAddresses = atom<string[]>([]);