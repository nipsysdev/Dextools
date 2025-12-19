import { atom } from "nanostores";
import type { Peer } from "./peersTypes";

export const $manualPeerConnections = atom<Peer[]>([])
export const $isAddPeerDialogOpened = atom(false)