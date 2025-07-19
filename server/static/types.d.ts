// TypeScript type definitions for FileshareCloud frontend
export interface FileMeta {
    name: string;
    size: number;
    modified: string;
    hash: string;
    owner: string;
    permissions: string[];
}

export interface User {
    id: string;
    username: string;
    role: string;
    email: string;
}

export interface SyncStatus {
    connected: boolean;
    lastSync: string;
    pending: number;
}
